"""The checks of M11 Late joiners: US-066, US-067, US-068 and US-069.

Each part starts and stops a session of its own, once per repeat (`--repeat`), with no session running
before it, in the project `m11` in the work folder: the checks of `acceptance/m11/` and two setups loading
`@skd_fixture` alone, `ded` (a dedicated server with two clients, one late client `p3` and a headless
client) and `sp` (singleplayer). Per repeat:

- `ded`: the session's machines at the start (US-066 1); a late `p3` joins and agrees on a crate, then the
  next check starts without it, while `status` is read four times a second (US-066 2 to 4); the same check
  through `run_start` (US-066 5); `p2` leaves and joins again in one check, with the harness waiting for
  `status` to show `p2` gone before it lets the check go on (US-067 1 to 3, 5); a check that leaves `p2`
  out, then the next (US-067 4); a join of `p3` whose process the harness suspends by its id right after its
  launch, and one whose process it ends by its id (US-068); the five wrong joins and leaves, each followed
  by a check that passes (US-069 1, 3).
- `sp`: `["p1"] call skd_fnc_leave` and the check after it (US-069 2, 3).

A suspended process is resumed by its id before its session stops; every game process a part saw is
stopped by its id if it outlives the stop. A part whose game start is lost to `Error 20 reading file` of a
base-game file is started again, at most three times in all.
"""

import json
import re
import shutil
import threading
import time
import traceback
from datetime import datetime, timezone
from pathlib import Path

from . import driver, win
from .core import check
from .m04 import message
from .m06 import finish
from .m08 import game_folder, steam_running
from .mcp import Mcp, structured

ACC = Path(__file__).resolve().parent.parent
JOIN, AFTER = "US-066-join-late", "after"
REJOIN, LEAVE_ONLY = "US-067-leave-rejoin", "US-067-leave-only"
HANG = "US-068-join-hangs"
MISTAKES = {  # check id -> (the machine named, the machines that can join then, that can leave then)
    "US-069-leave-server": ("server", {"p3"}, {"p1", "p2"}),
    "US-069-leave-hc1": ("hc1", {"p3"}, {"p1", "p2"}),
    "US-069-join-p1": ("p1", {"p3"}, {"p1", "p2"}),
    "US-069-join-p9": ("p9", {"p3"}, {"p1", "p2"}),
    "US-069-leave-twice": ("p2", {"p2", "p3"}, {"p1"}),
}
SP_LEAVE, AFTER_SP = "US-069-sp-leave", "after-sp"
START_MACHINES = ["server", "p1", "p2", "hc1"]
GRACE = 10                 # s, OBS-03
LEAVE_CEILING = 20         # s, the default ceiling of skd_fnc_leave (check-api.md)
START_CEILING = 180        # s, the default start_timeout (SES-04), the default ceiling of skd_fnc_join


# ================================================================ the project and the parts

def keep(ctx):
    if not hasattr(ctx, "m11_keep"):
        ctx.m11_keep = {}
    return ctx.m11_keep


def own_project(ctx):
    store = keep(ctx)
    if "project" in store:
        return store["project"]
    game = game_folder(ctx.project)
    project = ctx.work / "m11"
    if project.exists():
        shutil.rmtree(project)
    shutil.copytree(ACC / "m11", project / "tests")
    (project / "shakedown.toml").write_text(
        f"game = '{game}'\n\n[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n\n"
        "[setups.ded]\nmode = \"dedicated\"\nclients = 2\nlate = 1\nheadless = 1\nmods = [\"fixture\"]\n\n"
        "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n", encoding="utf-8")
    (project / ".mcp.json").write_text(json.dumps(
        {"mcpServers": {"shakedown": {"command": "shakedown", "args": ["mcp"], "timeout": 70000}}}, indent=2),
        encoding="utf-8")
    sh = driver.Shakedown(ctx.sh.exe, project, ctx.log)
    store["project"] = (project, sh)
    return store["project"]


def machine_fault(ctx, since):
    root = ctx.work / "m11" / ".shakedown" / "sessions"
    for rpt in root.glob("*/machines/*/*.rpt") if root.is_dir() else []:
        if rpt.stat().st_mtime >= since - 1:
            for line in rpt.read_text(encoding="utf-8", errors="replace").splitlines():
                if "Error 20 reading file" in line:
                    return line.strip()[:200]
    return None


def once(ctx, key, fn):
    """Runs a part once per repeat and keeps its value or its error; the checks of that repeat share it."""
    store = keep(ctx)
    k = (key, getattr(ctx, "repeat_n", 1))
    if k not in store:
        for attempt in range(1, 4):
            began = time.time()
            try:
                store[k] = fn(ctx)
                break
            except Exception as e:  # noqa: BLE001
                ctx.log(f"M11 part {k}: {traceback.format_exc()}")
                store[k] = e
                fault = machine_fault(ctx, began)
                if not fault or attempt == 3:
                    break
                ctx.log(f"M11 part {k}: a game process could not read a base-game file ({fault}); the machine, "
                        f"not the framework. Starting the part again, attempt {attempt + 1} of 3")
    value = store[k]
    if isinstance(value, Exception):
        raise AssertionError(f"the M11 part {key} failed: {value!r}")
    return value


def got(out, key):
    """What one stage of a part stored; a stage that raised fails the checks that read it."""
    assert key in out, f"the stage {key} did not run (an earlier stage stopped the part): {out.get('stopped')}"
    value = out[key]
    if isinstance(value, Exception):
        raise AssertionError(f"the stage {key} failed: {value!r}")
    return value


def stage(ctx, out, key, fn):
    try:
        out[key] = fn()
    except Exception as e:  # noqa: BLE001
        ctx.log(f"M11 stage {key}: {traceback.format_exc()}")
        out[key] = e


def status_of(sh):
    return sh("status", "--json", ceiling=30).json()


def machines(st):
    return {m["name"]: m for m in (st.get("session") or {}).get("machines", [])}


def need_quiet(sh):
    assert steam_running(), "no steam process runs; a game start would be refused"
    busy = driver.game_pids()
    assert not busy, f"game processes run before the part starts: {sorted(busy)}"
    st = status_of(sh)
    assert st.get("session") is None, f"status shows a session before the part starts: {(st['session'] or {}).get('id')}"


def start(sh, setup, pids):
    answer = sh("session", "start", "--setup", setup, "--json", ceiling=400)
    session = answer.json() if answer.stdout.strip() else {}
    pids |= {m["pid"] for m in session.get("machines", []) if m.get("pid")}
    assert answer.code == 0 and session.get("state") == "ready", \
        f"{answer}: state {session.get('state')}, reason {str(session.get('reason'))[:600]}"
    return session


def stop(ctx, sh, pids, suspended):
    for pid in suspended:
        try:
            if driver.alive(pid):
                win.resume(pid)
        except Exception as e:  # noqa: BLE001
            ctx.log(f"M11: resuming {pid}: {e!r}")
    try:
        if status_of(sh).get("session"):
            sh("session", "stop", "--json", ceiling=180)
    except Exception as e:  # noqa: BLE001
        ctx.log(f"M11: status or session stop after the part: {e!r}")
    for pid in set(pids):
        if driver.alive(pid) and driver.wait_gone(pid, 30) is None:
            ctx.log(f"M11: game process {pid} still runs; stopping it by id")
            driver.kill(pid)
    ctx.runner_pid = driver.runner_pid() or ctx.runner_pid


def result_of(answer):
    out = answer.json()
    folder = Path(out["result_file"]).parent
    return {"answer": answer, "out": out, "result": json.loads((folder / "result.json").read_text(encoding="utf-8")),
            "folder": folder}


def run(sh, ids, ceiling=900):
    return result_of(sh("run", *ids, "--json", ceiling=ceiling))


class Watch:
    """Reads `status --json` four times a second in a thread of its own, keeping every answer with its time,
    and adds every game process id it sees to `pids`. A quiet driver, so the harness log keeps one line."""

    def __init__(self, ctx, project, pids, ceiling=900):
        self.sh = driver.Shakedown(ctx.sh.exe, project, lambda text: None)
        self.pids, self.snaps, self.ceiling = pids, [], ceiling
        self.done = threading.Event()
        self.thread = threading.Thread(target=self._loop, daemon=True)
        self.thread.start()

    def _loop(self):
        began = time.monotonic()
        while not self.done.is_set() and time.monotonic() - began < self.ceiling:
            try:
                st = self.sh("status", "--json", ceiling=30).json()
                self.snaps.append((time.time(), st))
                self.pids |= {m["pid"] for m in machines(st).values() if m.get("pid")}
            except Exception:  # noqa: BLE001  a status that failed once is read again
                pass
            self.done.wait(0.25)

    def stop(self):
        self.done.set()
        self.thread.join(timeout=60)
        return self.snaps

    def latest(self):
        return self.snaps[-1][1] if self.snaps else {}


def seen(snaps, name):
    """Every (time, machine) in which status listed this machine."""
    return [(t, machines(st)[name]) for t, st in snaps if name in machines(st)]


def poll(fn, ceiling, period=0.2):
    """Calls fn until it returns something truthy, at most `ceiling` seconds; returns that or None."""
    for _ in range(int(ceiling / period)):
        value = fn()
        if value:
            return value
        time.sleep(period)
    return None


def rpt_lines(profile):
    rpts = sorted(Path(profile).rglob("*.rpt"), key=lambda p: p.stat().st_mtime) if profile else []
    if not rpts:
        return []
    return [line.strip() for line in rpts[-1].read_text(encoding="utf-8", errors="replace").splitlines() if line.strip()]


def strings_in(value):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for v in value.values():
            yield from strings_in(v)
    elif isinstance(value, list):
        for v in value:
            yield from strings_in(v)


def utc(text):
    return datetime.strptime(text, "%Y-%m-%dT%H:%M:%S.%fZ").replace(tzinfo=timezone.utc)


def execution(result, check_id, repeat=1):
    found = [c for c in result["checks"] if c["check_id"] == check_id and c.get("repeat", 1) == repeat]
    assert len(found) == 1, f"the result file holds {len(found)} executions of {check_id} #{repeat}, expected 1"
    return found[0]


def passed(ex):
    assert ex["outcome"] == "pass", f"{ex['check_id']} ended {ex['outcome']}: {ex['reason']}"


def step_of(ex, title):
    found = [s for s in ex["steps"] if s["path"] == [title]]
    assert len(found) == 1, f"{ex['check_id']} has {len(found)} steps {title!r}: {[s['path'] for s in ex['steps']]}"
    return found[0]


def starts(result, name):
    return [m for m in result["session"]["machines"] if m["name"] == name]


# ---------------------------------------------------------------- the stages of the ded part

def join_late(ctx, project, sh, pids):
    before = status_of(sh)
    w = Watch(ctx, project, pids)
    try:
        r = run(sh, [JOIN, AFTER])
    finally:
        snaps = w.stop()
    p3 = seen(snaps, "p3")
    p3_pids = {m["pid"] for _, m in p3 if m.get("pid")}
    return {"run": r, "before": before, "snaps": snaps, "p3_pids": p3_pids,
            "p3_alive": {pid: driver.alive(pid) for pid in p3_pids}, "after": status_of(sh)}


def join_by_tool(ctx, project, pids):
    calls = []
    with Mcp(project, ctx.install, ctx.log) as m:
        def call(name, args):
            result, took = m.call(name, args, ceiling=120)
            calls.append((name, took))
            return structured(result)
        started = call("run_start", {"ids": [JOIN]})
        last = None
        for _ in range(20):
            last = call("run_status", {"run_id": started["run_id"], "wait_s": 45})
            if last.get("state") in ("finished", "stopped"):
                break
        res = call("run_result", {"run_id": started["run_id"]})
    result = json.loads(Path(res["result_file"]).read_text(encoding="utf-8"))
    return {"calls": calls, "status": last, "result_tool": res, "result": result}


def leave_and_rejoin(ctx, project, sh, pids):
    before = status_of(sh)
    old = machines(before)["p2"]
    w = Watch(ctx, project, pids)
    args = ("run", REJOIN, AFTER, "--json")
    bg = sh.start(*args)
    gone, go = None, None
    try:
        def p2_gone():
            st = w.latest()
            if st and "p2" not in machines(st) and not driver.alive(old["pid"]):
                return (time.time(), st)
            return None
        gone = poll(p2_gone, 120, 0.25)
        if gone:
            go = sh("exec", "--machine", "server", "--code", "missionNamespace setVariable ['skd_m11_go', true]; true",
                    "--json", ceiling=60)
        answer = finish(ctx, bg, args, ceiling=600)
    finally:
        if bg.poll() is None:
            driver.kill(bg.pid)
        snaps = w.stop()
    return {"run": result_of(answer), "before": before, "old": old, "gone": gone, "go": go, "snaps": snaps,
            "after": status_of(sh)}


def leave_out(ctx, project, sh, pids):
    before = status_of(sh)
    w = Watch(ctx, project, pids)
    try:
        r = run(sh, [LEAVE_ONLY, AFTER])
    finally:
        snaps = w.stop()
    return {"run": r, "before": before, "snaps": snaps, "after": status_of(sh)}


def broken_join(ctx, project, sh, pids, suspended, how):
    """Runs the join of p3 with a ceiling of 30 s and, once p3's process is listed and has written its first
    RPT lines, suspends it (how == "suspend") or ends it (how == "end") by its id."""
    before = status_of(sh)
    qsh = driver.Shakedown(ctx.sh.exe, project, lambda text: None)
    w = Watch(ctx, project, pids)
    args = ("run", HANG, AFTER, "--json")
    bg = sh.start(*args)
    acted = None
    try:
        def p3_listed():
            try:
                m = machines(qsh("status", "--json", ceiling=30).json()).get("p3")
            except AssertionError:
                return None
            return m if m and m.get("pid") else None
        p3 = poll(p3_listed, 120, 0.2)
        if p3:
            pids.add(p3["pid"])
            poll(lambda: len(rpt_lines(p3.get("profile"))) >= 5, 15, 0.2)
            lines = rpt_lines(p3.get("profile"))
            if how == "suspend":
                win.suspend(p3["pid"])
                suspended.append(p3["pid"])
            else:
                driver.kill(p3["pid"])
            acted = {"at": time.time(), "p3": p3, "rpt_lines_then": len(lines)}
            ctx.log(f"M11: {how} p3, pid {p3['pid']}, by its id, with {len(lines)} lines in its RPT")
        answer = finish(ctx, bg, args, ceiling=600)
    finally:
        if bg.poll() is None:
            driver.kill(bg.pid)
        snaps = w.stop()
    pid = acted["p3"]["pid"] if acted else None
    out = {"run": result_of(answer), "before": before, "acted": acted, "snaps": snaps,
           "p3_alive": driver.alive(pid) if pid else None,
           "rpt": rpt_lines(acted["p3"].get("profile")) if acted else [], "after": status_of(sh)}
    return out


def mistakes(ctx, project, sh, pids):
    before = status_of(sh)
    ids = []
    for cid in MISTAKES:
        ids += [cid, AFTER]
    w = Watch(ctx, project, pids)
    try:
        r = run(sh, ids)
    finally:
        snaps = w.stop()
    return {"run": r, "before": before, "snaps": snaps, "after": status_of(sh)}


def ded_part(ctx):
    project, sh = own_project(ctx)
    need_quiet(sh)
    pids, suspended, out = set(), [], {}
    try:
        out["session"] = start(sh, "ded", pids)
        out["ready"] = status_of(sh)
        stage(ctx, out, "join", lambda: join_late(ctx, project, sh, pids))
        stage(ctx, out, "tool", lambda: join_by_tool(ctx, project, pids))
        stage(ctx, out, "rejoin", lambda: leave_and_rejoin(ctx, project, sh, pids))
        stage(ctx, out, "leave_out", lambda: leave_out(ctx, project, sh, pids))
        stage(ctx, out, "hang", lambda: broken_join(ctx, project, sh, pids, suspended, "suspend"))
        stage(ctx, out, "exits", lambda: broken_join(ctx, project, sh, pids, suspended, "end"))
        stage(ctx, out, "mistakes", lambda: mistakes(ctx, project, sh, pids))
    finally:
        stop(ctx, sh, pids, suspended)
    return out


def sp_part(ctx):
    project, sh = own_project(ctx)
    need_quiet(sh)
    pids, out = set(), {}
    try:
        out["session"] = start(sh, "sp", pids)
        out["before"] = status_of(sh)
        out["run"] = run(sh, [SP_LEAVE, AFTER_SP])
        out["after"] = status_of(sh)
    finally:
        stop(ctx, sh, pids, [])
    return out


def ded(ctx, key):
    return got(once(ctx, "ded", ded_part), key)


# ================================================================ CFG-01: the late key in the project file

@check("CFG-01 MCH-01 a setup with late = 9 or late = \"one\" makes session start exit 3 naming late and its line, and starts no game", "start")
def cfg01_late(ctx):
    game = game_folder(ctx.project)
    before = driver.game_pids()
    for i, value in enumerate(("9", "\"one\"")):
        root = ctx.work / f"m11bad-{i}"
        if root.exists():
            shutil.rmtree(root)
        (root / "tests").mkdir(parents=True)
        (root / "tests" / "one.sqf").write_text("// @check CFG-01 one\n[1, \"==\", 1, \"one\"] call skd_fnc_expect;\n",
                                                 encoding="utf-8")
        (root / "shakedown.toml").write_text(
            f"game = '{game}'\n\n[mods.fixture]\npath = '{game / '@skd_fixture'}'\n\n"
            f"[setups.ded]\nmode = \"dedicated\"\nclients = 2\nlate = {value}\nmods = [\"fixture\"]\n", encoding="utf-8")
        sh = driver.Shakedown(ctx.sh.exe, root, ctx.log)
        a = sh("session", "start", "--setup", "ded", ceiling=120)
        said = message(a)
        assert a.code == 3, f"late = {value}: {a}, expected exit 3: {said[:300]}"
        assert re.search(r"\blate\b", said), f"late = {value}: the message does not name the key late: {said[:300]}"
        assert re.search(r"\bline 9\b", said) or ("line" in said and "9" in said), \
            f"late = {value}: the message does not name line 9, where late is: {said[:300]}"
    assert not driver.game_pids() - before, "a game process started"


# ================================================================ US-066

@check("US-066 MCH-01 INS-04 on a setup with clients = 2 and late = 1 session start starts server, p1, p2 and hc1 only: status and skd_fnc_machines list no p3", "start", repeat=True)
def us066_start(ctx):
    session = ded(ctx, "session")
    names = [m["name"] for m in session["machines"]]
    assert sorted(names) == sorted(START_MACHINES), f"session start gave the machines {names}"
    listed = sorted(machines(ded(ctx, "ready")))
    assert listed == sorted(START_MACHINES), f"status of the ready session lists {listed}"
    j = ded(ctx, "join")
    early = [t for t, st in j["snaps"] if "p3" in machines(st)]
    ex = execution(j["run"]["result"], JOIN)
    first = step_of(ex, "no late client before the join")
    assert first["state"] == "passed", f"skd_fnc_machines before the join: {first}, evidence {ex['evidence']}"
    assert early, "status never listed p3 during the check that joins it"


@check("US-066 MCH-01 OBS-06 skd_fnc_join of p3 returns once p3 is ready within its ceiling, and skd_fnc_expectAgree over server, p1, p2 and p3 on the crate's position and variable passes", "start", repeat=True)
def us066_join(ctx):
    j = ded(ctx, "join")
    ex = execution(j["run"]["result"], JOIN)
    passed(ex)
    s = step_of(ex, "join p3")
    assert s["state"] == "passed" and s["end"] is not None, f"the step join p3: {s}"
    assert s["duration_s"] < START_CEILING, f"join p3 took {s['duration_s']} s, its ceiling is {START_CEILING} s"
    ready = [t for t, m in seen(j["snaps"], "p3") if m["state"] == "ready"]
    ended = utc(s["end"]["utc"]).timestamp()
    assert ready, "status never showed p3 ready during the check"
    assert ready[0] <= ended + 1, \
        f"join p3 returned at {s['end']['utc']}, before status first showed p3 ready ({ready[0]:.2f})"
    ctx.log(f"US-066: join p3 took {s['duration_s']:.1f} s")


@check("US-066 INS-04 RES-01 during the check status lists p3 with start number 1 and its process id from its launch, and the result file records the game and mod versions p3 loaded, ending stopped", "start", repeat=True)
def us066_status(ctx):
    j = ded(ctx, "join")
    p3 = seen(j["snaps"], "p3")
    assert p3, "status never listed p3 during the check"
    states = [m["state"] for _, m in p3]
    assert all(m["start"] == 1 and isinstance(m.get("pid"), int) and m["pid"] > 0 for _, m in p3), \
        f"status listed p3 as {[(m['start'], m.get('pid')) for _, m in p3][:5]}"
    assert states[0] != "ready", f"status first listed p3 already ready ({states[:3]}), not from its launch"
    assert len(j["p3_pids"]) == 1, f"status gave p3 the process ids {j['p3_pids']}"
    result = j["run"]["result"]
    errors = ctx.validate(result)
    assert not errors, f"the result file is not valid: {errors[:5]}"
    entries = starts(result, "p3")
    assert len(entries) == 1, f"the result file lists p3 {len(entries)} times: {entries}"
    e = entries[0]
    assert e["start"] == 1 and e["kind"] == "client" and e["game_version"], f"p3 in the result file: {e}"
    mods = {m["mod"]: m for m in e["mods"]}
    assert "fixture" in mods and mods["fixture"]["version"], f"p3's mods in the result file: {e['mods']}"
    other = starts(result, "p1")[0]
    assert mods["fixture"]["version"] == {m["mod"]: m for m in other["mods"]}["fixture"]["version"], \
        f"p3 loaded fixture {mods['fixture']} and p1 {other['mods']}"
    assert e["end_state"] == "stopped", f"p3, made to leave by the reset, ends {e['end_state']}"


@check("US-066 RST-01 MCH-01 the next check starts without p3: skd_fnc_machines gives server, p1, p2 and hc1, p3's process has ended, and p1 and p2 keep their start numbers", "start", repeat=True)
def us066_after(ctx):
    j = ded(ctx, "join")
    passed(execution(j["run"]["result"], AFTER))
    assert j["p3_pids"] and not any(j["p3_alive"].values()), f"p3's process still runs: {j['p3_alive']}"
    after, before = machines(j["after"]), machines(j["before"])
    assert sorted(after) == sorted(START_MACHINES), f"status after the run lists {sorted(after)}"
    assert j["after"]["session"]["state"] == "ready", f"the session is {j['after']['session']['state']}"
    for name in ("server", "p1", "p2", "hc1"):
        assert after[name]["start"] == before[name]["start"] and after[name]["pid"] == before[name]["pid"], \
            f"{name} was start {before[name]['start']} pid {before[name]['pid']}, is {after[name]['start']} pid {after[name]['pid']}"


@check("US-066 AGT-01 the same check run through MCP with run_start ends pass, and no tool call of it lasts more than 60 s", "start", repeat=True)
def us066_tool(ctx):
    t = ded(ctx, "tool")
    assert t["status"]["state"] == "finished", f"run_status ended at {t['status']}"
    outcomes = t["result_tool"]["outcomes"]
    assert [o["check_id"] for o in outcomes] == [JOIN] and outcomes[0]["outcome"] == "pass", f"run_result: {outcomes}"
    ex = execution(t["result"], JOIN)
    passed(ex)
    slow = [(n, round(s, 1)) for n, s in t["calls"] if s > 60]
    assert not slow, f"tool calls over 60 s: {slow}"
    ctx.log(f"US-066: tool calls {[(n, round(s, 1)) for n, s in t['calls']]}")


# ================================================================ US-067

@check("US-067 MCH-01 skd_fnc_leave of p2 returns once the server's PlayerDisconnected fired, within its default ceiling of 20 s; p2's process has ended and status shows no p2", "start", repeat=True)
def us067_leave(ctx):
    r = ded(ctx, "rejoin")
    ex = execution(r["run"]["result"], REJOIN)
    s = step_of(ex, "leave p2")
    assert s["state"] == "passed" and s["end"] is not None, f"the step leave p2: {s}; {ex['reason']}"
    assert s["duration_s"] < LEAVE_CEILING, f"leave p2 took {s['duration_s']} s"
    rec = step_of(ex, "the server recorded p2's disconnect")
    assert rec["state"] == "passed", f"after the leave: {rec}; {ex['reason']}"
    assert r["gone"], "status never showed the session without p2 and p2's old process ended, within 120 s of the run's start"
    t, st = r["gone"]
    assert t >= utc(s["start"]["utc"]).timestamp() - 1, "p2 was gone before the leave started"
    assert st["session"]["state"] in ("ready", "running"), f"the session was {st['session']['state']} with p2 gone"
    assert r["go"] is not None and r["go"].code == 0, f"exec on the server to let the check go on: {r['go']}"
    ctx.log(f"US-067: leave p2 took {s['duration_s']:.2f} s")


@check("US-067 DM-05 the fixture's skdfx_disconnected on the server holds p2 once more, read with skd_fnc_expectOn in the same check; the check ends pass, no outcome is invalid and the session stays ready", "start", repeat=True)
def us067_record(ctx):
    r = ded(ctx, "rejoin")
    result = r["run"]["result"]
    passed(execution(result, REJOIN))
    assert all(c["outcome"] != "invalid" for c in result["checks"]), \
        f"outcomes {[(c['check_id'], c['outcome']) for c in result['checks']]}"
    states = {st["session"]["state"] for _, st in r["snaps"] if st.get("session")}
    assert states <= {"ready", "running"}, f"the session was {states} during the run, never broken"
    assert r["after"]["session"]["state"] == "ready", f"the session is {r['after']['session']['state']} after the run"


@check("US-067 MCH-01 skd_fnc_join of p2 in the same check brings p2 back ready with start number 2, and skd_fnc_player for p2 gives its new unit", "start", repeat=True)
def us067_join(ctx):
    r = ded(ctx, "rejoin")
    result = r["run"]["result"]
    ex = execution(result, REJOIN)
    passed(ex)
    s = step_of(ex, "join p2")
    assert s["state"] == "passed", f"the step join p2: {s}"
    assert step_of(ex, "p2 has a new unit")["state"] == "passed", f"{ex['reason']}"
    assert r["old"]["start"] == 1, f"p2 was start {r['old']['start']} before the leave"
    p2 = [m for _, m in seen(r["snaps"], "p2") if m["pid"] != r["old"]["pid"]]
    assert p2 and all(m["start"] == 2 for m in p2), f"status listed the new p2 as {[(m['start'], m['state']) for m in p2][:5]}"
    assert any(m["state"] == "ready" for m in p2), "status never showed the new p2 ready"
    entries = sorted((m["start"], m["end_state"]) for m in starts(result, "p2"))
    assert entries == [(1, "stopped"), (2, "ready")], f"the result file lists p2's starts as {entries}"
    attached = {e["label"]: e["value"] for e in ex["evidence"]["attached"]}
    assert attached.get("p2's unit after the join") != attached.get("p2's unit before the leave"), f"{attached}"


@check("US-067 RST-01 P-06 after a check that made p2 join again the next check starts with that same start of p2", "start", repeat=True)
def us067_same_start(ctx):
    r = ded(ctx, "rejoin")
    passed(execution(r["run"]["result"], AFTER))
    new = [m for _, m in seen(r["snaps"], "p2") if m["pid"] != r["old"]["pid"]]
    assert new, "status never listed the new p2"
    after = machines(r["after"])
    assert "p2" in after and after["p2"]["start"] == 2 and after["p2"]["pid"] == new[-1]["pid"] \
        and after["p2"]["state"] == "ready", f"after the run status gives p2 {after.get('p2')}, the check joined pid {new[-1]['pid']}"
    assert sorted(after) == sorted(START_MACHINES), f"status after the run lists {sorted(after)}"


@check("US-067 RST-01 MCH-01 after a check that left p2 out the next check starts with p2 joined again, ready, and the reset verified on every machine", "start", repeat=True)
def us067_restore(ctx):
    lo = ded(ctx, "leave_out")
    result = lo["run"]["result"]
    passed(execution(result, LEAVE_ONLY))
    after_ex = execution(result, AFTER)
    passed(after_ex)
    old = machines(lo["before"])["p2"]
    after = machines(lo["after"])
    assert sorted(after) == sorted(START_MACHINES), f"status after the run lists {sorted(after)}"
    assert after["p2"]["start"] == old["start"] + 1 and after["p2"]["state"] == "ready" \
        and after["p2"]["pid"] != old["pid"], f"p2 was {old['start']} pid {old['pid']}, is {after['p2']}"
    assert not driver.alive(old["pid"]), f"p2's old process {old['pid']} still runs"
    entries = sorted((m["start"], m["end_state"]) for m in starts(result, "p2"))
    assert entries == [(old["start"], "stopped"), (old["start"] + 1, "ready")], f"the result file lists p2's starts as {entries}"
    assert lo["after"]["session"]["state"] == "ready", f"the session is {lo['after']['session']['state']}"


@check("US-067 US-066 RES-02 each join and leave is a step of the check in the result file, titled join or leave and the machine, with its start, end and duration", "start", repeat=True)
def us067_steps(ctx):
    pairs = [(ded(ctx, "rejoin")["run"]["result"], REJOIN, ["leave p2", "join p2"]),
             (ded(ctx, "join")["run"]["result"], JOIN, ["join p3"]),
             (ded(ctx, "leave_out")["run"]["result"], LEAVE_ONLY, ["leave p2"])]
    for result, cid, titles in pairs:
        ex = execution(result, cid)
        got_titles = [s["path"][0] for s in ex["steps"] if len(s["path"]) == 1 and s["path"][0] in titles]
        assert got_titles == titles, f"{cid}: steps {[s['path'] for s in ex['steps']]}, expected {titles} in order"
        for title in titles:
            s = step_of(ex, title)
            assert s["start"] and s["end"] and s["duration_s"] is not None and s["duration_s"] >= 0, f"{cid}: {s}"
            span = (utc(s["end"]["utc"]) - utc(s["start"]["utc"])).total_seconds()
            assert abs(span - s["duration_s"]) <= 0.5, f"{cid} {title}: duration {s['duration_s']}, utc span {span}"


# ================================================================ US-068

def broken_join_ex(b):
    assert b["acted"], "status never listed p3 with a process id within 120 s of the run's start"
    return execution(b["run"]["result"], HANG)


@check("US-068 MCH-01 SES-04 with p3 suspended right after its launch skd_fnc_join of p3 with a ceiling of 30 s ends the check invalid machine-lost within 30 s plus the grace, naming p3 and holding its last RPT lines", "start", repeat=True)
def us068_hang(ctx):
    b = ded(ctx, "hang")
    ex = broken_join_ex(b)
    assert ex["outcome"] == "invalid" and ex["reason"]["kind"] == "machine-lost", f"{ex['outcome']} {ex['reason']}"
    assert ex["reason"].get("machine") == "p3" or re.search(r"\bp3\b", ex["reason"]["message"]), f"{ex['reason']}"
    s = step_of(ex, "join p3")
    took = (utc(ex["ended_at"]) - utc(s["start"]["utc"])).total_seconds()
    assert took <= 30 + GRACE + 5, f"the check ended {took:.1f} s after join p3 started"
    assert not any(st["path"] == ["the step after the join"] for st in ex["steps"]), "the step after the join ran"
    rpt = set(b["rpt"][-40:])
    held = [line for text in strings_in({"e": ex["evidence"], "r": ex["reason"]}) for line in text.splitlines()
            if line.strip() and line.strip() in rpt]
    assert held, f"neither the evidence nor the reason holds a line of p3's RPT ({len(b['rpt'])} lines); " \
                 f"evidence {str(ex['evidence'])[:400]}"
    ctx.log(f"US-068: the suspended join ended {took:.1f} s after it started; {len(held)} RPT lines in the evidence")


def after_broken(b):
    assert b["acted"], "status never listed p3 with a process id"
    assert b["p3_alive"] is False, f"p3's process {b['acted']['p3']['pid']} still runs after the check"
    after = b["after"]["session"]
    assert after["state"] == "ready", f"the session is {after['state']}: {after.get('reason')}"
    assert sorted(machines(b["after"])) == sorted(START_MACHINES), f"status lists {sorted(machines(b['after']))}"
    passed(execution(b["run"]["result"], AFTER))


@check("US-068 STP-02 after the suspended join no process of p3 runs, status shows the session ready with server, p1, p2 and hc1, and the next check runs with its own outcome", "start", repeat=True)
def us068_after_hang(ctx):
    after_broken(ded(ctx, "hang"))


@check("US-068 DM-05 when p3's process is ended by its id during its join the check ends invalid machine-lost at once, naming p3 and its exit code", "start", repeat=True)
def us068_exit(ctx):
    b = ded(ctx, "exits")
    ex = broken_join_ex(b)
    assert ex["outcome"] == "invalid" and ex["reason"]["kind"] == "machine-lost", f"{ex['outcome']} {ex['reason']}"
    msg = ex["reason"]["message"]
    assert ex["reason"].get("machine") == "p3" or re.search(r"\bp3\b", msg), f"{ex['reason']}"
    assert re.search(r"exit", msg, re.I) and re.search(r"\b1\b", msg), f"the reason does not name the exit code 1: {msg}"
    took = utc(ex["ended_at"]).timestamp() - b["acted"]["at"]
    assert took <= 5, f"the check ended {took:.1f} s after p3's process was ended"
    ctx.log(f"US-068: the check ended {took:.2f} s after p3's process was ended")


@check("US-068 STP-02 after the ended join no process of p3 runs, status shows the session ready with server, p1, p2 and hc1, and the next check runs with its own outcome", "start", repeat=True)
def us068_after_exit(ctx):
    after_broken(ded(ctx, "exits"))


# ================================================================ US-069

def segment(msg, word):
    m = re.search(rf"can {word}[^:]*:\s*([^;]*?)(?:;|\.\s|\.$|$)", msg)
    return set(re.findall(r"\b(?:p\d+|hc\d+|server)\b", m.group(1))) if m else None


def mistake_check(cid):
    machine, can_join, can_leave = MISTAKES[cid]
    fn = "leave" if "leave" in cid else "join"

    @check(f"US-069 DM-04 {cid}: [\"{machine}\"] call skd_fnc_{fn} ends the check fail check-error naming {machine} and the machines that can join and leave then", "start", repeat=True)
    def one(ctx):
        result = ded(ctx, "mistakes")["run"]["result"]
        ex = execution(result, cid)
        assert ex["outcome"] == "fail" and ex["reason"]["kind"] == "check-error", f"{ex['outcome']} {ex['reason']}"
        msg = ex["reason"]["message"]
        assert re.search(rf"\b{machine}\b", msg), f"the message does not name {machine}: {msg}"
        j, lv = segment(msg, "join"), segment(msg, "leave")
        assert j == can_join and lv == can_leave, \
            f"the message names {j} as able to join and {lv} as able to leave, expected {can_join} and {can_leave}: {msg}"
        assert not any("the step after" in s["path"][0] for s in ex["steps"]), "the step after the call ran"
    return one


for _cid in MISTAKES:
    mistake_check(_cid)


@check("US-069 DM-04 in singleplayer [\"p1\"] call skd_fnc_leave ends the check fail check-error saying joining and leaving needs a dedicated session", "start", repeat=True)
def us069_sp(ctx):
    sp = once(ctx, "sp", sp_part)
    ex = execution(sp["run"]["result"], SP_LEAVE)
    assert ex["outcome"] == "fail" and ex["reason"]["kind"] == "check-error", f"{ex['outcome']} {ex['reason']}"
    assert "dedicated" in ex["reason"]["message"], f"the message does not say a dedicated session: {ex['reason']['message']}"


@check("US-069 after each wrong join or leave the next check passes and the session is ready with the machines it had before the check, on the dedicated session and in singleplayer", "start", repeat=True)
def us069_after(ctx):
    m = ded(ctx, "mistakes")
    result = m["run"]["result"]
    afters = [c for c in result["checks"] if c["check_id"] == AFTER]
    assert len(afters) == len(MISTAKES) and all(c["outcome"] == "pass" for c in afters), \
        f"the checks after: {[(c['repeat'], c['outcome'], c['reason']) for c in afters]}"
    assert all(c["outcome"] != "invalid" for c in result["checks"]), \
        f"outcomes {[(c['check_id'], c['outcome']) for c in result['checks']]}"
    before, after = machines(m["before"]), machines(m["after"])
    assert sorted(after) == sorted(START_MACHINES) and m["after"]["session"]["state"] == "ready", \
        f"status after: {m['after']['session']['state']} {sorted(after)}"
    for name in ("server", "p1", "hc1"):
        assert after[name]["start"] == before[name]["start"], f"{name} went from start {before[name]['start']} to {after[name]['start']}"
    states = {st["session"]["state"] for _, st in m["snaps"] if st.get("session")}
    assert states <= {"ready", "running"}, f"the session was {states} during the run, never broken"
    sp = once(ctx, "sp", sp_part)
    passed(execution(sp["run"]["result"], AFTER_SP))
    b, a = machines(sp["before"]), machines(sp["after"])
    assert sorted(a) == ["p1"] and a["p1"]["start"] == b["p1"]["start"] and sp["after"]["session"]["state"] == "ready", \
        f"singleplayer after: {sp['after']['session']['state']} {a}"
