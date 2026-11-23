"""The acceptance checks of M1 A grappling-hook check in singleplayer: US-018, US-021, US-022,
US-039, the requirements of the check API, the reset and the selection, and exit criteria 2 and 3."""

import json
import re

from . import driver
from .core import batch, check, execution

# SQF check ids, as `list` gives them (their files are in acceptance/checks/).
UNKNOWN_CLASS = "us018/US-018-unknown-class"
PLACE_POSITION = "us018/US-018-place-position"
CEILING_NEGATIVE = "us018/US-018-ceiling-negative"
CEILING_STRING = "us018/US-018-ceiling-string"
US018_NEXT = "us018/US-018-next-check-passes"
SLOW_PART = "us039/US-039-slow-part"
CEILING_CUT = "us039/US-039-ceiling-cut"
NO_CBA = "us022/US-022-no-cba"
NO_KEY = "us022/US-022-no-key-passes"
MISSPELLED = "us022/US-022-misspelled-keybind"
UNKNOWN_MOD = "us022/US-022-unknown-mod"
PRESS = "us021/US-021-press"
DOWN_UP = "us021/US-021-down-then-up"
CONTEXT = "m01/API-03-context"
VALUES = "m01/CTL-01-values"
ON_EACH = "m01/CTL-01-on-each"
WAITS = "m01/OBS-01-waits"
WAIT_CEILING = "m01/OBS-01-wait-ceiling"
NOT_BOOLEAN = "m01/OBS-01-not-boolean"
EXPECT = "m01/OBS-06-expect"
EACH_FAILS = "m01/OBS-06-expect-each-fails"
ON_FAILS = "m01/OBS-06-expect-on-fails"
CREATE = "m01/SCN-01-create"
PLACE = "m01/SCN-02-place"
DATE_WEATHER = "m01/SCN-03-date-weather"
INVALID = "m01/API-03-invalid"
CLEANUP_REGISTER = "m01/RST-01-cleanup-register"
CLEANUP_RAN = "m01/RST-01-cleanup-ran"
KILL = "m01/RST-05-kill-player"
ALIVE_AFTER = "m01/RST-05-alive-after"
RL = [f"m01/NFR-RL-02-start-state-{n:02}" for n in range(1, 21)]
DEMO_KEY = "agh/AGH-US-009-let-go-key"
DEMO_ACTION = "agh/AGH-US-009-let-go-action"
SEL_A, SEL_B, SEL_C = "sel/RUN-01-a", "sel/RUN-01-b", "sel/RUN-01-c"

# The sp batch runs in this order. RL[0] comes first so the start state is recorded before
# ALIVE_AFTER compares with it; each broken check of US-018 comes before US018_NEXT; the pairs
# CLEANUP_REGISTER, CLEANUP_RAN and KILL, ALIVE_AFTER run back to back.
SP_ORDER = [RL[0], UNKNOWN_CLASS, PLACE_POSITION, CEILING_NEGATIVE, CEILING_STRING, US018_NEXT,
            SLOW_PART, CEILING_CUT, NO_CBA, NO_KEY, CONTEXT, VALUES, ON_EACH, WAITS, WAIT_CEILING,
            NOT_BOOLEAN, EXPECT, EACH_FAILS, ON_FAILS, CREATE, PLACE, DATE_WEATHER, INVALID,
            CLEANUP_REGISTER, CLEANUP_RAN, KILL, ALIVE_AFTER]
CBA_ORDER = [PRESS, DOWN_UP, MISSPELLED, UNKNOWN_MOD]
US018_BROKEN = [UNKNOWN_CLASS, PLACE_POSITION, CEILING_NEGATIVE, CEILING_STRING]
DEMO_RUNS = 10

# The selection runs of RUN-01 and RUN-03: (name, ids, tags).
SELECTIONS = [
    ("ids", [SEL_C, SEL_A], []),
    ("tag", [], ["acc-sel-x"]),
    ("tags", [], ["acc-sel-x", "acc-sel-y"]),
    ("ids and tags", [SEL_B], ["acc-sel-x"]),
]

# The runs of finding m01-3: an id given more than once (RUN-01, RUN-03, US-031 criterion 2).
REPEATS = [
    ("a a b", [SEL_A, SEL_A, SEL_B], []),
    ("a b a", [SEL_A, SEL_B, SEL_A], []),
    ("a a a", [SEL_A, SEL_A, SEL_A], []),
    ("a a and a tag", [SEL_A, SEL_A], ["acc-sel-x"]),
]


# ---------------------------------------------------------------- the runs, per repeat

@batch("sp")
def sp_batch(ctx, checks):
    needed = {i for c in checks for i in c.needs}
    ids = [i for i in SP_ORDER if i in needed]
    if ids:
        ctx.m01["sp"] = ctx.run(ids)
    if any(i in needed for i in RL[1:]):
        ctx.m01["rl"] = ctx.run(RL)
    if any(i in needed for i in (SEL_A, SEL_B, SEL_C)):
        ctx.m01["sel"] = {name: ctx.run(ids, tags=tags) for name, ids, tags in SELECTIONS}
    if any(c.name.startswith("US-031") for c in checks):
        ctx.m01["rep"] = {name: ctx.run(ids, tags=tags) for name, ids, tags in REPEATS}
        ctx.m01["rep text"] = run_text(ctx, [SEL_A, SEL_A, SEL_B])


def run_text(ctx, ids):
    """`shakedown run` without --json, as a person runs it; returns the answer and the result file."""
    before = set(ctx.sh.run_folders())
    answer = ctx.sh("run", *ids, ceiling=600)
    new = sorted(set(ctx.sh.run_folders()) - before)
    assert len(new) == 1, f"{answer} made the run folders {new}, expected one"
    path = ctx.project / ".shakedown" / "runs" / new[0] / "result.json"
    result = json.loads(path.read_text(encoding="utf-8"))
    out = {"run_id": new[0], "result_file": str(path)}
    ctx.runs.append((answer, out, result, new))
    ctx.session_runs.append((answer, out, result, new))
    return answer, out, result


@batch("cba")
def cba_batch(ctx, checks):
    needed = {i for c in checks for i in c.needs}
    ids = [i for i in CBA_ORDER if i in needed]
    if ids:
        ctx.m01["cba"] = ctx.run(ids)
    if DEMO_KEY in needed:
        ctx.m01["demo"] = ctx.run([DEMO_KEY, DEMO_ACTION] * DEMO_RUNS)


def ran(ctx, key, cid):
    assert key in ctx.m01, f"the {key} run of this repeat did not happen; see harness.log"
    return execution(ctx.m01[key][2], cid)


def passed(ex):
    assert ex["outcome"] == "pass", f"{ex['check_id']} ended {ex['outcome']}: {ex['reason']}"


def ended(ex, outcome, kind, *parts):
    """The execution ended outcome / kind, and its message holds every part."""
    reason = ex["reason"] or {}
    assert ex["outcome"] == outcome and reason.get("kind") == kind, \
        f"{ex['check_id']} ended {ex['outcome']} / {reason}, expected {outcome} / {kind}"
    for part in parts:
        assert part in reason["message"], f"the message {reason['message']!r} does not name {part!r}"
    return reason["message"]


def stopped_in(ex, *path):
    """The outcome was decided in the step at path, and no step after it ran."""
    assert ex["evidence"]["failed_step"] == list(path), \
        f"evidence.failed_step is {ex['evidence']['failed_step']}, expected {list(path)}"
    ran_steps = [s["path"] for s in ex["steps"]]
    assert ["never runs"] not in ran_steps, f"a step after the failure ran: {ran_steps}"


# ---------------------------------------------------------------- US-018

@check("US-018 an unknown class to skd_fnc_create ends fail check-error naming the class and CfgVehicles", "session",
       [UNKNOWN_CLASS])
def us_018_unknown_class(ctx):
    ex = ran(ctx, "sp", UNKNOWN_CLASS)
    ended(ex, "fail", "check-error", "B_Soldier_Typo", "CfgVehicles")
    stopped_in(ex, "build the scene", "create a soldier with a typo in its class")


@check("US-018 a string position to skd_fnc_place ends fail check-error naming the function, position and the type it takes",
       "session", [PLACE_POSITION])
def us_018_place(ctx):
    ex = ran(ctx, "sp", PLACE_POSITION)
    ended(ex, "fail", "check-error", "skd_fnc_place", "position", "[x, y, z]")
    stopped_in(ex, "move p1")


@check("US-018 a ceiling of -1 to skd_fnc_waitUntil ends fail check-error naming the function, ceiling and seconds above 0",
       "session", [CEILING_NEGATIVE])
def us_018_ceiling_negative(ctx):
    ex = ran(ctx, "sp", CEILING_NEGATIVE)
    ended(ex, "fail", "check-error", "skd_fnc_waitUntil", "ceiling", "number of seconds above 0")
    stopped_in(ex, "wait for p1")


@check("US-018 a ceiling of \"10\" to skd_fnc_waitUntil ends fail check-error naming the function, ceiling and seconds above 0",
       "session", [CEILING_STRING])
def us_018_ceiling_string(ctx):
    ex = ran(ctx, "sp", CEILING_STRING)
    ended(ex, "fail", "check-error", "skd_fnc_waitUntil", "ceiling", "number of seconds above 0")
    stopped_in(ex, "wait for p1")


@check("US-018 the check after the wrong arguments runs and passes", "session", US018_BROKEN + [US018_NEXT])
def us_018_next(ctx):
    result = ctx.m01["sp"][2]
    order = [c["check_id"] for c in result["checks"]]
    assert all(order.index(b) < order.index(US018_NEXT) for b in US018_BROKEN), f"run order {order}"
    passed(execution(result, US018_NEXT))
    assert all(execution(result, b)["outcome"] != "invalid" for b in US018_BROKEN), "a wrong argument ended invalid"


# ---------------------------------------------------------------- US-021, US-022 (setup cba)

@check("US-021 pressKey returns true and the server then reads skdfx_signalDown and skdfx_signalUp of 1", "session",
       [PRESS], setup="cba")
def us_021_press(ctx):
    passed(ran(ctx, "cba", PRESS))


@check("US-021 mode down raises only skdfx_signalDown and a later mode up raises skdfx_signalUp", "session",
       [DOWN_UP], setup="cba")
def us_021_down_up(ctx):
    passed(ran(ctx, "cba", DOWN_UP))


@check("US-022 a misspelled keybind ends fail check-error listing the keybinds of skd_fixture with signal", "session",
       [MISSPELLED], setup="cba")
def us_022_misspelled(ctx):
    ex = ran(ctx, "cba", MISSPELLED)
    ended(ex, "fail", "check-error", "skd_fixture", "signal")
    stopped_in(ex, "press a keybind that is not registered")


@check("US-022 ACT-03 a mod with no keybinds ends fail check-error listing the mods with keybinds in lower case",
       "session", [UNKNOWN_MOD], setup="cba")
def us_022_unknown_mod(ctx):
    ex = ran(ctx, "cba", UNKNOWN_MOD)
    ended(ex, "fail", "check-error", "skd_no_such_mod", "skd_fixture", "grappling hooks")
    stopped_in(ex, "press a keybind of a mod that registered none")


@check("US-022 without CBA pressKey ends fail check-error saying CBA is not loaded on p1", "session", [NO_CBA])
def us_022_no_cba(ctx):
    ex = ran(ctx, "sp", NO_CBA)
    ended(ex, "fail", "check-error", "CBA", "not loaded", "p1")
    stopped_in(ex, "press the fixture signal")


@check("US-022 API-05 without CBA a check that presses no key passes", "session", [NO_KEY])
def us_022_no_key(ctx):
    passed(ran(ctx, "sp", NO_KEY))
    machines = ctx.m01["sp"][2]["session"]["machines"]
    mods = [m["mod"] for m in machines[0]["mods"]]
    assert "cba" not in mods, f"p1 reports the mods {mods}"


# ---------------------------------------------------------------- US-039

@check("US-039 a step looping tryWaitUntil for 20 s under @timeout 5 ends fail check-timeout after about 5 s in slow part, running",
       "session", [SLOW_PART])
def us_039_slow_part(ctx):
    ex = ran(ctx, "sp", SLOW_PART)
    ended(ex, "fail", "check-timeout")
    assert 4.5 <= ex["duration_s"] <= 7, f"the check ended after {ex['duration_s']} s, expected about 5 s"
    stopped_in(ex, "slow part")
    step = [s for s in ex["steps"] if s["path"] == ["slow part"]]
    assert len(step) == 1 and step[0]["state"] == "running", f"the step slow part is recorded as {step}"
    assert step[0]["end"] is None, f"the running step has an end {step[0]['end']}"


@check("US-039 a wait whose ceiling of 30 s reaches past @timeout 3 is cut and the check ends check-timeout, not invalid",
       "session", [CEILING_CUT])
def us_039_ceiling_cut(ctx):
    ex = ran(ctx, "sp", CEILING_CUT)
    ended(ex, "fail", "check-timeout")
    assert 2.5 <= ex["duration_s"] <= 5, f"the check ended after {ex['duration_s']} s, expected about 3 s"
    stopped_in(ex, "wait past the timeout")


# ---------------------------------------------------------------- NFR-US-01

def sentences(message):
    return [s for s in re.split(r"(?<=[.!?])\s+(?=[A-Z])", message.strip()) if s]


@check("NFR-US-01 every check-error message of M1 names the function, what went wrong and what to do, in one or two sentences",
       "session", [UNKNOWN_CLASS, PLACE_POSITION, CEILING_NEGATIVE, CEILING_STRING, NO_CBA, NOT_BOOLEAN])
def nfr_us_01_check_errors(ctx):
    # What counts as the next thing to do: a rule the named argument must meet ("must be ..."), or
    # a sentence that tells the author what to do.
    next_step = ("must be", "Check ", "Make ", "Add ", "Keep ", "Give ", "Use ")
    for cid in [UNKNOWN_CLASS, PLACE_POSITION, CEILING_NEGATIVE, CEILING_STRING, NO_CBA, NOT_BOOLEAN]:
        message = ran(ctx, "sp", cid)["reason"]["message"]
        assert message.startswith("skd_fnc_"), f"{cid}: the message {message!r} does not start with the function"
        assert 1 <= len(sentences(message)) <= 2, f"{cid}: the message {message!r} is not one or two sentences"
        assert "\n" not in message and "panicked" not in message and ".rs:" not in message, \
            f"{cid}: the message {message!r} looks like a stack trace"
        assert any(n in message for n in next_step), f"{cid}: the message {message!r} names no next step"


@check("NFR-US-01 US-039 the check-timeout message says what to do next", "session", [SLOW_PART])
def nfr_us_01_timeout(ctx):
    message = ran(ctx, "sp", SLOW_PART)["reason"]["message"]
    assert "@timeout" in message, \
        f"the message {message!r} names no next step, such as raising @timeout or looking at the step that ran long"
    # US-039: the failure names the step it was in, so the author need not open the result file.
    assert "slow part" in message, f"the message {message!r} does not name the step slow part"
    assert 1 <= len(sentences(message)) <= 2, f"the message {message!r} is not one or two sentences"
    assert "\n" not in message and "panicked" not in message and ".rs:" not in message, \
        f"the message {message!r} looks like a stack trace"


# ---------------------------------------------------------------- the check API

@check("API-03 skd_fnc_machines, skd_fnc_setup and skd_fnc_player give p1, the setup sp and p1's unit", "session", [CONTEXT])
def api_03_context(ctx):
    passed(ran(ctx, "sp", CONTEXT))


@check("API-03 skd_fnc_invalid ends the check invalid explicit with its message", "session", [INVALID])
def api_03_invalid(ctx):
    ex = ran(ctx, "sp", INVALID)
    ended(ex, "invalid", "explicit", "the environment is broken on purpose")
    stopped_in(ex, "detect a broken environment")


@check("CTL-01 CTL-02 CTL-03 DM-13 DM-14 values, objects and groups cross to p1 and back, locals stay behind, code may wait",
       "session", [VALUES])
def ctl_01_values(ctx):
    passed(ran(ctx, "sp", VALUES))


@check("CTL-01 skd_fnc_onEach returns a map from machine name to value, leaving out a machine that returned nothing",
       "session", [ON_EACH])
def ctl_01_on_each(ctx):
    passed(ran(ctx, "sp", ON_EACH))


@check("OBS-01 OBS-07 a wait returns once its condition holds and a try wait returns false at its ceiling", "session", [WAITS])
def obs_01_waits(ctx):
    passed(ran(ctx, "sp", WAITS))


@check("OBS-01 a wait at its ceiling ends fail wait-ceiling with its description and ceiling", "session", [WAIT_CEILING])
def obs_01_wait_ceiling(ctx):
    ex = ran(ctx, "sp", WAIT_CEILING)
    ended(ex, "fail", "wait-ceiling", "the door that never opens is open", "1 s")
    stopped_in(ex, "wait for a door")
    assert 0.9 <= ex["duration_s"] <= 4, f"the check ended after {ex['duration_s']} s, the ceiling was 1 s"


@check("OBS-01 a condition that returns a number ends fail check-error naming the type it returned", "session", [NOT_BOOLEAN])
def obs_01_not_boolean(ctx):
    ex = ran(ctx, "sp", NOT_BOOLEAN)
    message = ended(ex, "fail", "check-error", "skd_fnc_waitUntil")
    assert "SCALAR" in message or "number" in message.lower(), f"the message {message!r} names no type"
    stopped_in(ex, "wait on a number")


@check("OBS-06 expectOn, expectEach and expectAgree hold and return what they read; every operator holds", "session", [EXPECT])
def obs_06_expect(ctx):
    passed(ran(ctx, "sp", EXPECT))


@check("OBS-06 expectEach at its ceiling ends fail assertion with every machine's last value", "session", [EACH_FAILS])
def obs_06_each_fails(ctx):
    ex = ran(ctx, "sp", EACH_FAILS)
    ended(ex, "fail", "assertion", "one is two on every machine", "p1")
    assert ex["evidence"]["observed"] == {"p1": 1}, f"evidence.observed {ex['evidence']['observed']}"
    stopped_in(ex, "expect one to be two")


@check("OBS-06 expectOn at its ceiling ends fail assertion with the last value read", "session", [ON_FAILS])
def obs_06_on_fails(ctx):
    ex = ran(ctx, "sp", ON_FAILS)
    ended(ex, "fail", "assertion", "seven is eight on p1", "7")
    assert ex["evidence"]["observed"] == {"p1": 7}, f"evidence.observed {ex['evidence']['observed']}"
    stopped_in(ex, "expect seven to be eight")


@check("SCN-01 skd_fnc_create makes an object, a vehicle and a unit in a new group of its side, at the position and direction",
       "session", [CREATE])
def scn_01(ctx):
    passed(ran(ctx, "sp", CREATE))


@check("SCN-02 skd_fnc_place puts p1 and a created unit where the server sees them within 0.1 m and 3 degrees", "session",
       [PLACE])
def scn_02(ctx):
    passed(ran(ctx, "sp", PLACE))


@check("SCN-03 skd_fnc_setDate and skd_fnc_setWeather return once every machine reports the date and the overcast",
       "session", [DATE_WEATHER])
def scn_03(ctx):
    passed(ran(ctx, "sp", DATE_WEATHER))


# ---------------------------------------------------------------- the reset

@check("RST-01 the cleanups a check registered run at the next reset, last registered first", "session",
       [CLEANUP_REGISTER, CLEANUP_RAN])
def rst_01_cleanup(ctx):
    result = ctx.m01["sp"][2]
    order = [c["check_id"] for c in result["checks"]]
    assert order.index(CLEANUP_RAN) == order.index(CLEANUP_REGISTER) + 1, f"run order {order}"
    passed(execution(result, CLEANUP_REGISTER))
    passed(execution(result, CLEANUP_RAN))


@check("RST-05 a player killed in a check gets a new unit and the next check finds p1 alive at its start with no body",
       "session", [RL[0], KILL, ALIVE_AFTER])
def rst_05(ctx):
    result = ctx.m01["sp"][2]
    order = [c["check_id"] for c in result["checks"]]
    assert order.index(ALIVE_AFTER) == order.index(KILL) + 1, f"run order {order}"
    passed(execution(result, KILL))
    passed(execution(result, ALIVE_AFTER))
    assert all(m["end_state"] == "ready" for m in result["session"]["machines"]), f"{result['session']['machines']}"


@check("NFR-RL-02 RST-02 SCN-04 M1 exit 3: 20 checks in one singleplayer session that change the scene each see the start state",
       "session", RL)
def nfr_rl_02(ctx):
    answer, out, result = ctx.m01["rl"]
    assert [c["check_id"] for c in result["checks"]] == RL, f"ran {[c['check_id'] for c in result['checks']]}"
    for ex in result["checks"]:
        passed(ex)
    assert answer.code == 0, f"{answer}, expected exit 0"
    assert result["session"]["machines"][0]["start"] == 1, "p1 was started again during the run"


# ---------------------------------------------------------------- the selection

def selection(ctx, name):
    answer, out, result = ctx.m01["sel"][name]
    assert answer.code == 0, f"{answer}"
    return [c["check_id"] for c in result["checks"]]


@check("RUN-03 checks given by id run in the order given", "session", [SEL_A, SEL_B, SEL_C])
def run_03_ids(ctx):
    got = selection(ctx, "ids")
    assert got == [SEL_C, SEL_A], f"ran {got}, expected {[SEL_C, SEL_A]}"


@check("RUN-01 RUN-03 checks selected by a tag run in check id order", "session", [SEL_A, SEL_B, SEL_C])
def run_03_tag(ctx):
    got = selection(ctx, "tag")
    assert got == [SEL_A, SEL_C], f"ran {got}, expected {[SEL_A, SEL_C]}"


@check("RUN-01 several tags select every check carrying any of them, in check id order", "session", [SEL_A, SEL_B, SEL_C])
def run_01_tags(ctx):
    got = selection(ctx, "tags")
    assert got == [SEL_A, SEL_B, SEL_C], f"ran {got}, expected {[SEL_A, SEL_B, SEL_C]}"


@check("RUN-01 ids and tags together select the union, each check once", "session", [SEL_A, SEL_B, SEL_C])
def run_01_union(ctx):
    got = selection(ctx, "ids and tags")
    assert sorted(got) == [SEL_A, SEL_B, SEL_C], f"ran {got}, expected the union of {SEL_B} and the tag acc-sel-x"


def repeated(ctx, name, ids):
    """The run of REPEATS named name ended exit 0, every execution passed, and selection.ids is ids as given."""
    answer, out, result = ctx.m01["rep"][name]
    assert answer.code == 0, f"{answer}, expected exit 0"
    assert result["run"]["selection"]["ids"] == ids, \
        f"run.selection.ids is {result['run']['selection']['ids']}, expected the ids as given {ids}"
    for ex in result["checks"]:
        passed(ex)
    assert result["not_run"] == [], f"not_run is {result['not_run']}"
    return [(c["check_id"], c["repeat"]) for c in result["checks"]], result


@check("US-031 m01-3 RUN-03 run a a b executes a, a, b and the result file holds three entries with repeat 1, 2, 1",
       "session", [SEL_A, SEL_B])
def us_031_a_a_b(ctx):
    got, result = repeated(ctx, "a a b", [SEL_A, SEL_A, SEL_B])
    assert got == [(SEL_A, 1), (SEL_A, 2), (SEL_B, 1)], f"ran {got}"
    s = result["summary"]
    assert s["total"] == 3 and s["pass"] == 3, f"summary {s}, expected 3 executions, 3 passed"


@check("US-031 m01-3 RUN-03 run a b a executes a, b, a with repeat 1, 1, 2", "session", [SEL_A, SEL_B])
def us_031_a_b_a(ctx):
    got, _ = repeated(ctx, "a b a", [SEL_A, SEL_B, SEL_A])
    assert got == [(SEL_A, 1), (SEL_B, 1), (SEL_A, 2)], f"ran {got}"


@check("US-031 m01-3 RUN-03 run a a a executes a three times with repeat 1, 2, 3", "session", [SEL_A])
def us_031_a_a_a(ctx):
    got, result = repeated(ctx, "a a a", [SEL_A, SEL_A, SEL_A])
    assert got == [(SEL_A, 1), (SEL_A, 2), (SEL_A, 3)], f"ran {got}"
    assert result["summary"]["total"] == 3, f"summary {result['summary']}"


@check("US-031 m01-3 RUN-01 a tag adds nothing for a check given twice by id: run a a --tag x executes a, a, c",
       "session", [SEL_A, SEL_C])
def us_031_a_a_tag(ctx):
    got, _ = repeated(ctx, "a a and a tag", [SEL_A, SEL_A])
    assert got == [(SEL_A, 1), (SEL_A, 2), (SEL_C, 1)], f"ran {got}, expected a, a by id, then c of the tag acc-sel-x"


@check("US-031 m01-3 RUN-03 run a a b without --json prints one line per execution, the second of a marked #2",
       "session", [SEL_A, SEL_B])
def us_031_text(ctx):
    answer, out, result = ctx.m01["rep text"]
    assert answer.code == 0, f"{answer}, expected exit 0; stderr {answer.stderr[:300]!r}"
    got = [(c["check_id"], c["repeat"]) for c in result["checks"]]
    assert got == [(SEL_A, 1), (SEL_A, 2), (SEL_B, 1)], f"ran {got}"
    lines = [l.rstrip() for l in answer.stdout.splitlines() if l.strip()]
    # The wording of text output is not contract, its information is: the line of the second
    # execution carries #2 after the check id (the line goes on with the duration).
    marked = [l for l in lines if re.search(r"#\d+\b", l)]
    assert len(marked) == 1 and re.search(re.escape(SEL_A) + r"\s+#2\b", marked[0]), \
        f"the lines marked #<repeat> are {marked}, expected one line of {SEL_A} marked #2; stdout {lines}"
    a_lines = [l for l in lines if SEL_A in l]
    assert len(a_lines) >= 2, f"the run printed {len(a_lines)} lines naming {SEL_A}, expected one per execution: {lines}"
    b_lines = [l for l in lines if SEL_B in l]
    assert b_lines, f"the run printed no line naming {SEL_B}: {lines}"


# ---------------------------------------------------------------- the demo (setup cba)

@check("M1 exit 2 the demo AGH-US-009 passes 10 of 10 in one run together with a second check against one session",
       "session", [DEMO_KEY, DEMO_ACTION], setup="cba")
def m1_demo(ctx):
    answer, out, result = ctx.m01["demo"]
    got = [(c["check_id"], c["repeat"]) for c in result["checks"]]
    expected = [(cid, n) for n in range(1, DEMO_RUNS + 1) for cid in (DEMO_KEY, DEMO_ACTION)]
    assert got == expected, f"the run executed {got}, expected the demo and the second check {DEMO_RUNS} times each"
    for ex in result["checks"]:
        assert ex["outcome"] == "pass", f"{ex['check_id']} #{ex['repeat']} ended {ex['outcome']}: {ex['reason']}"
    assert result["session"]["id"] == ctx.session["id"], "the run used another session"
    assert answer.code == 0, f"{answer}, expected exit 0"


# ---------------------------------------------------------------- the session

def ses_05(ctx):
    pids = {m["pid"] for m in ctx.session["machines"]}
    assert ctx.session_runs, "the session made no run"
    for answer, out, result, new in ctx.session_runs:
        assert result["session"]["id"] == ctx.session["id"], f"{out['run_id']} ran in session {result['session']['id']}"
        starts = [m["start"] for m in result["session"]["machines"]]
        assert starts == [1], f"{out['run_id']} ran on machine starts {starts}"
        errors = ctx.validate(result)
        assert not errors, f"{out['result_file']} is not valid against result.schema.json: {errors[:5]}"
    for pid in pids:
        assert driver.alive(pid), f"the session's game process {pid} is gone"
    ctx.log(f"{len(ctx.session_runs)} runs in session {ctx.session['id']} on game process {sorted(pids)}")


@check("SES-05 RES-01 every run of the sp session used the same session and game process and wrote a valid result file",
       "session", [RL[0]])
def ses_05_sp(ctx):
    ses_05(ctx)


@check("SES-05 RES-01 every run of the cba session used the same session and game process and wrote a valid result file",
       "session", [PRESS], setup="cba")
def ses_05_cba(ctx):
    ses_05(ctx)
    mods = {m["mod"]: m["version"] for m in ctx.session_runs[0][2]["session"]["machines"][0]["mods"]}
    assert mods.get("cba"), f"p1 reports no version for cba_main: {mods}"


@check("US-031 m01-3 RUN-02 list a a b lists a and b once each", "offline")
def us_031_list(ctx):
    answer = ctx.sh("list", SEL_A, SEL_A, SEL_B, "--json", ceiling=60)
    assert answer.code == 0, f"{answer}; stderr {answer.stderr[:300]!r}"
    got = [c["id"] for c in answer.json()["checks"]]
    assert sorted(got) == [SEL_A, SEL_B], f"list gave {got}, expected {SEL_A} and {SEL_B} once each"
