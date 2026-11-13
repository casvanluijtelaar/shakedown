"""The acceptance checks of M3 Script errors and logs: US-024, US-027, US-028, US-029, US-030, US-064,
US-065, the demo, DEF-09, and exit criterion 2 (NFR-RL-05).

The Steam ID and the Steam names are read from the machine at run time (identity()) and never
written anywhere: a message names a hit only as "the Steam ID", "a Steam ID" or "the Steam name".
"""

import os
import re
import subprocess
from pathlib import Path

from .core import batch, check, execution
from .m01 import ended, passed, stopped_in

WAIT_CEILING = "us024/US-024-wait-ceiling"
EXPECT_EACH = "us024/US-024-expect-each"
DEFAULT_CEILING = "us024/US-024-default-ceiling"
EXPECTED = "us027/US-027-expected-on-p2"
CASE = "us027/US-027-case-insensitive"
OTHER = "us027/US-027-other-error"
ON_P2 = "us028/US-028-on-p2"
SPAWN = "us028/US-028-spawn"
REMOTE_EXEC = "us028/US-028-remote-exec"
NEVER = "us029/US-029-never-comes"
RPT_ONLY = "us030/US-030-rpt-only"
NAMES = "us064/US-064-names"
IDENTITY = "us065/US-065-identity"
DEMO = "agh/AGH-US-024-rope-never-held-by-p2"

# One run of these per repeat; the checks whose story says "the run exits 1" run alone.
DED_ORDER = [EXPECT_EACH, DEFAULT_CEILING, EXPECTED, CASE, OTHER, SPAWN, REMOTE_EXEC, RPT_ONLY, NAMES]
ALONE = [WAIT_CEILING, ON_P2, NEVER]
DED_MACHINES = ["server", "p1", "p2", "hc1"]
THROW_FILE = "fn_throw.sqf"


# ---------------------------------------------------------------- the Steam identity

def identity():
    """(the Steam ID, [account name, persona name]) of the account INT-STM-04 names, read from the machine."""
    import winreg
    key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, r"Software\Valve\Steam")
    steam = winreg.QueryValueEx(key, "SteamPath")[0]
    text = (Path(steam) / "config" / "loginusers.vdf").read_text(encoding="utf-8", errors="replace")
    users = []
    for sid, body in re.findall(r'"(\d{17})"\s*\{(.*?)\}', text, re.S):
        kv = {a.lower(): b for a, b in re.findall(r'"(\w+)"\s+"([^"]*)"', body)}
        users.append((kv.get("mostrecent") == "1", int(kv.get("timestamp", "0") or 0), sid,
                      [kv.get("accountname", ""), kv.get("personaname", "")]))
    assert users, "loginusers.vdf names no Steam account; the identity checks cannot run"
    _, _, sid, names = max(users)
    return sid, [n for n in names if n]


def leaks(text, ident):
    """What of the identity the text holds, by description only: never the values themselves."""
    sid, names = ident
    found = []
    if sid in text:
        found.append("the Steam ID")
    elif re.search(r"7656119\d{10}", text):
        found.append("a Steam ID")
    for n in names:
        if re.search(rf"(?<!\w){re.escape(n)}(?!\w)", text):
            found.append("the Steam name")
            break
    return found


def search_files(paths, ident):
    """Every (path, what it holds) among the files."""
    hits = []
    for p in paths:
        found = leaks(Path(p).read_text(encoding="utf-8", errors="replace"), ident)
        if found:
            hits.append((str(p), found))
    return hits


# ---------------------------------------------------------------- RPT lines (INT-AR-07)

TIMED = re.compile(r"^\s?(\d{1,2}):(\d{2}):(\d{2}) (.*)$")
MARKER = re.compile(r"\[shakedown\] (\S+) (start|end) ")


def rpt_errors(lines):
    """Every script error the lines show: (index of its first line, its text from the indented Error line)."""
    found = []
    for i, line in enumerate(lines):
        m = TIMED.match(line)
        if not m:
            continue
        rest = m.group(4)
        if not (rest.startswith("Error ") or "Error in expression" in rest):
            continue
        text = rest[len("Error "):] if rest.startswith("Error ") and "Error in expression" not in rest else ""
        for follow in lines[i + 1:i + 12]:
            fm = TIMED.match(follow)
            body = fm.group(4) if fm else follow
            if fm and not body.startswith(" "):
                break  # the next timed line that is not part of this error
            e = re.match(r"^\s+Error (?!position)(.*)$", body)
            if e:
                text = e.group(1).strip()
                break
        found.append((i, text))
    return found


def unmatched(rpt, errors):
    """The texts of the RPT errors that no entry of `errors` (one machine's) accounts for, each entry once."""
    left = [e["text"] for e in errors]
    missing = []
    for _, text in rpt:
        hit = next((k for k, t in enumerate(left) if t == text or (text and (text in t or t in text))), None)
        if hit is None:
            missing.append(text)
        else:
            left.pop(hit)
    return missing


def read_lines(path):
    return Path(path).read_text(encoding="utf-8", errors="replace").splitlines()


# ---------------------------------------------------------------- the runs, per repeat

@batch("ded")
def ded_batch(ctx, checks):
    needed = {i for c in checks for i in c.needs}
    ctx.m03_keep["ded"] = {"runs": ctx.session_runs, "session": ctx.session}
    ids = [i for i in DED_ORDER if i in needed]
    if ids:
        ctx.m03["ded"] = ctx.run(ids)
    for cid in ALONE:
        if cid in needed:
            ctx.m03[cid] = ctx.run([cid])
    if IDENTITY in needed:
        ctx.m03[IDENTITY] = ctx.run([IDENTITY])
        ctx.m03["logs-p1-50"] = ctx.sh("logs", "--machine", "p1", "--last", "50", ceiling=60)
        ctx.m03["logs-p1-check"] = ctx.sh("logs", "--machine", "p1", "--check", IDENTITY, ceiling=60)
    if DEMO in needed:
        ctx.m03[DEMO] = ctx.run([DEMO])
        ctx.m03["logs-demo"] = ctx.sh("logs", "--machine", "p1", "--check", DEMO, "--last", "20", ceiling=60)
    if any(c.name.startswith("US-064 LOG-03") for c in checks):
        ctx.m03["logs-server-500"] = ctx.sh("logs", "--machine", "server", "--last", "500", ceiling=60)
        ctx.m03["logs-p1-500"] = ctx.sh("logs", "--machine", "p1", "--last", "500", ceiling=60)
    if any(c.name.startswith("US-064 NFR-PV-02") for c in checks):
        ctx.m03["cmdlines"] = {m["name"]: command_line(m["pid"]) for m in ctx.session["machines"]}


def command_line(pid):
    out = subprocess.run(["powershell", "-NoProfile", "-Command",
                          f"(Get-CimInstance Win32_Process -Filter 'ProcessId={pid}').CommandLine"],
                         capture_output=True, text=True, timeout=60)
    return out.stdout.strip()


def got(ctx, key, cid=None):
    assert key in ctx.m03, f"the {key} run of this repeat did not happen; see harness.log"
    run = ctx.m03[key]
    return run if cid is None else execution(run[2], cid)


def alone(ctx, cid):
    answer, out, result = got(ctx, cid)
    assert answer.code == 1, f"{answer}, expected exit 1"
    return execution(result, cid), Path(out["result_file"]).parent


def handler_error_on_p2(ex):
    """The one error skdfx_fnc_throw raised on p2, as the handler sent it; asserts its fields."""
    found = [e for e in ex["errors"] if e["machine"] == "p2" and "_skdfx_undefined" in e["text"]]
    assert len(found) == 1, f"{ex['check_id']} lists {len(found)} errors of p2 holding _skdfx_undefined: {ex['errors']}"
    e = found[0]
    assert e["source"] == "handler", f"the error's source is {e['source']!r}, expected handler: {e}"
    assert e["step_path"] == ["trigger on p2"], f"the error's step_path is {e['step_path']}, expected ['trigger on p2']"
    assert e["check_id"] == ex["check_id"], f"the error's check_id is {e['check_id']!r}"
    assert e["file"].replace("/", "\\").endswith("skdfx\\addons\\main\\functions\\" + THROW_FILE), f"the error's file is {e['file']!r}"
    assert isinstance(e["line"], int) and e["line"] > 0, f"the error's line is {e['line']!r}"
    assert e["stack"] and any(THROW_FILE in str(f) for f in e["stack"]), f"the error's stack is {e['stack']!r}"
    at = e["at"]
    assert at.get("utc") and at.get("tick", 0) > 0 and at.get("time") is not None and at.get("frame", 0) > 0, \
        f"the error's clocks are {at}"
    assert e["expected"] is False, f"the error is marked expected: {e}"
    return e


def script_error_on_p2(ex):
    message = ended(ex, "fail", "script-error", "_skdfx_undefined", THROW_FILE)
    assert ex["reason"]["machine"] == "p2", f"reason.machine is {ex['reason']['machine']!r}, expected p2"
    assert ex["evidence"]["failed_step"] == ["trigger on p2"], f"evidence.failed_step is {ex['evidence']['failed_step']}"
    return message


# ---------------------------------------------------------------- US-024

@check("US-024 OBS-04 DM-07 a waitUntil flag reached p1 with ceiling 3 that never holds ends fail wait-ceiling naming it and 3 s, and the run exits 1",
       "session", [WAIT_CEILING], setup="ded")
def us_024_wait_ceiling(ctx):
    ex, _ = alone(ctx, WAIT_CEILING)
    message = ended(ex, "fail", "wait-ceiling", "flag reached p1")
    assert re.search(r"\b3 s\b", message), f"the message {message!r} does not name the ceiling of 3 s"
    assert 2.5 <= ex["duration_s"] <= 9, f"the check took {ex['duration_s']} s with a wait of 3 s"
    stopped_in(ex, "wait for the flag")


@check("US-024 OBS-04 RES-03 an expectEach on all that never holds ends fail assertion with every machine's last value, the failed step and a log per machine in the run folder",
       "session", [EXPECT_EACH], setup="ded")
def us_024_expect_each(ctx):
    answer, out, result = got(ctx, "ded")
    ex = execution(result, EXPECT_EACH)
    ended(ex, "fail", "assertion", "the flag reads 99")
    stopped_in(ex, "every machine sees 99")
    observed = ex["evidence"]["observed"]
    assert observed == {m: 5 for m in DED_MACHINES}, f"evidence.observed is {observed}, expected 5 on each machine"
    logs = ex["evidence"]["logs"]
    assert sorted(logs) == sorted(DED_MACHINES), f"evidence.logs names {sorted(logs)}"
    folder = Path(out["result_file"]).parent
    for machine, rel in logs.items():
        assert (folder / rel).is_file(), f"evidence.logs gives {rel} for {machine}, which is not in the run folder"


@check("US-024 NFR-PF-06 a waitUntil given no ceiling ends at 10 s and its message names the 10 s ceiling",
       "session", [DEFAULT_CEILING], setup="ded")
def us_024_default(ctx):
    ex = got(ctx, "ded", DEFAULT_CEILING)
    message = ended(ex, "fail", "wait-ceiling", "a condition that never holds")
    assert re.search(r"\b10 s\b", message), f"the message {message!r} does not name the ceiling of 10 s"
    assert 9.5 <= ex["duration_s"] <= 16, f"the check took {ex['duration_s']} s with the default ceiling"


# ---------------------------------------------------------------- US-027

@check("US-027 ERR-04 ERR-01 expectError _skdfx_undefined around skdfx_fnc_throw on p2 passes and lists the error with machine p2 and expected true",
       "session", [EXPECTED], setup="ded")
def us_027_expected(ctx):
    ex = got(ctx, "ded", EXPECTED)
    passed(ex)
    found = [e for e in ex["errors"] if e["machine"] == "p2" and "_skdfx_undefined" in e["text"]]
    assert len(found) == 1 and found[0]["expected"] is True, f"errors {ex['errors']}"


@check("US-027 ERR-04 the expected text matches case-insensitively", "session", [CASE], setup="ded")
def us_027_case(ctx):
    ex = got(ctx, "ded", CASE)
    passed(ex)
    assert any(e["machine"] == "p2" and e["expected"] is True for e in ex["errors"]), f"errors {ex['errors']}"


@check("US-027 ERR-03 an error of another text in the same expectError block still fails the check script-error",
       "session", [OTHER], setup="ded")
def us_027_other(ctx):
    ex = got(ctx, "ded", OTHER)
    ended(ex, "fail", "script-error")
    assert ex["reason"]["machine"] == "p1", f"reason.machine is {ex['reason']['machine']!r}, expected p1"
    expected = [e for e in ex["errors"] if e["machine"] == "p2" and "_skdfx_undefined" in e["text"]]
    other = [e for e in ex["errors"] if e["machine"] == "p1"]
    assert len(expected) == 1 and expected[0]["expected"] is True, f"errors {ex['errors']}"
    assert other and all(e["expected"] is False for e in other), f"errors {ex['errors']}"


# ---------------------------------------------------------------- US-028

@check("US-028 ERR-01 ERR-03 skdfx_fnc_throw on p2 in step trigger on p2 ends fail script-error naming p2, the error and fn_throw.sqf, and the run exits 1",
       "session", [ON_P2], setup="ded")
def us_028_on_p2(ctx):
    ex, _ = alone(ctx, ON_P2)
    script_error_on_p2(ex)


@check("US-028 OBS-05 RES-02 the result lists the error with machine p2, text, file, line, stack, step_path trigger on p2, source handler and the four clocks",
       "session", [ON_P2], setup="ded")
def us_028_fields(ctx):
    ex, _ = alone(ctx, ON_P2)
    handler_error_on_p2(ex)


@check("US-028 ERR-01 the same holds when skdfx_fnc_throw runs on p2 with spawn", "session", [SPAWN], setup="ded")
def us_028_spawn(ctx):
    ex = got(ctx, "ded", SPAWN)
    script_error_on_p2(ex)
    handler_error_on_p2(ex)


@check("US-028 ERR-01 the same holds when skdfx_fnc_throw reaches p2 through remoteExec from the server", "session",
       [REMOTE_EXEC], setup="ded")
def us_028_remote_exec(ctx):
    ex = got(ctx, "ded", REMOTE_EXEC)
    script_error_on_p2(ex)
    handler_error_on_p2(ex)


# ---------------------------------------------------------------- US-029

@check("US-029 ERR-04 DM-03 an expectError whose error never comes ends fail expected-error-missing quoting the text, in its step, and the run exits 1",
       "session", [NEVER], setup="ded")
def us_029(ctx):
    ex, _ = alone(ctx, NEVER)
    ended(ex, "fail", "expected-error-missing", "_skdfx_undefined")
    assert ex["evidence"]["failed_step"] == ["expect an error that never comes"], \
        f"evidence.failed_step is {ex['evidence']['failed_step']}"
    assert [s["path"] for s in ex["steps"]] == [["expect an error that never comes"], ["the rest of the check passes"]], \
        f"the steps that ran are {[s['path'] for s in ex['steps']]}"


# ---------------------------------------------------------------- US-030

@check("US-030 ERR-02 INT-AR-07 an error line in p1's RPT that the handler never saw ends fail script-error and is listed with machine p1 and source rpt",
       "session", [RPT_ONLY], setup="ded")
def us_030_rpt(ctx):
    ex = got(ctx, "ded", RPT_ONLY)
    ended(ex, "fail", "script-error")
    found = [e for e in ex["errors"] if "_acc_m03_rpt_only" in e["text"]]
    assert len(found) == 1, f"the result lists {len(found)} errors holding _acc_m03_rpt_only: {ex['errors']}"
    e = found[0]
    assert e["machine"] == "p1" and e["source"] == "rpt", f"the error is {e}"
    want = "acc_m03" + chr(92) + "fn_rptOnly.sqf"
    assert (e["file"], e["line"]) == (want, 7), f"the error gives file {e['file']!r} and line {e['line']!r}, expected {want!r} and 7"
    assert e["at"]["utc"] and e["at"]["tick"] == 0, f"the error's clocks are {e['at']}"
    assert ex["reason"]["machine"] == "p1", f"reason.machine is {ex['reason']['machine']!r}, expected p1"


def all_executions(ctx):
    """(run folder, execution) for every check execution of every run the suite made so far."""
    for answer, out, result, new in ctx.runs:
        folder = Path(out["result_file"]).parent
        for ex in result["checks"]:
            yield folder, ex


@check("US-030 NFR-RL-05 every script error line in each machine's log excerpt of every check execution of the suite's runs is in that check's errors",
       "end")
def us_030_excerpts(ctx):
    assert ctx.runs, "the suite made no run"
    missing, counted = [], 0
    for folder, ex in all_executions(ctx):
        for machine, rel in (ex["evidence"].get("logs") or {}).items():
            path = folder / rel
            if not path.is_file():
                missing.append(f"{folder.name} {ex['check_id']}: {rel} is not in the run folder")
                continue
            rpt = rpt_errors(read_lines(path))
            counted += len(rpt)
            for text in unmatched(rpt, [e for e in ex["errors"] if e["machine"] == machine]):
                missing.append(f"{folder.name} {ex['check_id']} #{ex['repeat']} on {machine}: {text!r}")
    ctx.log(f"US-030: {counted} script error lines in the excerpts of {sum(1 for _ in all_executions(ctx))} executions")
    assert not missing, f"{len(missing)} error lines of an excerpt are not in their check's errors: {missing[:10]}"


# ---------------------------------------------------------------- US-064

@check("US-064 NFR-PV-02 DM-11 every game process of the dedicated session has -name=<machine> and name player reads p1 on p1 and p2 on p2",
       "session", [NAMES], setup="ded")
def us_064_names(ctx):
    passed(got(ctx, "ded", NAMES))
    lines = got(ctx, "cmdlines")
    for machine in DED_MACHINES:
        assert re.search(rf'-name={machine}(?=["\s]|$)', lines[machine]), \
            f"{machine}'s process has no -name={machine} on its command line"


@check("US-064 LOG-03 LOG-02 logs --machine server --last 500 and logs --machine p1 --last 500 print lines and neither the Steam ID nor a Steam name",
       "session", [EXPECT_EACH], setup="ded")
def us_064_logs(ctx):
    ident = identity()
    for key in ("logs-server-500", "logs-p1-500"):
        answer = got(ctx, key)
        assert answer.code == 0, f"{answer}; stderr {answer.stderr[:300]!r}"
        lines = answer.stdout.splitlines()
        assert len(lines) >= 50, f"{answer} printed {len(lines)} lines"
        assert len(lines) <= 501, f"{answer} printed {len(lines)} lines with --last 500"
        found = leaks(answer.stdout + answer.stderr, ident)
        assert not found, f"{answer} prints {found}"


@check("US-064 NFR-PV-01 NFR-PV-02 LOG-02 after session stop no result file, run log, check log or stopped session's RPT holds the Steam ID or a Steam name",
       "end")
def us_064_files(ctx):
    assert "ded" in ctx.sessions and "ded" in ctx.stops, "no dedicated session ran and stopped; run a ded check with this one"
    answer, _ = ctx.stops["ded"]
    assert answer.code == 0, f"{answer}"
    ident = identity()
    runs = ctx.project / ".shakedown" / "runs"
    run_files = [p for p in runs.rglob("*") if p.is_file() and "dumps" not in p.relative_to(runs).parts]
    assert any(p.name == "result.json" for p in run_files), "no result file under the project's runs"
    rpts, stored = [], []
    for setup, session in ctx.sessions.items():
        for m in session["machines"]:
            found = sorted(Path(m["profile"]).glob("*.rpt"))
            assert found, f"{m['name']} of the {setup} session left no RPT in {m['profile']}"
            rpts += found
        # What the framework stores in the session folder outside the profile folders, such as
        # the files logs names in its path.
        folder = Path(session["machines"][0]["profile"]).parent.parent
        stored += [p for p in folder.rglob("*") if p.is_file() and "machines" not in p.relative_to(folder).parts]
    runner_log = Path(os.environ.get("LOCALAPPDATA", "")) / "Shakedown" / "runner.log"
    if runner_log.is_file():
        stored.append(runner_log)
    hits = search_files(run_files + rpts + stored, ident)
    ctx.log(f"US-064: searched {len(run_files)} run folder files, {len(rpts)} RPTs and {len(stored)} other stored files")
    assert not hits, f"{len(hits)} files hold the Steam identity: {hits[:10]}"


# ---------------------------------------------------------------- US-065

@check("US-065 LOG-02 a check attaching and failing with the Steam ID and name ends fail explicit with <steamid> and <steamname> in the result and the run's output",
       "session", [IDENTITY], setup="ded")
def us_065_result(ctx):
    answer, out, result = got(ctx, IDENTITY)
    ex = execution(result, IDENTITY)
    ended(ex, "fail", "explicit")
    assert ex["reason"]["message"] == "uid <steamid> name <steamname>", f"the message is {len(ex['reason']['message'])} " \
        f"characters and not 'uid <steamid> name <steamname>'; it holds {leaks(ex['reason']['message'], identity())}"
    attached = {a["label"]: a["value"] for a in ex["evidence"]["attached"]}
    assert attached.get("uid") == "<steamid>" and attached.get("name") == "<steamname>", \
        f"the attached values are not the placeholders; they hold {leaks(str(attached), identity())}"
    ident = identity()
    found = leaks(answer.stdout + answer.stderr + Path(out["result_file"]).read_text(encoding="utf-8"), ident)
    assert not found, f"the run's output or result file holds {found}"


@check("US-065 LOG-02 LOG-03 the diag_log of the Steam ID and name reads uid <steamid> name <steamname> in the check's p1 log and in logs --machine p1 --last 50",
       "session", [IDENTITY], setup="ded")
def us_065_logs(ctx):
    answer, out, result = got(ctx, IDENTITY)
    ex = execution(result, IDENTITY)
    ident = identity()
    excerpt = (Path(out["result_file"]).parent / ex["evidence"]["logs"]["p1"]).read_text(encoding="utf-8", errors="replace")
    want = "uid <steamid> name <steamname>"
    assert want in excerpt, f"p1's log of the check has no line {want!r}; it holds {leaks(excerpt, ident)}"
    assert not leaks(excerpt, ident), f"p1's log of the check holds {leaks(excerpt, ident)}"
    for key in ("logs-p1-50", "logs-p1-check"):
        logs = got(ctx, key)
        assert logs.code == 0, f"{logs}; stderr {logs.stderr[:300]!r}"
        assert want in logs.stdout, f"{logs} prints no line {want!r}; it holds {leaks(logs.stdout, ident)}"
        assert not leaks(logs.stdout + logs.stderr, ident), f"{logs} prints {leaks(logs.stdout, ident)}"


# ---------------------------------------------------------------- the demo

@check("M3 demo US-024 the M2 rope check with its last assertion changed fails with its step, each machine's last value, a log per machine and any script error with its machine and step, and no Steam identity",
       "session", [DEMO], setup="ded")
def m3_demo(ctx):
    ex, folder = alone(ctx, DEMO)
    ended(ex, "fail", "assertion", "the rope's holder is p2")
    stopped_in(ex, "the server and both clients see p2 holding the rope")
    observed = ex["evidence"]["observed"]
    assert observed == {"server": "nobody", "p1": "nobody", "p2": "nobody"}, f"evidence.observed is {observed}"
    logs = ex["evidence"]["logs"]
    for machine in ["server", "p1", "p2"]:
        assert machine in logs and (folder / logs[machine]).is_file(), f"evidence.logs {logs} gives no file for {machine}"
    for e in ex["errors"]:
        assert e["machine"] and e["step_path"], f"an error has no machine or step: {e}"
    ctx.log(f"M3 demo: {len(ex['errors'])} script errors listed: {[(e['machine'], e['text'][:60]) for e in ex['errors']]}")
    logs_answer = got(ctx, "logs-demo")
    assert logs_answer.code == 0 and logs_answer.stdout.strip(), f"{logs_answer} printed nothing"
    ident = identity()
    hits = search_files([p for p in folder.rglob("*") if p.is_file()], ident)
    assert not hits, f"the demo's run folder holds the Steam identity: {hits}"
    assert not leaks(logs_answer.stdout, ident), "logs of the demo prints the Steam identity"


# ---------------------------------------------------------------- after the sessions stop

@check("DEF-09 RST-02 the server of the dedicated session logged no Object not found line over the session's resets", "end")
def def_09(ctx):
    assert "ded" in ctx.sessions, "no dedicated session ran; run a ded check with this one"
    server = [m for m in ctx.sessions["ded"]["machines"] if m["name"] == "server"][0]
    lines = [l for p in sorted(Path(server["profile"]).glob("*.rpt")) for l in read_lines(p)]
    # DEF-09 promises no such line for the objects a check creates. The body a killed p2 leaves is DEF-25,
    # deferred: the checks that kill p2 log its net id, and a line naming one of those ids is not DEF-09's.
    bodies = {m.group(1) for l in lines for m in [re.search(r"\[acc\] DEF-25 body (\d+:\d+)", l)] if m}
    found = [l for l in lines if re.search(r"Object .* not found", l)]
    body = [l for l in found if (m := re.search(r"Object (\d+:\d+) not found", l)) and m.group(1) in bodies]
    bad = [l for l in found if l not in body]
    resets = sum(1 for l in lines if MARKER.search(l) and "#" in l and " start " in l)
    ctx.log(f"DEF-09: {resets} check executions in the server's RPT, {len(bodies)} bodies of a killed p2, "
            f"{len(body)} Object not found lines naming one of those bodies (DEF-25): {body}, {len(bad)} others")
    assert resets, "the dedicated session ran no check, so no reset was watched"
    assert not bad, f"the server's RPT holds {len(bad)} Object not found lines, such as {bad[0]!r}"


@check("NFR-RL-05 M3 exit 2 every script error a stopped session's RPT shows inside a check execution is in that execution's errors",
       "end")
def nfr_rl_05(ctx):
    by_key = {}
    for answer, out, result, new in ctx.runs:
        for n, ex in enumerate(result["checks"], 1):
            by_key[f"{result['run']['id']}#{n}"] = ex
    missing, inside, outside = [], 0, []
    for setup, session in ctx.sessions.items():
        for m in session["machines"]:
            for rpt in sorted(Path(m["profile"]).glob("*.rpt")):
                lines = read_lines(rpt)
                # The check execution each line falls in, from the markers.
                window, key_at = None, []
                for line in lines:
                    mk = MARKER.search(line)
                    if mk and "#" in mk.group(1):
                        window = mk.group(1) if mk.group(2) == "start" else None
                    key_at.append(window)
                errors = rpt_errors(lines)
                grouped = {}
                for i, text in errors:
                    if key_at[i] is None:
                        outside.append((setup, m["name"], text))
                    else:
                        grouped.setdefault(key_at[i], []).append((i, text))
                for key, rpt_list in grouped.items():
                    inside += len(rpt_list)
                    ex = by_key.get(key)
                    if ex is None:
                        missing.append(f"{setup} {m['name']} {key}: no result file of the suite holds this execution")
                        continue
                    for text in unmatched(rpt_list, [e for e in ex["errors"] if e["machine"] == m["name"]]):
                        missing.append(f"{setup} {m['name']} {key} {ex['check_id']}: {text!r}")
    ctx.m03_keep["rl05"] = {"inside": inside, "outside": outside, "missing": missing}
    ctx.log(f"NFR-RL-05: {inside} script errors inside check executions, {len(missing)} not in a result file; "
            f"{len(outside)} outside any check execution: {outside[:10]}")
    assert not missing, f"{len(missing)} script errors of the RPTs are in no result file: {missing[:10]}"
