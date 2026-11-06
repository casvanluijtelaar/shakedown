"""The acceptance checks of M2 Dedicated sessions: US-002, US-004, US-008, US-009, US-017, US-019,
US-020, US-025, US-026, the requirements of a dedicated session, and exit criteria 2, 3 and 4."""

import json
import re
import subprocess
import time
from pathlib import Path

from . import driver
from .core import batch, check, execution
from .m01 import DEMO_ACTION, DEMO_KEY, ended, passed, stopped_in

# SQF check ids, as `list` gives them (their files are in acceptance/checks/).
MACHINES = "us002/US-002-machines"
GROUPS = "us002/DM-12-groups"
RL = [f"us008/US-008-start-state-{n:02}" for n in range(1, 21)]
RL_LAST = "us008/US-008-start-state-21"
KILL_P2 = "us009/US-009-kill-p2"
P2_ALIVE = "us009/US-009-p2-alive"
KILL_P1 = "us009/US-009-kill-p1"
P1_ALIVE = "us009/US-009-p1-alive"
SCENE = "us017/US-017-scene"
OPEN_CRATE = "us019/US-019-open-crate"
TAGGED = "us019/US-019-tagged-title"
WRONG_TITLE = "us020/US-020-wrong-title"
TOO_FAR = "us020/US-020-too-far"
OPENED = "us020/US-020-opened"
SERVER_ACTS = "us020/US-020-server"
PLAYER_IN_BODY = "us020/US-020-player-in-body"
AGREE = "us025/US-025-agree"
DISAGREE = "us026/US-026-disagree"
DEMO = "agh/AGH-US-009-dedicated"

# The ded batch runs these in this order; the kill of p2 is followed by the check that looks at it.
# No check of the ded session kills p1: arma-grappling-hook does not give a respawned unit its
# actions again, so a demo after a death of p1 would fail on the mod, not on the framework.
DED_ORDER = [MACHINES, GROUPS, SCENE, OPEN_CRATE, TAGGED, WRONG_TITLE, TOO_FAR, OPENED, SERVER_ACTS,
             PLAYER_IN_BODY, AGREE, DISAGREE, KILL_P2, P2_ALIVE, DEMO_KEY, DEMO_ACTION]
SP_ORDER = [SCENE, OPEN_CRATE, TAGGED, KILL_P1, P1_ALIVE]
DEMO_RUNS = 10
RESETS = 3
DED_MACHINES = ["server", "p1", "p2", "hc1"]


# ---------------------------------------------------------------- the runs, per repeat

@batch("ded")
def ded_batch(ctx, checks):
    needed = {i for c in checks for i in c.needs}
    ids = [i for i in DED_ORDER if i in needed]
    if ids:
        ctx.m02["ded"] = ctx.run(ids)
    if any(i in needed for i in RL + [RL_LAST]):
        ctx.m02["rl"] = ctx.run(RL + [RL_LAST])
    if any(c.name.startswith("US-008 RST-03") for c in checks):
        resets = []
        for _ in range(RESETS):
            run = ctx.run([RL[1]])
            answer = ctx.sh("reset", "--json", ceiling=60)
            resets.append((run, answer, answer.json() if answer.stdout.strip() else None))
        ctx.m02["resets"] = resets
    if DEMO in needed:
        ctx.m02["demo"] = ctx.run([DEMO] * DEMO_RUNS)
    if any(c.name.startswith("INS-02") for c in checks):
        ctx.m02["status"] = ctx.sh("status", "--json", ceiling=60)


@batch("sp")
def sp_batch(ctx, checks):
    needed = {i for c in checks for i in c.needs}
    ids = [i for i in SP_ORDER if i in needed]
    if ids:
        ctx.m02["sp"] = ctx.run(ids)


def ran(ctx, key, cid):
    assert key in ctx.m02, f"the {key} run of this repeat did not happen; see harness.log"
    return execution(ctx.m02[key][2], cid)


def back_to_back(result, first, second):
    order = [c["check_id"] for c in result["checks"]]
    assert order.index(second) == order.index(first) + 1, f"run order {order}"


def numbers(text):
    return [float(n) for n in re.findall(r"\d+(?:\.\d+)?", text)]


# ---------------------------------------------------------------- US-002

@check("US-002 SES-01 session start of a setup with 2 clients and 1 headless client exits 0 with server, p1, p2 and hc1 each ready",
       "session", [MACHINES], setup="ded")
def us_002_start(ctx):
    answer = ctx.start_answer
    assert answer.code == 0, f"{answer}, expected exit 0"
    s = ctx.session
    assert s["state"] == "ready" and s["mode"] == "dedicated", f"session state {s['state']}, mode {s['mode']}"
    got = [(m["name"], m["kind"], m["state"]) for m in s["machines"]]
    expected = [("server", "server", "ready"), ("p1", "client", "ready"), ("p2", "client", "ready"),
                ("hc1", "headless", "ready")]
    assert got == expected, f"the session's machines are {got}, expected {expected}"


@check("US-002 DM-10 DM-11 skd_fnc_machines gives server, p1, p2, hc1 and skd_fnc_player gives objNull for hc1 and server",
       "session", [MACHINES], setup="ded")
def us_002_machines(ctx):
    passed(ran(ctx, "ded", MACHINES))


@check("US-002 RES-02 the result file lists every machine with its kind, start 1, game version and each patched mod's version",
       "session", [MACHINES], setup="ded")
def us_002_result(ctx):
    result = ctx.m02["ded"][2]
    machines = result["session"]["machines"]
    assert [(m["name"], m["kind"], m["start"]) for m in machines] == \
        [("server", "server", 1), ("p1", "client", 1), ("p2", "client", 1), ("hc1", "headless", 1)], \
        f"session.machines {[(m['name'], m['kind'], m['start']) for m in machines]}"
    for m in machines:
        assert m["game_version"], f"{m['name']} has no game_version"
        mods = {x["mod"]: x for x in m["mods"]}
        for mod, patch in [("cba", "cba_main"), ("fixture", "skdfx_main"), ("fixture-cba", "skdfx_cba")]:
            assert mod in mods and mods[mod]["patch"] == patch and mods[mod]["version"], \
                f"{m['name']} reports {mods.get(mod)} for the mod {mod} with patch {patch}"
    setup = result["session"]["setup"]
    assert (setup["mode"], setup["clients"], setup["headless"]) == ("dedicated", 2, 1), f"session.setup {setup}"


@check("DM-12 P-03 CTL-01 the groups all, clients and headless and an array of names reach their machines, and values and objects travel",
       "session", [GROUPS], setup="ded")
def dm_12_groups(ctx):
    passed(ran(ctx, "ded", GROUPS))


# ---------------------------------------------------------------- US-004 (before any session)

def poll_start(ctx, proc, ceiling, until=None):
    """Polls `status --json` twice a second while a background `session start` runs, at most `ceiling` polls.

    Returns every status Session seen. Stops early when until(session) is true or the start ended.
    """
    seen = []
    for _ in range(ceiling):
        if proc.poll() is not None:
            break
        answer = ctx.sh("status", "--json", ceiling=30)
        try:
            session = answer.json().get("session")
        except AssertionError:
            session = None
        if session:
            seen.append((time.monotonic(), session))
            if until and until(session):
                break
        elif seen:
            ctx.log("status gives no session while session start has not yet returned")
            break
        time.sleep(0.5)
    return seen


def finish(proc, ceiling):
    try:
        out, err = proc.communicate(timeout=ceiling)
    except subprocess.TimeoutExpired:
        driver.kill(proc.pid)
        out, err = proc.communicate(timeout=30)
        raise AssertionError(f"session start did not return within {ceiling} s; stopped it by id")
    return proc.returncode, out, err


def gone_after(pids, ceiling):
    """The pids still alive after polling once a second, at most `ceiling` times."""
    left = set(pids)
    for _ in range(ceiling):
        left = {p for p in left if driver.alive(p)}
        if not left:
            break
        time.sleep(1)
    return left


def failed_start(ctx, setup, prepare):
    """Starts setup in the background, lets prepare(proc) act on it, and judges the failed start.

    Returns (exit code, Session JSON, every status Session seen, seconds the start took). Whatever
    happens, every game process the start left is stopped by its id.
    """
    before = driver.game_pids()
    started = time.monotonic()
    proc = ctx.sh.start("session", "start", "--setup", setup, "--json")
    seen, left = [], set()
    try:
        seen = prepare(proc)
        code, out, err = finish(proc, 240)
        took = time.monotonic() - started
        ctx.log(f"`shakedown session start --setup {setup}` exit {code} in {took:.1f} s; stderr {err[:300]!r}")
    finally:
        if proc.poll() is None:
            driver.kill(proc.pid)
        ours = (driver.game_pids() - before) | {m["pid"] for _, s in seen for m in s["machines"] if m.get("pid")}
        left = gone_after(ours, 10)
        for pid in left:
            ctx.log(f"game process {pid} of the failed start still runs; stopping it by id")
            driver.kill(pid)
    try:
        session = json.loads(out)
    except ValueError:
        raise AssertionError(f"session start printed no JSON document: stdout {out[:300]!r}, stderr {err[:300]!r}")
    ctx.failed_starts.append(session)
    last = None
    for at, s in seen:
        row = [(m["name"], m["state"], m["pid"]) for m in s["machines"]]
        if row != last:
            ctx.log(f"status {at - started:.1f} s into the start: {s['state']} {row}")
            last = row
    ctx.log(f"the start ended {session.get('state')}: {session.get('reason')}")
    assert not left, f"the game processes {sorted(left)} of the failed start still ran 10 s after it returned"
    return code, session, seen, took


@check("US-004 SES-04 a client ended during the start fails it with exit 4 naming p2, its state and its last RPT lines, leaving nothing running",
       "start")
def us_004_killed(ctx):
    killed = {}

    def prepare(proc):
        def p2_up(s):
            return any(m["name"] == "p2" and m.get("pid") for m in s["machines"])
        seen = poll_start(ctx, proc, 400, p2_up)
        assert seen and p2_up(seen[-1][1]), "status never listed p2 with a process id during the start"
        p2 = [m for m in seen[-1][1]["machines"] if m["name"] == "p2"][0]
        # Let p2 write some of its RPT before it is ended: poll the profile folder, at most 30 times.
        rpt = None
        for _ in range(30):
            rpts = sorted(Path(p2["profile"]).glob("*.rpt")) if p2.get("profile") else []
            if rpts and len(rpts[-1].read_text(encoding="utf-8", errors="replace").splitlines()) >= 25:
                rpt = rpts[-1]
                break
            if proc.poll() is not None:
                break
            time.sleep(1)
        killed.update(pid=p2["pid"], at=time.monotonic(), rpt=rpt, state=p2["state"])
        ctx.log(f"ending p2's process {p2['pid']} in state {p2['state']}")
        subprocess.run(["taskkill", "/PID", str(p2["pid"]), "/F"], capture_output=True, timeout=30)
        return seen

    ctx.start_seen = []
    code, session, seen, took = failed_start(ctx, "ded", prepare)
    ctx.start_seen = seen
    assert code == 4, f"session start exited {code}, expected 4"
    assert session["state"] == "failed", f"the session ended {session['state']}, expected failed"
    reason = session["reason"] or {}
    assert reason.get("machine") == "p2", f"reason.machine is {reason.get('machine')!r}, expected p2: {reason}"
    message = reason.get("message", "")
    assert "p2" in message, f"the message {message!r} does not name p2"
    assert "panicked" not in message and ".rs:" not in message, f"the message {message!r} looks like a stack trace"
    tail = reason.get("rpt_tail")
    assert isinstance(tail, list) and tail, f"reason.rpt_tail is {tail!r}, expected p2's last RPT lines"
    if killed.get("rpt"):
        assert len(tail) == 20, f"reason.rpt_tail holds {len(tail)} lines of an RPT of 25 or more lines, expected 20"
    since_kill = time.monotonic() - killed["at"]
    assert since_kill < 90, f"session start returned {since_kill:.0f} s after p2 ended"


@check("INS-04 m02-1 status during a dedicated start lists a machine as launched only with its process id and process start time",
       "start")
def ins_04_starting(ctx):
    seen = getattr(ctx, "start_seen", None)
    assert seen, "the start of the US-004 check that ends p2 was not watched; run it with this check"
    for at, s in seen:
        assert s["state"] == "starting", f"status gave the session state {s['state']} during the start"
        for m in s["machines"]:
            assert m["profile"] and m["start"] == 1 and m["health"] is not None, f"status during the start lists {m}"
            if m["state"] in ("launched", "connected", "ready"):
                assert m["pid"] is not None and m["process_started_at"] is not None,                     f"{at - seen[0][0]:.1f} s into the watch, status lists {m['name']} as {m['state']} with pid "                     f"{m['pid']} and process_started_at {m['process_started_at']}"


@check("US-004 SES-04 a setup with start_timeout 30 fails at about 30 s with exit 4, naming each machine not ready and its state",
       "start")
def us_004_timeout(ctx):
    def prepare(proc):
        return poll_start(ctx, proc, 200)

    code, session, seen, took = failed_start(ctx, "ded30", prepare)
    assert code == 4, f"session start exited {code}, expected 4"
    assert session["state"] == "failed", f"the session ended {session['state']}, expected failed"
    assert 29 <= took <= 75, f"the start took {took:.0f} s with a start ceiling of 30 s"
    message = (session["reason"] or {}).get("message", "")
    assert seen, "status never listed the starting session"
    last = seen[-1][1]["machines"]
    not_ready = [m for m in last if m["state"] != "ready"]
    ctx.log(f"the last status before the ceiling: {[(m['name'], m['state']) for m in last]}; message {message!r}")
    for m in not_ready:
        assert m["name"] in message, f"the message {message!r} does not name {m['name']}, {m['state']} in the last status"
    named = [n for n in DED_MACHINES if re.search(rf"\b{n}\b", message)]
    assert named, f"the message {message!r} names no machine"
    assert "30" in message or "ceiling" in message or "start_timeout" in message, \
        f"the message {message!r} does not say the start ceiling passed"


@check("US-004 after the failed starts the next session start is accepted and ready", "session", [MACHINES], setup="ded")
def us_004_next(ctx):
    assert ctx.failed_starts, "no start failed before this session; run the US-004 checks with it"
    assert ctx.start_answer.code == 0 and ctx.session["state"] == "ready", f"{ctx.start_answer}, state {ctx.session['state']}"


# ---------------------------------------------------------------- US-008, NFR-RL-02, NFR-PF-04

@check("US-008 NFR-RL-02 RST-02 SCN-04 M2 exit 3: 20 checks in a dedicated session with 2 clients and a headless client each see the start state on every machine",
       "session", RL + [RL_LAST], setup="ded")
def us_008_rl(ctx):
    answer, out, result = ctx.m02["rl"]
    assert [c["check_id"] for c in result["checks"]] == RL + [RL_LAST], f"ran {[c['check_id'] for c in result['checks']]}"
    for ex in result["checks"]:
        passed(ex)
    assert answer.code == 0, f"{answer}, expected exit 0"
    got = [(m["name"], m["start"], m["end_state"]) for m in result["session"]["machines"]]
    assert got == [(n, 1, "ready") for n in DED_MACHINES], f"session.machines {got}"


@check("US-008 RST-03 NFR-PF-04 M2 exit 3: shakedown reset after a check that changed the scene is done in under 2 s on a dedicated session",
       "session", [RL[1]], setup="ded")
def us_008_reset(ctx):
    for (run, answer, out) in ctx.m02["resets"]:
        assert answer.code == 0, f"{answer}, expected exit 0; stderr {answer.stderr[:300]!r}"
        assert out and out["state"] == "done", f"reset answered {out}"
        assert out["duration_s"] < 2, f"the reset took {out['duration_s']} s"
        assert out["mismatches"] == [], f"the reset reported {out['mismatches']}"
    ctx.log(f"reset durations: {[o['duration_s'] for _, _, o in ctx.m02['resets']]}")


# ---------------------------------------------------------------- US-009

@check("US-009 RST-05 after a check that kills p2 on a dedicated session the next check sees p2 alive at its start, and p2 keeps start 1",
       "session", [KILL_P2, P2_ALIVE], setup="ded")
def us_009_ded(ctx):
    result = ctx.m02["ded"][2]
    back_to_back(result, KILL_P2, P2_ALIVE)
    passed(execution(result, KILL_P2))
    passed(execution(result, P2_ALIVE))
    starts = {m["name"]: (m["start"], m["end_state"]) for m in result["session"]["machines"]}
    assert starts["p2"] == (1, "ready"), f"p2 is {starts['p2']} in the result file"


@check("US-009 RST-05 after a check that kills p1 in singleplayer the next check's expectOn p1 alive passes and the session stays ready",
       "session", [KILL_P1, P1_ALIVE], setup="sp")
def us_009_sp(ctx):
    result = ctx.m02["sp"][2]
    back_to_back(result, KILL_P1, P1_ALIVE)
    passed(execution(result, KILL_P1))
    passed(execution(result, P1_ALIVE))
    assert all(m["end_state"] == "ready" and m["start"] == 1 for m in result["session"]["machines"]), \
        f"{result['session']['machines']}"


# ---------------------------------------------------------------- US-017

@check("US-017 SCN-01 SCN-02 SCN-03 the scene check passes on a dedicated session with every machine seeing the same scene",
       "session", [SCENE], setup="ded")
def us_017_ded(ctx):
    passed(ran(ctx, "ded", SCENE))


@check("US-017 the same scene check, unchanged, passes in singleplayer", "session", [SCENE], setup="sp")
def us_017_sp(ctx):
    passed(ran(ctx, "sp", SCENE))


# ---------------------------------------------------------------- US-019

@check("US-019 ACT-01 OBS-06 p1 opens the crate through its action and the server, p1 and p2 see one more first aid kit",
       "session", [OPEN_CRATE], setup="ded")
def us_019_ded(ctx):
    passed(ran(ctx, "ded", OPEN_CRATE))


@check("US-019 the same crate check, unchanged, passes in singleplayer", "session", [OPEN_CRATE], setup="sp")
def us_019_sp(ctx):
    passed(ran(ctx, "sp", OPEN_CRATE))


@check("US-019 ACT-01 an action titled in structured text is used by its title without the tags on a dedicated session",
       "session", [TAGGED], setup="ded")
def us_019_tagged_ded(ctx):
    passed(ran(ctx, "ded", TAGGED))


@check("US-019 ACT-01 an action titled in structured text is used by its title without the tags in singleplayer",
       "session", [TAGGED], setup="sp")
def us_019_tagged_sp(ctx):
    passed(ran(ctx, "sp", TAGGED))


# ---------------------------------------------------------------- US-020

@check("US-020 Open Crate ends fail assertion saying there is no such action and listing Open crate, on p1",
       "session", [WRONG_TITLE], setup="ded")
def us_020_wrong_title(ctx):
    ex = ran(ctx, "ded", WRONG_TITLE)
    ended(ex, "fail", "assertion", "Open Crate", "Open crate")
    assert ex["reason"]["machine"] == "p1", f"reason.machine is {ex['reason']['machine']!r}, expected p1"
    stopped_in(ex, "p1 uses Open Crate")


@check("US-020 p1 10 m from the crate ends fail assertion giving the distance and the radius of 3 m", "session",
       [TOO_FAR], setup="ded")
def us_020_too_far(ctx):
    ex = ran(ctx, "ded", TOO_FAR)
    message = ended(ex, "fail", "assertion")
    found = numbers(message)
    assert any(9 <= n <= 11 for n in found), f"the message {message!r} gives no distance of about 10 m"
    assert 3 in found and "radius" in message.lower(), f"the message {message!r} does not give the radius of 3 m"
    assert ex["reason"]["machine"] == "p1", f"reason.machine is {ex['reason']['machine']!r}, expected p1"
    stopped_in(ex, "p1 uses Open crate from 10 m")


@check("US-020 an opened crate ends fail assertion saying the action's condition is false", "session", [OPENED], setup="ded")
def us_020_opened(ctx):
    ex = ran(ctx, "ded", OPENED)
    message = ended(ex, "fail", "assertion")
    assert "condition" in message.lower() and "false" in message.lower(), \
        f"the message {message!r} does not say the action's condition is false"
    stopped_in(ex, "p1 opens it again")


@check("US-020 server as the acting machine ends fail check-error saying server has no player", "session",
       [SERVER_ACTS], setup="ded")
def us_020_server(ctx):
    ex = ran(ctx, "ded", SERVER_ACTS)
    message = ended(ex, "fail", "check-error", "server")
    assert "player" in message.lower(), f"the message {message!r} does not say server has no player"
    stopped_in(ex, "server uses Open crate")


@check("US-020 ACT-01 player as the target in a dedicated check's body ends fail check-error naming objNull and skd_fnc_player",
       "session", [PLAYER_IN_BODY], setup="ded")
def us_020_player_in_body(ctx):
    ex = ran(ctx, "ded", PLAYER_IN_BODY)
    ended(ex, "fail", "check-error", "objNull", "skd_fnc_player")
    stopped_in(ex, "p1 uses an action on player")


@check("NFR-US-01 the US-020 messages are one or two sentences without a stack trace", "session",
       [WRONG_TITLE, TOO_FAR, OPENED, SERVER_ACTS, PLAYER_IN_BODY], setup="ded")
def nfr_us_01_actions(ctx):
    from .m01 import sentences
    for cid in [WRONG_TITLE, TOO_FAR, OPENED, SERVER_ACTS, PLAYER_IN_BODY]:
        message = ran(ctx, "ded", cid)["reason"]["message"]
        assert 1 <= len(sentences(message)) <= 2, f"{cid}: the message {message!r} is not one or two sentences"
        assert "\n" not in message and "panicked" not in message and ".rs:" not in message, \
            f"{cid}: the message {message!r} looks like a stack trace"


# ---------------------------------------------------------------- US-025, US-026

@check("US-025 OBS-06 expectAgree on server, p1 and p2 returns 5 for each, expectEach clients sees 5, positions and numbers agree within tolerance",
       "session", [AGREE], setup="ded")
def us_025(ctx):
    passed(ran(ctx, "ded", AGREE))


@check("US-026 OBS-06 OBS-04 RES-03 p2 reading 6 where server and p1 read 5 ends fail assertion with reason.machine p2 and every value",
       "session", [DISAGREE], setup="ded")
def us_026(ctx):
    ex = ran(ctx, "ded", DISAGREE)
    message = ended(ex, "fail", "assertion", "flag agrees")
    assert ex["reason"]["machine"] == "p2", f"reason.machine is {ex['reason']['machine']!r}, expected p2"
    assert re.search(r"p2\D*\b6\b.*\bserver\D*\b5\b", message), f"the message {message!r} does not say p2 read 6 where server read 5"
    assert ex["evidence"]["observed"] == {"server": 5, "p1": 5, "p2": 6}, f"evidence.observed {ex['evidence']['observed']}"
    stopped_in(ex, "server, p1 and p2 agree")


# ---------------------------------------------------------------- the demo and the M1 demo

@check("M2 exit 2 the demo AGH-US-009 on a dedicated session with two clients passes 10 of 10 in one session",
       "session", [DEMO], setup="ded")
def m2_demo(ctx):
    answer, out, result = ctx.m02["demo"]
    got = [(c["check_id"], c["repeat"]) for c in result["checks"]]
    assert got == [(DEMO, n) for n in range(1, DEMO_RUNS + 1)], f"the run executed {got}"
    for ex in result["checks"]:
        assert ex["outcome"] == "pass", f"{ex['check_id']} #{ex['repeat']} ended {ex['outcome']}: {ex['reason']}"
    assert answer.code == 0 and result["session"]["id"] == ctx.session["id"], f"{answer}"


@check("M2 exit 4 the M1 demo checks AGH-US-009 pass unchanged on a dedicated session", "session",
       [DEMO_KEY, DEMO_ACTION], setup="ded")
def m2_m1_demo(ctx):
    passed(ran(ctx, "ded", DEMO_KEY))
    passed(ran(ctx, "ded", DEMO_ACTION))


# ---------------------------------------------------------------- the session

@check("SES-05 RES-01 every run of the ded session used the same session and machine starts and wrote a valid result file",
       "session", [MACHINES], setup="ded")
def ses_05_ded(ctx):
    pids = {m["pid"] for m in ctx.session["machines"]}
    assert ctx.session_runs, "the session made no run"
    for answer, out, result, new in ctx.session_runs:
        assert result["session"]["id"] == ctx.session["id"], f"{out['run_id']} ran in session {result['session']['id']}"
        starts = [(m["name"], m["start"]) for m in result["session"]["machines"]]
        assert starts == [(n, 1) for n in DED_MACHINES], f"{out['run_id']} ran on machine starts {starts}"
        errors = ctx.validate(result)
        assert not errors, f"{out['result_file']} is not valid against result.schema.json: {errors[:5]}"
    for pid in pids:
        assert driver.alive(pid), f"the session's game process {pid} is gone"


@check("INS-02 INS-04 status lists the runner and each machine of the ready dedicated session with its process, start time and profile folder",
       "session", [MACHINES], setup="ded")
def ins_02(ctx):
    answer = ctx.m02["status"]
    assert answer.code == 0, f"{answer}"
    st = answer.json()
    assert st["runner"] == "running" and isinstance(st["runner_pid"], int), f"runner {st['runner']}, pid {st['runner_pid']}"
    s = st["session"]
    assert s["id"] == ctx.session["id"] and s["state"] == "ready", f"status session {s['id']} {s['state']}"
    assert [m["name"] for m in s["machines"]] == DED_MACHINES, f"status machines {[m['name'] for m in s['machines']]}"
    for m in s["machines"]:
        assert driver.alive(m["pid"]), f"{m['name']}'s process {m['pid']} is not running"
        assert m["process_started_at"] and Path(m["profile"]).is_dir(), f"{m['name']}: {m}"
        assert m["state"] == "ready" and m["start"] == 1, f"{m['name']}: {m}"
        assert set(m["health"]) >= {"flag", "since", "heartbeat"}, f"{m['name']}'s health {m['health']}"


@check("STP-02 SES-07 session stop of the dedicated session leaves no server, client or headless client process", "end")
def stp_02_ded(ctx):
    assert "ded" in ctx.stops, "the ded session was not stopped by this run; run a ded check with it"
    answer, alive = ctx.stops["ded"]
    assert answer.code == 0, f"{answer}"
    assert answer.json()["state"] == "stopped", f"session stop answered {answer.json()['state']}"
    assert not alive, f"the processes {sorted(alive)} of the ded session still ran when session stop returned"
