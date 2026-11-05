"""The acceptance checks of M0 Walking skeleton: US-037, US-038 and the requirements of the demo."""

import time
from datetime import datetime
from pathlib import Path

from . import driver
from .core import batch, check, execution

# SQF check ids, as `list` gives them (their files are in acceptance/checks/).
STEPS_ONLY = "us037/US-019-steps-only"
STEP_RECORDS = "us037/US-037-step-records"
EVIDENCE_PASS = "us037/US-037-evidence-on-pass"
EVIDENCE_FAIL = "us037/US-037-evidence-on-fail"
NO_CHECK = "us038/US-038-no-check-line"
TIMEOUT_FAST = "us038/US-038-timeout-fast"
KEY_TWICE = "us038/US-038-key-twice"
SYNTAX = "us038/US-038-syntax-error"
API_IN_SENT = "us038/US-038-api-in-sent-code"
WRONG_ARG = "us038/US-038-wrong-argument"
NEXT_PASSES = "us038/US-038-next-check-passes"

# The batch runs the selected ids in this order, so every broken check comes before NEXT_PASSES.
ORDER = [STEPS_ONLY, STEP_RECORDS, EVIDENCE_PASS, EVIDENCE_FAIL, NO_CHECK, TIMEOUT_FAST, KEY_TWICE,
         SYNTAX, API_IN_SENT, WRONG_ARG, NEXT_PASSES]
BROKEN = [NO_CHECK, TIMEOUT_FAST, KEY_TWICE, SYNTAX, API_IN_SENT, WRONG_ARG]

SYNTAX_LINE = 5
SYNTAX_BROKEN = "private _value = (1 + ;"
SYNTAX_FIXED = "private _value = (1 + 1);"

FIX_CHECK = "US-038 a fixed syntax error passes on the next run with no reload and the same start number"


def utc(text):
    return datetime.strptime(text, "%Y-%m-%dT%H:%M:%S.%fZ")


def listed(ctx):
    answer = ctx.sh("list", "--json", ceiling=60)
    assert answer.code == 0, f"{answer}; stderr {answer.stderr[:300]!r}"
    return {c["id"]: c for c in answer.json()["checks"]}


def check_error(ex, *parts):
    """The execution ended fail / check-error and its message holds every part."""
    assert ex["outcome"] == "fail", f"outcome {ex['outcome']}, expected fail (never invalid): {ex['reason']}"
    assert ex["reason"]["kind"] == "check-error", f"reason {ex['reason']}, expected kind check-error"
    message = ex["reason"]["message"]
    for part in parts:
        assert part in message, f"the message {message!r} does not name {part!r}"
    return message


def set_syntax(project, fixed):
    path = project / "checks" / (SYNTAX + ".sqf")
    lines = path.read_text(encoding="utf-8").split("\n")
    lines[SYNTAX_LINE - 1] = SYNTAX_FIXED if fixed else SYNTAX_BROKEN
    path.write_text("\n".join(lines), encoding="utf-8")


@batch("sp")
def m00_batch(ctx, checks):
    """One run of every M0 check the selection needs, then the rerun of the fixed syntax error."""
    needed = {i for c in checks for i in c.needs}
    ids = [i for i in ORDER if i in needed]
    set_syntax(ctx.project, fixed=False)
    ctx.batch, ctx.fixed = None, None
    if ids:
        ctx.batch = ctx.run(ids)
    if any(c.name == FIX_CHECK for c in checks):
        set_syntax(ctx.project, fixed=True)
        ctx.fixed = ctx.run([SYNTAX])
        set_syntax(ctx.project, fixed=False)


# ---------------------------------------------------------------- offline, no game

@check("RUN-02 list gives every check's id, name, tags, setups, timeout and reset without a runner or a game", "offline")
def run_02(ctx):
    before = driver.runner_pid()
    games = driver.pids("arma3_x64.exe")
    checks = listed(ctx)
    files = ctx.project_ids()
    assert sorted(checks) == files, f"list gave {sorted(checks)}, the checks folder holds {files}"
    for c in checks.values():
        assert set(c) == {"id", "name", "tags", "setups", "timeout_s", "reset", "error"}, f"{c['id']}: keys {sorted(c)}"
    good = checks[STEP_RECORDS]
    assert good["tags"] == ["US-037", "acceptance"] and good["setups"] is None \
        and good["timeout_s"] == 120 and good["reset"] == "in-place" and good["error"] is None, f"{good}"
    assert driver.runner_pid() == before, "list started a runner"
    assert driver.pids("arma3_x64.exe") == games, "list started a game"


@check("US-037 list shows the steps-only check with its name starting with US-019", "offline")
def us_037_list(ctx):
    c = listed(ctx)[STEPS_ONLY]
    assert c["error"] is None, f"the steps-only check is listed as broken: {c['error']}"
    assert c["name"].startswith("US-019"), f"name {c['name']!r} does not start with US-019"
    text = ctx.sh("list", STEPS_ONLY, ceiling=60)
    assert text.code == 0 and "US-019" in text.stdout, f"{text}; stdout {text.stdout!r}"


@check("US-038 list shows each broken header with the line and the key", "offline")
def us_038_list(ctx):
    checks = listed(ctx)
    expect = {NO_CHECK: ["@check"], TIMEOUT_FAST: ["line 2", "@timeout"], KEY_TWICE: ["line 3", "@tags"]}
    for cid, parts in expect.items():
        err = checks[cid]["error"]
        assert err, f"{cid} is listed with no error"
        for part in parts:
            assert part in err, f"{cid}: the error {err!r} does not name {part!r}"


# ---------------------------------------------------------------- in the session, per repeat

@check("US-037 a check of only a header and steps passes", "session", [STEPS_ONLY])
def us_037_steps_only(ctx):
    ex = execution(ctx.batch[2], STEPS_ONLY)
    assert ex["outcome"] == "pass", f"outcome {ex['outcome']}: {ex['reason']}"
    assert ex["name"].startswith("US-019"), f"name {ex['name']!r}"
    assert [s["path"] for s in ex["steps"]] == [["add two numbers"], ["join two strings"]], f"steps {ex['steps']}"


@check("US-037 steps are recorded in start order with path, machine, state and clocks", "session", [STEP_RECORDS])
def us_037_step_records(ctx):
    ex = execution(ctx.batch[2], STEP_RECORDS)
    assert ex["outcome"] == "pass", f"outcome {ex['outcome']}: {ex['reason']}"
    steps = ex["steps"]
    paths = [s["path"] for s in steps]
    expected = [["outer"], ["outer", "inner one"], ["outer", "inner two"], ["outer", "inner two", "deepest"], ["last"]]
    assert paths == expected, f"step paths {paths}, expected {expected}"
    for s in steps:
        name = "/".join(s["path"])
        assert s["machine"] == "p1", f"{name}: machine {s['machine']}"
        assert s["state"] == "passed", f"{name}: state {s['state']}"
        assert s["error"] is None, f"{name}: error {s['error']}"
        a, b = s["start"], s["end"]
        assert b is not None, f"{name}: no end"
        for clock in ("tick", "time", "frame"):
            assert b[clock] >= a[clock], f"{name}: {clock} ends at {b[clock]} before it starts at {a[clock]}"
        span = (utc(b["utc"]) - utc(a["utc"])).total_seconds()
        assert span >= 0, f"{name}: utc ends before it starts"
        assert abs(s["duration_s"] - span) <= 0.05 or abs(s["duration_s"] - (b["tick"] - a["tick"])) <= 0.05, \
            f"{name}: duration {s['duration_s']} matches neither the utc span {span} nor the tick span"
    # Start order on every clock.
    for prev, cur in zip(steps, steps[1:]):
        for clock in ("tick", "time", "frame"):
            assert cur["start"][clock] >= prev["start"][clock], f"{cur['path']} starts before {prev['path']} on {clock}"
    # Each nested step lies inside its parent.
    by_path = {tuple(s["path"]): s for s in steps}
    for s in steps:
        if len(s["path"]) > 1:
            parent = by_path[tuple(s["path"][:-1])]
            for clock in ("tick", "frame"):
                assert parent["start"][clock] <= s["start"][clock] and s["end"][clock] <= parent["end"][clock], \
                    f"{s['path']} lies outside its parent on {clock}"
    # The waits: inner one and deepest each span at least 3 frames, outer at least 10.
    for path, frames in ((("outer", "inner one"), 3), (("outer", "inner two", "deepest"), 3), (("outer",), 10)):
        s = by_path[path]
        assert s["end"]["frame"] - s["start"]["frame"] >= frames, \
            f"{path}: spans {s['end']['frame'] - s['start']['frame']} frames, the step waited for {frames}"
    # The clocks read inside inner one lie within its recorded start and end.
    attached = {e["label"]: e["value"] for e in ex["evidence"]["attached"]}
    tick, mtime, frame = attached["clocks inside inner one"]
    inner = by_path[("outer", "inner one")]
    assert inner["start"]["tick"] <= tick <= inner["end"]["tick"] + 0.001, f"tick {tick} outside {inner['start']}..{inner['end']}"
    assert inner["start"]["time"] <= mtime <= inner["end"]["time"] + 0.001, f"time {mtime} outside {inner['start']}..{inner['end']}"
    assert inner["start"]["frame"] <= frame <= inner["end"]["frame"], f"frame {frame} outside {inner['start']}..{inner['end']}"


@check("US-037 evidence is kept for a passing check", "session", [EVIDENCE_PASS])
def us_037_evidence_pass(ctx):
    ex = execution(ctx.batch[2], EVIDENCE_PASS)
    assert ex["outcome"] == "pass", f"outcome {ex['outcome']}: {ex['reason']}"
    expected = [{"label": "number", "value": 42}, {"label": "text", "value": "hello"},
                {"label": "list", "value": [1, "two", [3]]}, {"label": "flag", "value": True}]
    assert ex["evidence"]["attached"] == expected, f"attached {ex['evidence']['attached']}"


@check("US-037 evidence is kept for a failing check", "session", [EVIDENCE_FAIL])
def us_037_evidence_fail(ctx):
    ex = execution(ctx.batch[2], EVIDENCE_FAIL)
    assert ex["outcome"] == "fail" and ex["reason"]["kind"] == "explicit", f"{ex['outcome']} {ex['reason']}"
    assert "deliberate failure for US-037" in ex["reason"]["message"], f"{ex['reason']}"
    expected = [{"label": "number", "value": 7}, {"label": "text", "value": "kept on failure"}]
    assert ex["evidence"]["attached"] == expected, f"attached {ex['evidence']['attached']}"
    assert ex["evidence"]["failed_step"] == ["fail on purpose"], f"failed_step {ex['evidence']['failed_step']}"
    assert [s["path"] for s in ex["steps"]] == [["attach two values"], ["fail on purpose"]], \
        f"steps {[s['path'] for s in ex['steps']]}: the step after skd_fnc_fail must not run"
    assert ex["steps"][1]["state"] == "failed", f"the failing step has state {ex['steps'][1]['state']}"


def header_error(ctx, cid, *parts):
    ex = execution(ctx.batch[2], cid)
    check_error(ex, *parts)
    assert ex["steps"] == [], f"a header error ran steps: {ex['steps']}"
    assert "machine" not in ex["reason"], f"a header error names machine {ex['reason'].get('machine')}"


@check("US-038 a check with no @check line ends fail check-error without touching the game", "session", [NO_CHECK])
def us_038_no_check(ctx):
    header_error(ctx, NO_CHECK, "@check")


@check("US-038 a check with @timeout fast ends fail check-error naming the line and the key", "session", [TIMEOUT_FAST])
def us_038_timeout_fast(ctx):
    header_error(ctx, TIMEOUT_FAST, "line 2", "@timeout")


@check("US-038 a check with a key given twice ends fail check-error naming the line and the key", "session", [KEY_TWICE])
def us_038_key_twice(ctx):
    header_error(ctx, KEY_TWICE, "line 3", "@tags")


@check("US-038 a body with a syntax error ends fail check-error with the compile message and its line", "session", [SYNTAX])
def us_038_syntax(ctx):
    ex = execution(ctx.batch[2], SYNTAX)
    check_error(ex, f"line {SYNTAX_LINE}")
    assert ex["steps"] == [], f"a body that does not compile ran steps: {ex['steps']}"


@check("US-038 skd_fnc_expect in code sent to a machine ends fail check-error", "session", [API_IN_SENT])
def us_038_api_in_sent(ctx):
    ex = execution(ctx.batch[2], API_IN_SENT)
    message = check_error(ex, "skd_fnc_")
    assert "body" in message, f"the message {message!r} does not say skd_fnc_* runs only in the body"


@check("US-038 a check API call with a wrong argument ends fail check-error naming the function", "session", [WRONG_ARG])
def us_038_wrong_argument(ctx):
    ex = execution(ctx.batch[2], WRONG_ARG)
    check_error(ex, "skd_fnc_step")


@check("US-038 the check after broken ones runs and passes and the session stays ready", "session", BROKEN + [NEXT_PASSES])
def us_038_next_passes(ctx):
    answer, out, result = ctx.batch
    ex = execution(result, NEXT_PASSES)
    assert ex["outcome"] == "pass", f"outcome {ex['outcome']}: {ex['reason']}"
    ran = [c["check_id"] for c in result["checks"]]
    assert ran.index(NEXT_PASSES) == len(ran) - 1, f"the passing check did not run last: {ran}"
    for m in result["session"]["machines"]:
        assert m["end_state"] == "ready", f"{m['name']} ended the run {m['end_state']}"
    assert result["session"]["id"] == ctx.session["id"], "the run used another session"
    assert all(c["outcome"] != "invalid" for c in result["checks"]), "a broken check ended invalid"


@check(FIX_CHECK, "session", [SYNTAX])
def us_038_fixed(ctx):
    answer, out, result = ctx.fixed
    assert answer.code == 0, f"{answer}, expected exit 0 for a passing run"
    ex = execution(result, SYNTAX)
    assert ex["outcome"] == "pass", f"the fixed check ended {ex['outcome']}: {ex['reason']}"
    assert result["session"]["id"] == ctx.session["id"], "the rerun used another session"
    starts = {m["name"]: m["start"] for m in ctx.session["machines"]}
    for m in result["session"]["machines"]:
        assert m["start"] == starts[m["name"]], f"{m['name']} has start {m['start']}, the session started it as {starts[m['name']]}"


@check("RES-01 RES-02 RES-04 every result file is valid and holds the run, session, selection and summary", "session", [STEPS_ONLY])
def res_01(ctx):
    runs = [ctx.batch] + ([ctx.fixed] if ctx.fixed else [])
    for answer, out, result in runs:
        errors = ctx.validate(result)
        assert not errors, f"{out['result_file']} is not valid against result.schema.json: {errors[:5]}"
        folder = Path(out["result_file"]).parent
        assert folder.parent.resolve() == (ctx.project / ".shakedown" / "runs").resolve(), f"the result file is in {folder}"
        assert folder.name == result["run"]["id"] == out["run_id"], f"run id {result['run']['id']} in folder {folder.name}"
        new = next(r[3] for r in ctx.runs if r[2] is result)
        assert new == [folder.name], f"the run made run folders {new}, expected exactly one"
        s = result["summary"]
        assert s["total"] == s["pass"] + s["fail"] + s["invalid"] + s["not_run"] == len(result["checks"]) + len(result["not_run"]), f"summary {s}"
        assert result["session"]["id"] == ctx.session["id"] and result["session"]["setup"]["name"] == "sp", f"{result['session']}"
        assert result["session"]["setup"]["mode"] == "singleplayer", f"{result['session']['setup']}"
        names = [m["name"] for m in result["session"]["machines"]]
        assert names == ["p1"], f"machines {names}"
        assert result["session"]["machines"][0]["game_version"], "p1 has no game version"
        want_code = 1 if s["fail"] else 2 if s["invalid"] else 0
        assert answer.code == want_code, f"{answer}, expected exit {want_code} for summary {s}"
        assert result["run"]["selection"]["ids"] == [c["check_id"] for c in result["checks"]], \
            f"selection {result['run']['selection']}"


@check("CFG-02 the project gains only .shakedown with a .gitignore that ignores everything", "session", [STEPS_ONLY])
def cfg_02(ctx):
    entries = sorted(p.name for p in ctx.project.iterdir())
    assert entries == [".shakedown", "checks", "shakedown.toml"], f"the project folder holds {entries}"
    ignore = (ctx.project / ".shakedown" / ".gitignore").read_text(encoding="utf-8").split()
    assert "*" in ignore, f".shakedown/.gitignore holds {ignore}, which does not ignore everything"


# ---------------------------------------------------------------- after the session

@check("SES-07 session stop returns with no game process of the session left", "end")
def ses_07(ctx):
    answer = ctx.stop_answer
    assert answer is not None and answer.code == 0, f"{answer}"
    stopped = answer.json()
    assert stopped["state"] == "stopped", f"session state {stopped['state']}"
    assert not ctx.alive_after_stop,         f"game processes {sorted(ctx.alive_after_stop)} of the session still ran when session stop returned"


@check("P-01 the runner exits by itself within 70 s of the session stopping", "end")
def p_01(ctx):
    pid = ctx.runner_pid
    assert pid, "no runner pid was read from runner.json during the session"
    left = max(1, 75 - (time.monotonic() - ctx.stopped_at))
    driver.wait_gone(pid, left)
    took = time.monotonic() - ctx.stopped_at
    assert not driver.alive(pid), f"the runner {pid} still runs {took:.0f} s after the session stopped"
    ctx.log(f"the runner exited within {took:.1f} s of the session stopping")
    assert took <= 70, f"the runner exited {took:.0f} s after the session stopped"
