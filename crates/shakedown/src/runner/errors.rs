//! The script errors of a check execution (`ERR-01` to `ERR-04`): those the machines' handlers
//! sent, those only their RPTs show, the `expectError` blocks that expected them, and the outcome
//! they decide with the lead's report (`DM-02`, `DM-03`, `DM-06`).

use serde_json::{Value, json};
use shakedown_protocol::result::{CheckExecution, Reason, Step};

use crate::logs;

fn number(v: &Value) -> f64 {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0.0)
}

/// A `ScriptError` of the result file from the message a machine's handler sent. `_compile`
/// marks an error raised while the agent compiled code it was sent, which the check reports as
/// `check-error` itself; `judge` removes it from the shape.
pub fn from_handler(e: &Value, check_id: &str) -> Value {
    let at = &e["at"];
    json!({
        "machine": e["machine"],
        "text": e["text"].as_str().unwrap_or_default(),
        "file": e["file"].as_str().unwrap_or_default(),
        "line": e["line"].as_f64().map(|l| l as i64),
        "stack": e["stack"].as_array().cloned().unwrap_or_default(),
        "check_id": check_id,
        "step_path": e["step_path"].as_array().cloned().unwrap_or_default(),
        "source": "handler",
        "expected": false,
        "at": {
            "utc": at["utc"].as_str().unwrap_or_default(),
            "tick": number(&at["tick"]),
            "time": number(&at["time"]),
            "frame": number(&at["frame"]).max(0.0) as u64,
        },
        "_compile": e["compile"].as_bool().unwrap_or(false),
    })
}

/// Adds the script errors a machine's lines for a check show that its handler did not send
/// (`ERR-02`). Each RPT error is matched to a handler error of the same machine with the same text
/// and file, each handler error once; the rest are the handler's misses, with `source` `rpt` and
/// the wall-clock time of their RPT line to the second, from the start marker's time.
pub fn from_rpt(errors: &mut Vec<Value>, machine: &str, check_id: &str, lines: &str) {
    let first = lines.lines().next().unwrap_or_default();
    let mut used = vec![false; errors.len()];
    for re in logs::scan_errors(lines) {
        let found = errors.iter().enumerate().position(|(i, e)| {
            !used[i]
                && e["source"] == "handler"
                && e["machine"] == machine
                && e["text"] == re.text.as_str()
                && e["file"].as_str().unwrap_or_default().trim_start_matches('\\') == re.file.trim_start_matches('\\')
        });
        if let Some(i) = found {
            used[i] = true;
            continue;
        }
        let utc = re
            .local_s
            .and_then(|s| logs::utc_of(first, s))
            .unwrap_or_else(shakedown_protocol::time::now);
        errors.push(json!({
            "machine": machine,
            "text": re.text,
            "file": re.file,
            "line": re.line,
            "stack": [],
            "check_id": check_id,
            "step_path": [],
            "source": "rpt",
            "expected": false,
            "at": {"utc": utc, "tick": 0.0, "time": 0.0, "frame": 0},
            "_compile": false,
        }));
        used.push(true);
    }
}

/// The step that ran on the lead at a wall-clock time: the innermost one running then, or else
/// the last one that had started (`P-05` step 7).
fn step_at(steps: &[Step], utc: &str) -> Vec<String> {
    let running = steps
        .iter()
        .filter(|s| s.start.utc.as_str() <= utc && s.end.as_ref().is_none_or(|e| e.utc.as_str() >= utc))
        .max_by_key(|s| s.path.len());
    running
        .or_else(|| {
            steps
                .iter()
                .filter(|s| s.start.utc.as_str() <= utc)
                .max_by(|a, b| a.start.utc.cmp(&b.start.utc))
        })
        .map(|s| s.path.clone())
        .unwrap_or_default()
}

fn utc(e: &Value) -> String {
    e["at"]["utc"].as_str().unwrap_or_default().to_string()
}

/// The latest time an error can have happened: an RPT error's time is to the second, so it counts
/// as the end of that second when it is compared with the times of the check.
fn latest(e: &Value) -> String {
    let t = utc(e);
    if e["source"] == "rpt" {
        t.replace(".000Z", ".999Z")
    } else {
        t
    }
}

/// Puts the errors into the execution and decides what they change of its outcome (`ERR-03`,
/// `ERR-04`): an error with no step takes the lead's step at its time; an error whose text holds
/// the text of an `expectError` block started before it is expected; an unexpected error after the
/// body was sent fails the check `script-error` unless an earlier cause decided it (`DM-06`); and
/// a block no error matched fails a check that would otherwise pass `expected-error-missing`.
pub fn judge(execution: &mut CheckExecution, mut errors: Vec<Value>, expects: &Value, executing_at: Option<&str>) {
    for e in errors.iter_mut() {
        if e["step_path"].as_array().is_none_or(Vec::is_empty) {
            // An RPT error's time is to the second: the step at the start of that second, or else
            // at its end.
            let mut path = step_at(&execution.steps, &utc(e));
            if path.is_empty() {
                path = step_at(&execution.steps, &latest(e));
            }
            e["step_path"] = json!(path);
        }
    }
    errors.sort_by_key(utc);
    let expects: Vec<(String, String, Vec<String>, String)> = expects
        .as_array()
        .into_iter()
        .flatten()
        .map(|x| {
            (
                x["text"].as_str().unwrap_or_default().to_lowercase(),
                x["from"].as_str().unwrap_or_default().to_string(),
                x["path"]
                    .as_array()
                    .map(|p| p.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                    .unwrap_or_default(),
                x["text"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect();
    let mut met = vec![false; expects.len()];
    for e in errors.iter_mut() {
        let text = e["text"].as_str().unwrap_or_default().to_lowercase();
        for (i, (want, from, _, _)) in expects.iter().enumerate() {
            if !want.is_empty() && text.contains(want.as_str()) && latest(e).as_str() >= from.as_str() {
                e["expected"] = json!(true);
                met[i] = true;
            }
        }
    }
    let unexpected = errors.iter().find(|e| {
        e["expected"] == false && e["_compile"] != true && executing_at.is_some_and(|at| latest(e).as_str() >= at)
    });
    let decided_at = execution.reason.as_ref().map(|r| r.at.clone());
    if let Some(e) = unexpected
        && decided_at.as_ref().is_none_or(|at| utc(e) < *at)
    {
        let machine = e["machine"].as_str().unwrap_or_default().to_string();
        let path: Vec<String> = e["step_path"]
            .as_array()
            .map(|p| p.iter().filter_map(|s| s.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let place = match (e["file"].as_str().unwrap_or_default(), e["line"].as_i64()) {
            ("", _) => String::new(),
            (file, Some(line)) => format!(" ({file}, line {line})"),
            (file, None) => format!(" ({file})"),
        };
        let step = if path.is_empty() {
            "outside any step".to_string()
        } else {
            format!("in step \"{}\"", path.join(" > "))
        };
        let message = format!(
            "A script error on {machine} {step}: {}{place}. Fix the error in the mod, or wrap the code that should raise it in skd_fnc_expectError.",
            e["text"].as_str().unwrap_or_default()
        );
        if let Some(s) = execution.steps.iter_mut().find(|s| s.path == path) {
            s.state = "failed".into();
            s.error = Some(message.clone());
        }
        execution.outcome = "fail".into();
        execution.reason = Some(Reason {
            kind: "script-error".into(),
            message,
            machine: Some(machine),
            at: utc(e),
        });
        execution.evidence.failed_step = (!path.is_empty()).then_some(path);
    } else if execution.outcome == "pass"
        && let Some(i) = met.iter().position(|m| !m)
    {
        let (_, _, path, text) = &expects[i];
        execution.outcome = "fail".into();
        execution.reason = Some(Reason {
            kind: "expected-error-missing".into(),
            message: format!(
                "No script error containing \"{text}\" was caught on any machine from the start of its skd_fnc_expectError block to the end of the check. Check that the code in the block still raises it."
            ),
            machine: None,
            at: shakedown_protocol::time::now(),
        });
        execution.evidence.failed_step = (!path.is_empty()).then_some(path.clone());
    }
    for e in errors.iter_mut() {
        if let Some(o) = e.as_object_mut() {
            o.remove("_compile");
        }
    }
    execution.errors = errors;
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakedown_protocol::result::{Clocks, Evidence};

    fn clocks(utc: &str) -> Clocks {
        Clocks {
            utc: utc.into(),
            tick: 1.0,
            time: 1.0,
            frame: 1,
        }
    }

    fn execution(outcome: &str, reason: Option<Reason>) -> CheckExecution {
        CheckExecution {
            check_id: "c".into(),
            name: "US-028 c".into(),
            tags: vec![],
            repeat: 1,
            outcome: outcome.into(),
            reason,
            started_at: "2026-11-06T10:00:00.000Z".into(),
            ended_at: "2026-11-06T10:00:09.000Z".into(),
            duration_s: 9.0,
            reload: "full".into(),
            steps: vec![
                Step {
                    path: vec!["trigger on p2".into()],
                    machine: "server".into(),
                    state: "passed".into(),
                    start: clocks("2026-11-06T10:00:02.000Z"),
                    end: Some(clocks("2026-11-06T10:00:03.000Z")),
                    duration_s: 1.0,
                    error: None,
                },
                Step {
                    path: vec!["later".into()],
                    machine: "server".into(),
                    state: "passed".into(),
                    start: clocks("2026-11-06T10:00:05.000Z"),
                    end: Some(clocks("2026-11-06T10:00:06.000Z")),
                    duration_s: 1.0,
                    error: None,
                },
            ],
            errors: vec![],
            evidence: Evidence::default(),
        }
    }

    fn error(machine: &str, text: &str, utc: &str, path: &[&str]) -> Value {
        from_handler(
            &json!({"machine": machine, "text": text, "file": "skdfx\\addons\\main\\functions\\fn_throw.sqf", "line": 4,
                     "stack": ["skdfx\\addons\\main\\functions\\fn_throw.sqf:5"], "step_path": path, "compile": false,
                     "at": {"utc": utc, "tick": "10.5", "time": "3.25", "frame": "120"}}),
            "c",
        )
    }

    const SENT: &str = "2026-11-06T10:00:01.000Z";

    #[test]
    fn us_028_an_unexpected_error_fails_the_check_naming_machine_file_and_step() {
        let mut e = execution("pass", None);
        let errors = vec![error(
            "p2",
            "Undefined variable in expression: _skdfx_undefined",
            "2026-11-06T10:00:02.500Z",
            &["trigger on p2"],
        )];
        judge(&mut e, errors, &Value::Null, Some(SENT));
        assert_eq!(e.outcome, "fail");
        let r = e.reason.unwrap();
        assert_eq!(r.kind, "script-error");
        assert_eq!(r.machine.as_deref(), Some("p2"));
        assert!(
            r.message.contains("_skdfx_undefined") && r.message.contains("fn_throw.sqf"),
            "{}",
            r.message
        );
        assert_eq!(e.evidence.failed_step, Some(vec!["trigger on p2".to_string()]));
        assert_eq!(e.errors[0]["source"], "handler");
        assert_eq!(e.errors[0]["at"]["frame"], 120);
        assert!(e.errors[0].get("_compile").is_none());
    }

    #[test]
    fn us_028_an_error_without_a_step_takes_the_lead_step_at_its_time() {
        let mut e = execution("pass", None);
        let errors = vec![
            error("p2", "x", "2026-11-06T10:00:02.500Z", &[]),
            error("p2", "y", "2026-11-06T10:00:04.000Z", &[]),
        ];
        judge(&mut e, errors, &Value::Null, Some(SENT));
        assert_eq!(e.errors[0]["step_path"], json!(["trigger on p2"]));
        // Between two steps: the last one that started.
        assert_eq!(e.errors[1]["step_path"], json!(["trigger on p2"]));
    }

    #[test]
    fn us_027_a_matching_error_is_expected_case_insensitively_and_others_still_fail() {
        let expects =
            json!([{"text": "_SKDFX_Undefined", "from": "2026-11-06T10:00:02.000Z", "path": ["trigger on p2"]}]);
        let mut e = execution("pass", None);
        judge(
            &mut e,
            vec![error(
                "p2",
                "Undefined variable in expression: _skdfx_undefined",
                "2026-11-06T10:00:02.500Z",
                &[],
            )],
            &expects,
            Some(SENT),
        );
        assert_eq!(e.outcome, "pass");
        assert_eq!(e.errors[0]["expected"], true);

        let mut e = execution("pass", None);
        let errors = vec![
            error(
                "p2",
                "Undefined variable in expression: _skdfx_undefined",
                "2026-11-06T10:00:02.500Z",
                &[],
            ),
            error("p2", "Generic error in expression", "2026-11-06T10:00:02.600Z", &[]),
        ];
        judge(&mut e, errors, &expects, Some(SENT));
        assert_eq!(e.reason.unwrap().kind, "script-error");
    }

    #[test]
    fn us_029_an_expected_error_that_never_comes_fails_quoting_its_text() {
        let expects = json!([{"text": "_skdfx_undefined", "from": "2026-11-06T10:00:02.000Z", "path": ["expect it"]}]);
        let mut e = execution("pass", None);
        judge(&mut e, vec![], &expects, Some(SENT));
        let r = e.reason.unwrap();
        assert_eq!(r.kind, "expected-error-missing");
        assert!(r.message.contains("_skdfx_undefined"));
        assert_eq!(e.evidence.failed_step, Some(vec!["expect it".to_string()]));
    }

    #[test]
    fn err_03_an_earlier_cause_decides_and_errors_before_the_body_do_not_fail() {
        let reason = Reason {
            kind: "assertion".into(),
            message: "m".into(),
            machine: None,
            at: "2026-11-06T10:00:02.200Z".into(),
        };
        let mut e = execution("fail", Some(reason));
        judge(
            &mut e,
            vec![error("p2", "x", "2026-11-06T10:00:02.500Z", &[])],
            &Value::Null,
            Some(SENT),
        );
        assert_eq!(e.reason.unwrap().kind, "assertion");
        assert_eq!(e.errors.len(), 1);

        let mut e = execution("pass", None);
        judge(
            &mut e,
            vec![error("p2", "x", "2026-11-06T10:00:00.500Z", &[])],
            &Value::Null,
            Some(SENT),
        );
        assert_eq!(e.outcome, "pass");
    }

    #[test]
    fn us_030_an_rpt_error_the_handler_missed_is_added_with_source_rpt() {
        let lines = " 9:00:01 [shakedown] r-1#1 start 2026-11-06T08:00:01.250Z\n 9:00:03 Error in expression <x + 1>\n 9:00:03   Error position: <x + 1>\n 9:00:03   Error Undefined variable in expression: _skdfx_undefined\n 9:00:03 File skdfx\\addons\\main\\functions\\fn_throw.sqf..., line 4\n 9:00:04 \"Error in expression <made by hand>\"\n 9:00:05 [shakedown] r-1#1 end 2026-11-06T08:00:05.000Z\n";
        let mut errors = vec![error(
            "p1",
            "Undefined variable in expression: _skdfx_undefined",
            "2026-11-06T08:00:03.100Z",
            &[],
        )];
        from_rpt(&mut errors, "p1", "c", lines);
        assert_eq!(errors.len(), 2, "{errors:#?}");
        assert_eq!(errors[1]["source"], "rpt");
        assert_eq!(errors[1]["at"]["utc"], "2026-11-06T08:00:04.000Z");
    }
}
