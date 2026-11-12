"""The checks of M9 Invalid, not failed: US-010, US-015, US-016, US-041, US-043, US-044, US-045 and US-059.

Every part breaks the session it runs in, so each runs in a session of its own, once per repeat
(`--repeat`), with no session running before it, in the project `m09` in the work folder: the checks of
`acceptance/m09/` and two setups loading `@skd_fixture` alone, `ded` (a dedicated server with two clients,
as the M9 demo has it) and `sp` (singleplayer). Per repeat:

- `ded` 1, a stall: US-044 on the ready session (a failed check inspected by command and by tool, the
  four questions asked during a running check, an error raised by `exec` outside any check), then p2
  suspended by its id while a check waits on it (US-015), then the broken session asked (US-044, US-045),
  then p1 suspended and inspected (US-045).
- `ded` 2, a crash: p2 crashed by `skdfx_fnc_crash` while a check waits on it (US-016, US-043, US-041).
- `ded` 3, an assertion first: an assertion fails and p2 crashes 0.5 s later (US-041).
- `ded` 4, an end from outside: p2 ended with `taskkill /F` by its id while a check waits on it (US-016,
  US-043, the demo).
- `sp`: a reset of the ready session, the three outcomes and the exit codes (US-041), p1's clock held
  (US-059), a cleanup that never ends (US-010), the broken session reloaded, and `shakedown reset` that
  fails.

A suspended process is resumed by its id before its session stops; every game process a part started is
stopped by its id if it outlives the stop. A part whose game start is lost to `Error 20 reading file` of a
base-game file is started again, at most three times in all.
"""

import json
import re
import shutil
import subprocess
import time
import traceback
from pathlib import Path

from . import driver, win
from .core import check
from .m06 import finish, text_of, utc
from .m08 import game_folder, steam_running
from .mcp import Mcp, structured

ACC = Path(__file__).resolve().parent.parent
WAIT, AFTER = "US-015-wait-on-p2", "after"
CRASH, FAIL_LOST = "US-016-crash", "US-041-fail-then-lost"
FAILS, SLOW = "US-044-fails", "US-044-slow"
HOLD, FAIL_FIRST = "US-059-hold", "US-059-fail-first"
NEVER = "US-010-cleanup-never-ends"
WAIT_CEILING = 60          # s, the in-game ceiling of the waits on p2
GRACE = 10                 # s, OBS-03
HEARTBEAT = 10             # s, HLT-02: no heartbeat for 10 s is lost
CRASH_CODE = 3221225477    # 0xC0000005, fixture-mod.md
WAIT_STEP = "wait on p2 for a condition that never holds"
SLOW_STEP = "wait for the flag the server sets 15 s later"
VOLATILE = {"time", "tick", "fps", "frame", "last_answer_at", "heartbeat", "scripts", "utc"}


# ================================================================ the project and the parts

def keep(ctx):
    if not hasattr(ctx, "m09_keep"):
        ctx.m09_keep = {}
    return ctx.m09_keep


def own_project(ctx):
    store = keep(ctx)
    if "project" in store:
        return store["project"]
    game = game_folder(ctx.project)
    project = ctx.work / "m09"
    if project.exists():
        shutil.rmtree(project)
    shutil.copytree(ACC / "m09", project / "checks")
    (project / "shakedown.toml").write_text(
        f"game = '{game}'\n\n[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n\n"
        "[setups.ded]\nmode = \"dedicated\"\nclients = 2\nheadless = 0\nmods = [\"fixture\"]\n\n"
        "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n", encoding="utf-8")
    (project / ".mcp.json").write_text(json.dumps(
        {"mcpServers": {"shakedown": {"command": "shakedown", "args": ["mcp"], "timeout": 70000}}}, indent=2),
        encoding="utf-8")
    sh = driver.Shakedown(ctx.sh.exe, project, ctx.log)
    store["project"] = (project, sh)
    return store["project"]


def machine_fault(ctx, since):
    root = ctx.work / "m09" / ".shakedown" / "sessions"
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
                ctx.log(f"M9 part {k}: {traceback.format_exc()}")
                store[k] = e
                fault = machine_fault(ctx, began)
                if not fault or attempt == 3:
                    break
                ctx.log(f"M9 part {k}: a game process could not read a base-game file ({fault}); the machine, "
                        f"not the framework. Starting the part again, attempt {attempt + 1} of 3")
    value = store[k]
    if isinstance(value, Exception):
        raise AssertionError(f"the M9 part {key} failed: {value!r}")
    return value


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


def start(ctx, sh, setup, pids):
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
            ctx.log(f"M9: resuming {pid}: {e!r}")
    try:
        if status_of(sh).get("session"):
            sh("session", "stop", "--json", ceiling=180)
    except Exception as e:  # noqa: BLE001
        ctx.log(f"M9: status or session stop after the part: {e!r}")
    for pid in set(pids):
        if driver.alive(pid) and driver.wait_gone(pid, 30) is None:
            ctx.log(f"M9: game process {pid} still runs; stopping it by id")
            driver.kill(pid)
    # The suite waits for this runner to exit by itself at its end, and stops it by id after 75 s.
    ctx.runner_pid = driver.runner_pid() or ctx.runner_pid


def result_of(answer):
    out = answer.json()
    folder = Path(out["result_file"]).parent
    return {"answer": answer, "out": out, "result": json.loads((folder / "result.json").read_text(encoding="utf-8")),
            "folder": folder}


def run(sh, ids, ceiling=300):
    return result_of(sh("run", *ids, "--json", ceiling=ceiling))


def timed(sh, *args, ceiling=60):
    began = time.monotonic()
    answer = sh(*args, ceiling=ceiling)
    return answer, time.monotonic() - began


def inspect(sh, machine, ceiling=60):
    answer, took = timed(sh, "inspect", "--machine", machine, "--json", ceiling=ceiling)
    return {"answer": answer, "json": answer.json() if answer.stdout.strip() else None, "s": took}


def poll_step(ctx, sh, bg, check_id, step, ceiling=120):
    """Asks the lead with inspect, twice a second, until it names this check and this step running."""
    for _ in range(int(ceiling * 2)):
        if bg.poll() is not None:
            return None
        try:
            i = sh("inspect", "--machine", "server", "--json", ceiling=30).json()
        except AssertionError:
            i = {}
        if i.get("check_id") == check_id and step in (i.get("step_path") or []):
            return i
        time.sleep(0.5)
    return None


def newest_rpt(profile):
    rpts = sorted(Path(profile).rglob("*.rpt"), key=lambda p: p.stat().st_mtime)
    return rpts[-1] if rpts else None


def dumps_in(profile):
    return sorted(p.name for p in Path(profile).rglob("*") if p.suffix.lower() in (".mdmp", ".bidmp"))


def background_run(ctx, sh, ids, check_id, step, act):
    """Runs ids in the background; once the lead names check_id and step running, calls act().
    Returns (the run as result_of gives it, what act returned, the inspect that found the step)."""
    args = ("run", *ids, "--json")
    bg = sh.start(*args)
    found = poll_step(ctx, sh, bg, check_id, step)
    acted = act() if found else None
    answer = finish(ctx, bg, args, ceiling=300)
    assert found, f"the lead never named {check_id} at the step {step!r} running; the run: {answer}"
    return result_of(answer), acted, found


# ---------------------------------------------------------------- ded 1: a stall, and asking the game

def ded_stall(ctx):
    project, sh = own_project(ctx)
    need_quiet(sh)
    pids, suspended, out = set(), [], {}
    try:
        out["session"] = start(ctx, sh, "ded", pids)
        out["ready"] = status_of(sh)
        # US-044 1: a failed check, then inspect by command and by tool.
        out["fails"] = run(sh, [FAILS])
        out["inspect_p1"] = inspect(sh, "p1")
        with Mcp(project, ctx.install, ctx.log) as m:
            out["inspect_p1_tool"], _ = m.call("inspect", {"machine": "p1"}, ceiling=70)
        # US-044 2: the same slow check with no questions, then with the four questions.
        out["slow_base"] = run(sh, [SLOW])

        def ask():
            asked = {"inspect_p1": inspect(sh, "p1"), "inspect_server": inspect(sh, "server")}
            asked["exec"] = timed(sh, "exec", "--machine", "p1", "--code", "1 + 1", "--json")
            asked["logs"] = timed(sh, "logs", "--machine", "p1", "--last", "20", "--json")
            asked["errors"] = timed(sh, "errors", "--json")
            asked["status"] = status_of(sh)
            return asked
        out["slow"], out["asked"], out["slow_found"] = background_run(ctx, sh, [SLOW], SLOW, SLOW_STEP, ask)
        # US-044 3: a script error raised by exec, outside any check.
        out["exec_err_at"] = time.time()
        out["exec_err"] = sh("exec", "--machine", "p1", "--code", "[] call skdfx_fnc_throw; 1", "--json")
        out["errors_after_exec"] = sh("errors", "--json")
        # US-015: p2 suspended while the check waits on it.
        p2 = machines(out["ready"])["p2"]["pid"]

        def suspend_p2():
            win.suspend(p2)
            suspended.append(p2)
            ctx.log(f"M9: suspended p2, pid {p2}, by its id")
            return time.time()
        out["stall"], out["suspended_at"], _ = background_run(ctx, sh, [WAIT, AFTER], WAIT, WAIT_STEP, suspend_p2)
        # The broken session, with p2 still suspended.
        out["broken"] = status_of(sh)
        out["alive"] = {name: driver.alive(m["pid"]) for name, m in machines(out["ready"]).items()}
        out["inspect_p2"] = inspect(sh, "p2")
        out["logs_p2"] = timed(sh, "logs", "--machine", "p2", "--last", "20", "--json")
        for name in ("server", "p1"):
            out[f"broken_inspect_{name}"] = inspect(sh, name)
            out[f"broken_logs_{name}"] = timed(sh, "logs", "--machine", name, "--last", "20", "--json")
        out["broken_errors"] = timed(sh, "errors", "--json")
        out["broken_reset"] = sh("reset", "--json", ceiling=60)
        # US-045: p1 suspended on the broken session, then inspected.
        p1 = machines(out["ready"])["p1"]["pid"]
        win.suspend(p1)
        suspended.append(p1)
        ctx.log(f"M9: suspended p1, pid {p1}, by its id")
        out["inspect_p1_fresh"] = inspect(sh, "p1", ceiling=90)
        out["inspect_server_last"] = inspect(sh, "server")
        out["status_last"] = status_of(sh)
        ctx.log(f"M9 ded stall: run {out['stall']['answer']}; p2 inspect {out['inspect_p2']['s']:.1f} s; "
                f"p1 fresh inspect {out['inspect_p1_fresh']['s']:.1f} s")
        return out
    finally:
        stop(ctx, sh, pids, suspended)


# ---------------------------------------------------------------- ded 2, 3, 4: p2 ends

def ded_end(ctx, how):
    """how: crash (skdfx_fnc_crash while the check waits), first (an assertion fails, then the crash),
    kill (taskkill /F of p2 by its id while the check waits)."""
    project, sh = own_project(ctx)
    need_quiet(sh)
    pids, out = set(), {}
    try:
        out["session"] = start(ctx, sh, "ded", pids)
        out["ready"] = status_of(sh)
        p2 = machines(out["ready"])["p2"]
        out["profile"] = p2["profile"]
        out["dumps_before"] = dumps_in(p2["profile"])
        if how == "crash":
            out["run"] = run(sh, [CRASH, AFTER])
        elif how == "first":
            out["run"] = run(sh, [FAIL_LOST, AFTER])
        else:
            def kill():
                subprocess.run(["taskkill", "/PID", str(p2["pid"]), "/F"], capture_output=True, timeout=30)
                ctx.log(f"M9: ended p2, pid {p2['pid']}, by its id with taskkill /F")
                return time.time()
            out["run"], out["killed_at"], _ = background_run(ctx, sh, [WAIT, AFTER], WAIT, WAIT_STEP, kill)
        out["broken"] = status_of(sh)
        out["inspect_p2"] = inspect(sh, "p2")
        out["dumps_after"] = dumps_in(p2["profile"])
        ctx.log(f"M9 ded {how}: run {out['run']['answer']}; p2 {json.dumps(next((m for m in out['run']['result']['session']['machines'] if m['name'] == 'p2'), None))[:600]}")
    finally:
        stop(ctx, sh, pids, [])
    # The session stopped, so p2's RPT in its profile folder is redacted (LOG-02) like the excerpt.
    rpt = newest_rpt(out["profile"])
    out["rpt_lines"] = rpt.read_text(encoding="utf-8", errors="replace").splitlines() if rpt else None
    return out


# ---------------------------------------------------------------- sp: outcomes, the clock, the reset

def health_poll(sh, alive, ceiling, until=None):
    """Polls status twice a second while alive() holds, at most `ceiling` s: [(time, p1's health)]."""
    seen = []
    for _ in range(int(ceiling * 2)):
        if not alive():
            break
        try:
            h = machines(status_of(sh)).get("p1", {}).get("health")
        except AssertionError:
            h = None
        seen.append((time.time(), h))
        if until and until(seen):
            break
        time.sleep(0.5)
    return seen


def sp_part(ctx):
    project, sh = own_project(ctx)
    need_quiet(sh)
    pids, out = set(), {}
    try:
        out["session"] = start(ctx, sh, "sp", pids)
        out["reset_ready"] = sh("reset", "--json", ceiling=60)
        # US-041
        out["three"] = run(sh, [AFTER, "US-041-fail", "US-041-invalid"])
        out["only_invalid"] = run(sh, ["US-041-invalid"])
        out["only_pass"] = run(sh, [AFTER])
        # US-059 1 and 2: the clock held during a check, status watched while it runs.
        args = ("run", HOLD, AFTER, "--json")
        bg = sh.start(*args)
        out["hold_health"] = health_poll(sh, lambda: bg.poll() is None, 120)
        out["hold"] = result_of(finish(ctx, bg, args, ceiling=120))
        # US-059 3: an assertion fails, then the clock is held.
        out["first"] = run(sh, [FAIL_FIRST])
        out["first_health"] = health_poll(
            sh, lambda: True, 40,
            until=lambda seen: any(h and h.get("flag") for _, h in seen) and seen[-1][1] and not seen[-1][1].get("flag"))
        # US-010: a cleanup that never ends.
        out["never"] = run(sh, [NEVER, AFTER, AFTER])
        out["never_status"] = status_of(sh)
        out["never_reset"] = sh("reset", "--json", ceiling=60)
        out["reload"] = sh("reload", "--json", ceiling=400)
        out["reloaded"] = status_of(sh)
        pids |= {m["pid"] for m in machines(out["reloaded"]).values() if m.get("pid")}
        out["never2"] = run(sh, [NEVER])
        out["reset_fails"] = timed(sh, "reset", "--json", ceiling=120)
        out["reset_fails_status"] = status_of(sh)
        ctx.log(f"M9 sp: hold {out['hold']['answer']}; reset that fails {out['reset_fails'][0]}: "
                f"{text_of(out['reset_fails'][0])[:600]}")
        return out
    finally:
        stop(ctx, sh, pids, [])


def part(ctx, key):
    fns = {"stall": ded_stall, "crash": lambda c: ded_end(c, "crash"), "first": lambda c: ded_end(c, "first"),
           "kill": lambda c: ded_end(c, "kill"), "sp": sp_part}
    return once(ctx, key, fns[key])


# ================================================================ judging helpers

def ex(result, check_id, repeat=1):
    found = [e for e in result["checks"] if e["check_id"] == check_id and e.get("repeat", 1) == repeat]
    assert len(found) == 1, f"the result holds {len(found)} executions of {check_id} #{repeat}: " \
                            f"{[(e['check_id'], e.get('repeat')) for e in result['checks']]}"
    return found[0]


def kinds(result):
    return [(e["check_id"], e["outcome"], (e["reason"] or {}).get("kind")) for e in result["checks"]]


def machine_in(result, name):
    found = [m for m in result["session"]["machines"] if m["name"] == name]
    assert found, f"the result file lists no machine {name}: {[m['name'] for m in result['session']['machines']]}"
    return found[-1]


def valid(ctx, r, what):
    errors = ctx.validate(r["result"])
    assert not errors, f"{what}: the result file is not valid against result.schema.json: {errors[:5]}"


def lost_check(r, check_id, machine, what):
    e = ex(r["result"], check_id)
    reason = e["reason"] or {}
    assert (e["outcome"], reason.get("kind")) == ("invalid", "machine-lost"), \
        f"{what}: {check_id} ended {e['outcome']} / {reason.get('kind')}: {reason.get('message')}"
    assert reason.get("machine") == machine, f"{what}: reason.machine {reason.get('machine')!r}, expected {machine}"
    return e


def broken_after(r, what):
    e = ex(r["result"], AFTER)
    k = (e["outcome"], (e["reason"] or {}).get("kind"))
    assert k == ("invalid", "session-broken"), f"{what}: the check after it ended {k}: {(e['reason'] or {}).get('message')}"


def mask(value):
    if isinstance(value, dict):
        return {k: ("<volatile>" if k in VOLATILE else mask(v)) for k, v in value.items()}
    if isinstance(value, list):
        return [mask(v) for v in value]
    if isinstance(value, float):
        return round(value, 2)
    return value


def answered(item, what):
    a = item["answer"] if isinstance(item, dict) else item[0] if isinstance(item, tuple) else item
    assert a.code == 0, f"{what}: {a}: {text_of(a)[:400]}"
    return a.json()


def m9check(name):
    def wrap(fn):
        return check(name, "start", repeat=True)(fn)
    return wrap


# ================================================================ US-015

@m9check("US-015 OBS-03 DM-07 P-07 p2 suspended by its id while a check waits on it: the check ends invalid machine-lost, reason.machine p2, a message that it stopped answering and what it was asked, within the wait's 60 s ceiling plus 10 s")
def us015_invalid(ctx):
    v = part(ctx, "stall")
    e = lost_check(v["stall"], WAIT, "p2", "the stalled check")
    msg = e["reason"]["message"]
    low = msg.lower()
    assert "stopped answering" in low, f"the message does not say p2 stopped answering: {msg!r}"
    assert "asked" in low and "false" in low, f"the message does not say what p2 was asked (to poll `false`): {msg!r}"
    took = utc(e["ended_at"]) - v["suspended_at"]
    ctx.log(f"M9 US-015: the check ended {took:.1f} s after the suspension; reason.at "
            f"{utc(e['reason']['at']) - v['suspended_at']:.1f} s after it")
    assert 0 < took <= WAIT_CEILING + GRACE, f"the check ended {took:.1f} s after p2 was suspended, limit {WAIT_CEILING + GRACE} s"


@m9check("US-015 HLT-02 P-07 the suspended p2 is lost about 10 s after its last heartbeat: reason.at within 15 s of the suspension")
def us015_heartbeat(ctx):
    v = part(ctx, "stall")
    e = lost_check(v["stall"], WAIT, "p2", "the stalled check")
    took = utc(e["reason"]["at"]) - v["suspended_at"]
    assert HEARTBEAT - 2 <= took <= HEARTBEAT + 5, f"p2 was lost {took:.1f} s after it was suspended; HLT-02 gives 10 s"


@m9check("US-015 RES-01 RES-05 the check after the stall is invalid session-broken, the run exits 2, and the valid result file lists p2 with end_state lost, lost.reason no-answer, exit_code null and lost.at")
def us015_result(ctx):
    v = part(ctx, "stall")
    r = v["stall"]
    broken_after(r, "the stall")
    assert r["answer"].code == 2, f"{r['answer']}, expected exit 2"
    valid(ctx, r, "the stall")
    p2 = machine_in(r["result"], "p2")
    lost = p2.get("lost") or {}
    assert p2["end_state"] == "lost", f"p2 end_state {p2['end_state']}"
    assert lost.get("reason") == "no-answer" and lost.get("exit_code") is None and lost.get("at"), f"p2 lost {lost}"


@m9check("US-015 INS-04 P-07 status then shows the session broken with reason machine p2, p2 lost, and every game process of the session still running")
def us015_status(ctx):
    v = part(ctx, "stall")
    s = v["broken"]["session"]
    assert s["state"] == "broken", f"the session is {s['state']}"
    reason = s.get("reason") or {}
    assert reason.get("machine") == "p2" and reason.get("message"), f"the session's reason: {str(reason)[:400]}"
    e = ex(v["stall"]["result"], WAIT)
    assert "p2" in reason["message"], f"the session's reason does not name p2: {reason['message']!r}"
    assert isinstance(reason.get("rpt_tail"), list), f"the session's reason has no rpt_tail: {str(reason)[:300]}"
    assert machines(v["broken"])["p2"]["state"] == "lost", f"p2 is {machines(v['broken'])['p2']['state']}"
    gone = [n for n, a in v["alive"].items() if not a]
    assert not gone, f"game processes gone after the stall: {gone}"
    ctx.log(f"M9 US-015: the check's message {e['reason']['message'][:300]!r}; the session's {reason['message'][:300]!r}")


# ================================================================ US-045

@m9check("US-045 INS-01 with p2 suspended and lost, inspect --machine p2 --json returns within 20 s with machine.state lost and every other field null, and status shows p2 lost with its last answer time and the session broken")
def us045_inspect_p2(ctx):
    v = part(ctx, "stall")
    i = v["inspect_p2"]
    assert i["s"] <= 20, f"inspect of p2 took {i['s']:.1f} s"
    j = answered(i, "inspect of p2")
    assert (j.get("machine") or {}).get("state") == "lost", f"inspect gives p2 {str(j.get('machine'))[:300]}"
    others = {k: val for k, val in j.items() if k != "machine" and val is not None}
    assert not others, f"inspect of the lost p2 gives fields that are not null: {sorted(others)}"
    m = machines(v["broken"])["p2"]
    assert m["state"] == "lost" and m.get("last_answer_at"), f"status gives p2 {m['state']}, last_answer_at {m.get('last_answer_at')}"
    assert v["broken"]["session"]["state"] == "broken", f"the session is {v['broken']['session']['state']}"


@m9check("US-045 INS-01 P-07 p1 suspended on the broken session: inspect --machine p1 returns within 20 s with machine.state lost, status then shows p1 lost, and inspect of the server still answers")
def us045_fresh(ctx):
    v = part(ctx, "stall")
    i = v["inspect_p1_fresh"]
    ctx.log(f"M9 US-045: inspect of the freshly suspended p1 returned in {i['s']:.1f} s, exit {i['answer'].code}")
    assert i["s"] <= 20, f"inspect of the suspended p1 took {i['s']:.1f} s"
    j = answered(i, "inspect of the suspended p1")
    assert (j.get("machine") or {}).get("state") == "lost", f"inspect gives p1 {str(j.get('machine'))[:300]}"
    assert machines(v["status_last"])["p1"]["state"] == "lost", f"status gives p1 {machines(v['status_last'])['p1']['state']}"
    s = answered(v["inspect_server_last"], "inspect of the server")
    assert (s.get("machine") or {}).get("state") == "ready", f"inspect gives the server {str(s.get('machine'))[:300]}"


@m9check("US-045 LOG-03 logs --machine p2 still returns p2's last lines while it is lost, and inspect of every other machine answers")
def us045_logs(ctx):
    v = part(ctx, "stall")
    a, took = v["logs_p2"]
    j = answered((a, took), "logs of p2")
    assert j.get("lines"), f"logs of the lost p2 gives no lines: {str(j)[:300]}"
    assert took < 20, f"logs of p2 took {took:.1f} s"
    for name in ("server", "p1"):
        k = answered(v[f"broken_inspect_{name}"], f"inspect of {name}")
        assert (k.get("machine") or {}).get("state") == "ready", f"inspect gives {name} {str(k.get('machine'))[:200]}"


# ================================================================ US-044

def us044_nearby(j, ids):
    near = {o.get("net_id"): o for o in j.get("nearby") or []}
    missing = [i for i in ids if i not in near]
    assert not missing, f"inspect of p1 lists no object with net id {missing}; it lists {sorted(near)[:20]}"
    for i in ids:
        o = near[i]
        assert o.get("class") in ("Land_WoodenBox_F", "Box_NATO_Ammo_F"), f"object {i} has class {o.get('class')!r}"
        assert isinstance(o.get("distance"), (int, float)) and o["distance"] <= 50, f"object {i} at distance {o.get('distance')}"


@m9check("US-044 INS-01 RST-07 after a check fails, inspect --machine p1 --json returns p1's player with position, direction, damage and vehicle, and both boxes the check created within 50 m with class, net id and distance")
def us044_after_fail(ctx):
    v = part(ctx, "stall")
    e = ex(v["fails"]["result"], FAILS)
    assert e["outcome"] == "fail", f"{FAILS} ended {e['outcome']}: {e['reason']}"
    ids = next((a["value"] for a in e["evidence"]["attached"] if a["label"] == "created"), None)
    assert isinstance(ids, list) and len(ids) == 2, f"the check attached no two net ids: {e['evidence']['attached']}"
    j = answered(v["inspect_p1"], "inspect of p1")
    p = j.get("player") or {}
    for key in ("class", "position", "direction", "damage", "vehicle", "alive", "animation"):
        assert key in p, f"inspect's player has no {key}: {sorted(p)}"
    assert isinstance(p["position"], list) and len(p["position"]) == 3, f"the player's position {p['position']}"
    assert isinstance(p["direction"], (int, float)) and isinstance(p["damage"], (int, float)), f"player {p}"
    assert (j.get("machine") or {}).get("name") == "p1", f"inspect's machine {str(j.get('machine'))[:200]}"
    us044_nearby(j, ids)


@m9check("US-044 INS-01 AGT-01 the MCP tool inspect of p1 after the failed check returns what inspect --json returns, clocks and running scripts aside")
def us044_tool(ctx):
    v = part(ctx, "stall")
    tool = structured(v["inspect_p1_tool"])
    cli = answered(v["inspect_p1"], "inspect of p1")
    a, b = mask(cli), mask(tool)
    assert a == b, f"inspect by command and by tool differ: {json.dumps(a)[:600]} vs {json.dumps(b)[:600]}"


@m9check("US-044 INS-03 P-11 inspect, exec, logs and errors asked while a check runs each answer within 10 s, and inspect names the check and the step running")
def us044_during(ctx):
    v = part(ctx, "stall")
    q = v["asked"]
    took = {"inspect p1": q["inspect_p1"]["s"], "inspect server": q["inspect_server"]["s"], "exec": q["exec"][1],
            "logs": q["logs"][1], "errors": q["errors"][1]}
    ctx.log(f"M9 US-044 during a check: {json.dumps({k: round(s, 2) for k, s in took.items()})}")
    slow = {k: s for k, s in took.items() if s >= 10}
    assert not slow, f"answers that took 10 s or more: {slow}"
    for k in ("exec", "logs", "errors"):
        answered(q[k], k)
    x = q["exec"][0].json()
    assert (x["results"].get("p1") or {}).get("value") == 2, f"exec gave {str(x)[:300]}"
    assert (q["status"].get("run") or {}).get("current_check") == SLOW, f"status gave the run {q['status'].get('run')}"
    j = answered(q["inspect_server"], "inspect of the server")
    assert j.get("check_id") == SLOW and j.get("run_id"), f"the lead names run {j.get('run_id')}, check {j.get('check_id')}"
    assert SLOW_STEP in (j.get("step_path") or []), f"the lead names the step {j.get('step_path')}"


@m9check("US-044 INS-01 inspect of p1 while p1 runs the wait of the check's step names the run, the check and that step")
def us044_during_p1(ctx):
    v = part(ctx, "stall")
    j = answered(v["asked"]["inspect_p1"], "inspect of p1")
    ctx.log(f"M9 US-044: inspect of p1 during the check: run {j.get('run_id')}, check {j.get('check_id')}, step {j.get('step_path')}")
    assert j.get("check_id") == SLOW and j.get("run_id"), f"p1 names run {j.get('run_id')}, check {j.get('check_id')}"
    assert SLOW_STEP in (j.get("step_path") or []), f"p1 names the step {j.get('step_path')}, expected {SLOW_STEP!r}"


@m9check("US-044 the check asked about while it ran ends as the same check with no questions asked: pass")
def us044_same(ctx):
    v = part(ctx, "stall")
    base, asked = ex(v["slow_base"]["result"], SLOW), ex(v["slow"]["result"], SLOW)
    assert base["outcome"] == "pass", f"with no questions: {base['outcome']} {base['reason']}"
    assert asked["outcome"] == base["outcome"], f"with questions: {asked['outcome']} {asked['reason']}"
    assert v["slow"]["answer"].code == v["slow_base"]["answer"].code == 0, \
        f"exit {v['slow_base']['answer'].code} then {v['slow']['answer'].code}"


@m9check("US-044 ERR-05 shakedown errors returns the script error an exec raised outside any check, with machine p1, check_id null and its time")
def us044_errors(ctx):
    v = part(ctx, "stall")
    x = v["exec_err"]
    assert x.code == 0, f"{x}: {text_of(x)[:300]}"
    j = answered(v["errors_after_exec"], "errors")
    found = [e for e in j.get("errors", []) if "_skdfx_undefined" in e.get("text", "")
             and e.get("machine") == "p1" and e.get("check_id") is None]
    assert found, f"errors lists no error of p1 with _skdfx_undefined and check_id null: {str(j)[:600]}"
    at = utc(found[-1]["at"]["utc"])
    assert abs(at - v["exec_err_at"]) < 30, f"the error's time {found[-1]['at']['utc']} is not that of the exec"


@m9check("US-044 INS-03 on the broken session inspect, logs and errors still answer for the server and p1, the machines that answer")
def us044_broken(ctx):
    v = part(ctx, "stall")
    assert v["broken"]["session"]["state"] == "broken", f"the session is {v['broken']['session']['state']}"
    for name in ("server", "p1"):
        j = answered(v[f"broken_inspect_{name}"], f"inspect of {name}")
        assert (j.get("machine") or {}).get("state") == "ready" and j.get("frame") is not None, \
            f"inspect of {name} on the broken session: {str(j)[:300]}"
        lj = answered(v[f"broken_logs_{name}"], f"logs of {name}")
        assert lj.get("lines"), f"logs of {name} gives no lines"
    answered(v["broken_errors"], "errors")


# ================================================================ US-016 and US-043

@m9check("US-016 P-09 p2 crashed by skdfx_fnc_crash while the check waits on it: invalid machine-lost, reason.machine p2, a message giving its exit code")
def us016_crash(ctx):
    v = part(ctx, "crash")
    e = lost_check(v["run"], CRASH, "p2", "the crash")
    msg = e["reason"]["message"]
    assert str(CRASH_CODE) in msg or "0xc0000005" in msg.lower(), f"the message gives no exit code {CRASH_CODE}: {msg!r}"


@m9check("US-016 RES-01 RES-05 the result file of the crash is valid and lists p2 with end_state lost, lost.reason exited, exit_code 3221225477, lost.at, and its crash dumps under dumps/p2/, an .mdmp and a .bidmp, each in the run folder")
def us016_result(ctx):
    v = part(ctx, "crash")
    r = v["run"]
    valid(ctx, r, "the crash")
    p2 = machine_in(r["result"], "p2")
    lost = p2.get("lost") or {}
    assert p2["end_state"] == "lost" and lost.get("reason") == "exited", f"p2 {p2['end_state']}, lost {lost}"
    assert lost.get("exit_code") == CRASH_CODE, f"p2 exit_code {lost.get('exit_code')}, expected {CRASH_CODE}"
    assert lost.get("at"), f"p2 has no lost.at: {lost}"
    dumps = lost.get("dumps") or []
    assert all(d.startswith("dumps/p2/") for d in dumps), f"dumps not under dumps/p2/: {dumps}"
    missing = [d for d in dumps if not (r["folder"] / d).is_file()]
    assert not missing, f"dumps listed but not in the run folder: {missing}"
    exts = {Path(d).suffix.lower() for d in dumps}
    assert {".mdmp", ".bidmp"} <= exts, f"the crash left dumps {dumps}; fixture-mod.md promises an .mdmp and a .bidmp"
    e = ex(r["result"], CRASH)
    assert abs(utc(e["reason"]["at"]) - utc(lost["at"])) < 2, f"reason.at {e['reason']['at']} and lost.at {lost['at']} differ"


@m9check("US-016 the check after the crash is invalid session-broken and the run exits 2")
def us016_after(ctx):
    v = part(ctx, "crash")
    broken_after(v["run"], "the crash")
    assert v["run"]["answer"].code == 2, f"{v['run']['answer']}, expected exit 2"


@m9check("US-016 P-09 the machine-lost message of the crash names p2's crash dumps")
def us016_message_dumps(ctx):
    v = part(ctx, "crash")
    e = ex(v["run"]["result"], CRASH)
    dumps = (machine_in(v["run"]["result"], "p2").get("lost") or {}).get("dumps") or []
    assert dumps, "p2 lost no dumps"
    named = [d for d in dumps if Path(d).name in e["reason"]["message"]]
    assert len(named) == len(dumps), f"the message names {len(named)} of {len(dumps)} dumps: {e['reason']['message']!r}"


@m9check("US-016 P-09 p2 ended with taskkill /F by its id during the check: invalid machine-lost naming p2 and exit code 1, lost exited with exit_code 1 and no dumps, the next check session-broken, exit 2, a valid result file, and inspect returns p2 lost")
def us016_kill(ctx):
    v = part(ctx, "kill")
    r = v["run"]
    e = lost_check(r, WAIT, "p2", "the end from outside")
    msg = e["reason"]["message"]
    assert re.search(r"\b1\b", msg) and "exit" in msg.lower(), f"the message gives no exit code 1: {msg!r}"
    took = utc(e["reason"]["at"]) - v["killed_at"]
    assert -1 < took < 5, f"p2 was lost {took:.1f} s after the taskkill"
    valid(ctx, r, "the end from outside")
    lost = machine_in(r["result"], "p2").get("lost") or {}
    assert lost.get("reason") == "exited" and lost.get("exit_code") == 1, f"p2 lost {lost}"
    assert lost.get("dumps") == [], f"p2 ended from outside lists dumps {lost.get('dumps')}"
    broken_after(r, "the end from outside")
    assert r["answer"].code == 2, f"{r['answer']}, expected exit 2"
    j = answered(v["inspect_p2"], "inspect of p2")
    assert (j.get("machine") or {}).get("state") == "lost" and v["inspect_p2"]["s"] <= 20, \
        f"inspect gives p2 {str(j.get('machine'))[:200]} in {v['inspect_p2']['s']:.1f} s"


def excerpt_up_to_end(v, check_id, what):
    r = v["run"]
    e = ex(r["result"], check_id)
    path = (e["evidence"].get("logs") or {}).get("p2")
    assert path, f"{what}: evidence.logs gives no path for p2: {e['evidence'].get('logs')}"
    f = r["folder"] / path
    assert f.is_file(), f"{what}: evidence.logs names {path}, which is not in the run folder"
    lines = [ln.rstrip() for ln in f.read_text(encoding="utf-8", errors="replace").splitlines() if ln.strip()]
    run_id = r["result"]["run"]["id"]
    n = r["result"]["checks"].index(e) + 1
    assert any(f"[shakedown] {run_id}#{n} start" in ln for ln in lines), \
        f"{what}: {path} holds no start marker of {run_id}#{n} ({len(lines)} lines)"
    rpt = [ln.rstrip() for ln in (v["rpt_lines"] or []) if ln.strip()]
    assert rpt, f"{what}: no RPT in p2's profile folder"
    # Compared without the text, which may hold anything the game wrote; only whether they match is said.
    last = lines[-1].split(" ", 1)[-1] if lines else ""
    assert lines and rpt[-1].endswith(last), \
        f"{what}: the last line of {path} is not the last line of p2's RPT (excerpt {len(lines)} lines)"


@m9check("US-043 LOG-04 after p2 crashed, and after it was ended from outside, the run folder holds p2's excerpt for the check, evidence.logs points to it, and it runs from the check's start marker to the last line of p2's RPT")
def us043_logs(ctx):
    excerpt_up_to_end(part(ctx, "crash"), CRASH, "the crash")
    excerpt_up_to_end(part(ctx, "kill"), WAIT, "the end from outside")


@m9check("US-043 LOG-01 P-09 every .mdmp and .bidmp the crash left in p2's profile folder is copied into the run folder, and lost.dumps gives each path")
def us043_dumps(ctx):
    v = part(ctx, "crash")
    new = sorted(set(v["dumps_after"]) - set(v["dumps_before"]))
    assert new, f"the crash left no dump in p2's profile folder ({v['dumps_after']})"
    dumps = (machine_in(v["run"]["result"], "p2").get("lost") or {}).get("dumps") or []
    names = sorted(Path(d).name for d in dumps)
    assert names == new, f"lost.dumps gives {names}; the profile folder got {new}"
    for d in dumps:
        src = next(Path(v["profile"]).rglob(Path(d).name))
        assert (v["run"]["folder"] / d).stat().st_size == src.stat().st_size, f"{d} differs in size from the dump in the profile"


# ================================================================ US-041

@m9check("US-041 OUT-01 DM-01 DM-02 DM-05 a run of a passing check, skd_fnc_fail \"broken on purpose\" and skd_fnc_invalid \"no network\" gives pass, fail explicit and invalid explicit with those messages, summary 1, 1 and 1 of 3, exit 1")
def us041_three(ctx):
    v = part(ctx, "sp")
    r = v["three"]
    valid(ctx, r, "the three outcomes")
    got = kinds(r["result"])
    want = [(AFTER, "pass", None), ("US-041-fail", "fail", "explicit"), ("US-041-invalid", "invalid", "explicit")]
    assert got == want, f"the outcomes are {got}, expected {want}"
    assert "broken on purpose" in ex(r["result"], "US-041-fail")["reason"]["message"], "the fail message"
    assert "no network" in ex(r["result"], "US-041-invalid")["reason"]["message"], "the invalid message"
    s = r["result"]["summary"]
    assert (s["total"], s["pass"], s["fail"], s["invalid"]) == (3, 1, 1, 1), f"summary {s}"
    assert r["answer"].code == 1, f"{r['answer']}, expected exit 1"
    for e in r["result"]["checks"]:
        if e["outcome"] != "pass":
            assert e["reason"].get("at"), f"{e['check_id']} has no reason.at (RES-05)"


@m9check("US-041 a run whose only non-pass outcome is invalid exits 2, and a run of only passing checks exits 0")
def us041_exit(ctx):
    v = part(ctx, "sp")
    assert v["only_invalid"]["answer"].code == 2, f"{v['only_invalid']['answer']}, expected exit 2"
    assert v["only_pass"]["answer"].code == 0, f"{v['only_pass']['answer']}, expected exit 0"


@m9check("US-041 DM-06 RES-05 an assertion that failed before p2 crashed stays fail assertion, p2 is lost exited after it, the next check is invalid session-broken, and the run exits 1")
def us041_fail_first(ctx):
    v = part(ctx, "first")
    r = v["run"]
    valid(ctx, r, "the assertion first")
    e = ex(r["result"], FAIL_LOST)
    k = (e["outcome"], (e["reason"] or {}).get("kind"))
    assert k == ("fail", "assertion"), f"{FAIL_LOST} ended {k}: {(e['reason'] or {}).get('message')}"
    lost = machine_in(r["result"], "p2").get("lost") or {}
    assert lost.get("reason") == "exited" and lost.get("at"), f"p2 lost {lost}"
    lost_at, fail_at, ended = utc(lost["at"]), utc(e["reason"]["at"]), utc(e["ended_at"])
    ctx.log(f"M9 US-041 3: the assertion at {e['reason']['at']}, p2 lost {lost_at - fail_at:.2f} s later, "
            f"the execution ended {ended - fail_at:.2f} s after the assertion")
    assert lost_at > fail_at, f"p2 was lost {fail_at - lost_at:.2f} s before the assertion failed"
    assert lost_at <= ended, f"p2 was lost {lost_at - ended:.2f} s after the execution ended, not during it"
    broken_after(r, "the assertion first")
    assert r["answer"].code == 1, f"{r['answer']}, expected exit 1"


@m9check("US-041 DM-06 DM-07 when p2 is lost first, the wait on it that follows does not decide: the check is invalid machine-lost, not fail")
def us041_lost_first(ctx):
    v = part(ctx, "crash")
    e = lost_check(v["run"], CRASH, "p2", "the crash")
    assert e["evidence"]["failed_step"] is None or isinstance(e["evidence"]["failed_step"], list)


@m9check("NFR-RL-01 US-041 US-015 US-016 US-010 over this repeat's M9 runs, every check after a lost machine, a failed reset or a broken session is invalid, and every fail came before any machine was lost")
def nfr_rl_01(ctx):
    runs = []
    for key, names in (("stall", ["stall"]), ("crash", ["run"]), ("first", ["run"]), ("kill", ["run"]),
                       ("sp", ["three", "only_invalid", "only_pass", "hold", "first", "never", "never2"])):
        v = part(ctx, key)
        runs += [(f"{key}/{n}", v[n]["result"]) for n in names]
    bad, counted = [], 0
    env = ("machine-lost", "session-broken", "reset-failed", "reload-failed", "run-stopped", "unhealthy")
    for what, res in runs:
        lost = [utc(m["lost"]["at"]) for m in res["session"]["machines"] if m.get("lost")]
        broken = False
        for e in res["checks"]:
            kind = (e["reason"] or {}).get("kind")
            if broken or (lost and utc(e["started_at"]) > min(lost)):
                counted += 1
                if e["outcome"] != "invalid":
                    bad.append(f"{what}: {e['check_id']} {e['outcome']} {kind} after the environment failed")
            if e["outcome"] == "fail" and lost and utc(e["reason"]["at"]) >= min(lost):
                bad.append(f"{what}: {e['check_id']} fail {kind} at {e['reason']['at']}, after a machine was lost")
            if e["outcome"] == "invalid" and kind in env:
                counted += 1
                broken = broken or kind in ("machine-lost", "session-broken", "reset-failed", "reload-failed")
    ctx.log(f"M9 NFR-RL-01 repeat {ctx.repeat_n}: {counted} executions ended by or after an environment failure")
    assert not bad, f"{len(bad)} environment failures not reported invalid: {bad}"


# ================================================================ US-059

@m9check("US-059 HLT-02 DM-05 p1's mission time held 8 s by skdfx_fnc_holdClock during a check: invalid unhealthy, reason.machine p1, a message that its mission time stopped, and the run exits 2")
def us059_invalid(ctx):
    v = part(ctx, "sp")
    r = v["hold"]
    valid(ctx, r, "the held clock")
    e = ex(r["result"], HOLD)
    k = (e["outcome"], (e["reason"] or {}).get("kind"))
    held = next((a["value"] for a in e["evidence"]["attached"] if a["label"] == "holdClock"), None)
    assert k == ("invalid", "unhealthy"), f"{HOLD} ended {k}: {(e['reason'] or {}).get('message')}; holdClock gave {held}"
    assert e["reason"].get("machine") == "p1", f"reason.machine {e['reason'].get('machine')!r}"
    assert "mission time" in e["reason"]["message"].lower(), f"the message: {e['reason']['message']!r}"
    assert r["answer"].code == 2, f"{r['answer']}, expected exit 2"


@m9check("US-059 HLT-02 INS-04 while p1's clock stands still status shows p1's health.flag mission-time with its since; the flag clears 5 s after the clock moves again, and the next check passes")
def us059_flag(ctx):
    v = part(ctx, "sp")
    seen = v["hold_health"]
    flagged = [(t, h) for t, h in seen if h and h.get("flag")]
    assert flagged, f"status never showed p1 flagged during the run ({len(seen)} polls)"
    flags = {h["flag"] for _, h in flagged}
    assert flags == {"mission-time"}, f"the flags shown: {flags}"
    assert all(h.get("since") for _, h in flagged), "a flag without since"
    last_flag = flagged[-1][0]
    cleared = [t for t, h in seen if t > last_flag and h and h.get("flag") is None]
    assert cleared, "status did not show the flag cleared before the run ended"
    e = ex(v["hold"]["result"], HOLD)
    hold = next(s for s in e["steps"] if s["path"] == ["hold the clock for 8 s"])
    resumed = utc(hold["start"]["utc"]) + 8
    took = cleared[0] - resumed
    ctx.log(f"M9 US-059: flagged at {flagged[0][1]['since']}, last seen flagged {last_flag - resumed:.1f} s and first seen "
            f"clear {took:.1f} s after the clock moved again")
    assert 4.0 <= took <= 9.0, f"the flag cleared {took:.1f} s after the clock moved again; HLT-02 gives 5 s"
    a = ex(v["hold"]["result"], AFTER)
    assert a["outcome"] == "pass", f"the next check ended {a['outcome']}: {a['reason']}"
    hb = (flagged[-1][1] or {}).get("heartbeat") or {}
    assert {"utc", "tick", "time", "frame"} <= set(hb), f"health.heartbeat {hb}"


@m9check("US-059 DM-06 an assertion that failed before p1's clock stopped stays fail assertion")
def us059_fail_first(ctx):
    v = part(ctx, "sp")
    e = ex(v["first"]["result"], FAIL_FIRST)
    k = (e["outcome"], (e["reason"] or {}).get("kind"))
    assert k == ("fail", "assertion"), f"{FAIL_FIRST} ended {k}: {(e['reason'] or {}).get('message')}"
    since = [h["since"] for _, h in v["first_health"] if h and h.get("flag")]
    assert since, "status never showed p1 flagged after the check, so its clock never stopped"
    stopped, failed, ended = utc(since[0]), utc(e["reason"]["at"]), utc(e["ended_at"])
    ctx.log(f"M9 US-059 3: the clock stopped {stopped - failed:.2f} s after the assertion; the execution ended "
            f"{ended - failed:.2f} s after it")
    assert stopped > failed, f"the clock stopped {failed - stopped:.2f} s before the assertion failed"


# ================================================================ US-010

@m9check("US-010 RST-04 P-06 after a check registers a cleanup that never ends, the next check is invalid reset-failed with a message naming that check, the one after invalid session-broken, and the run exits 2")
def us010_run(ctx):
    v = part(ctx, "sp")
    r = v["never"]
    valid(ctx, r, "the cleanup that never ends")
    got = kinds(r["result"])
    want = [(NEVER, "pass", None), (AFTER, "invalid", "reset-failed"), (AFTER, "invalid", "session-broken")]
    assert got == want, f"the outcomes are {got}, expected {want}"
    msg = ex(r["result"], AFTER, 1)["reason"]["message"]
    assert NEVER in msg or "cleanup that never ends" in msg, f"the reset-failed message does not name {NEVER}: {msg!r}"
    assert r["answer"].code == 2, f"{r['answer']}, expected exit 2"


@m9check("US-010 RST-03 status then shows the session broken with that reason, and shakedown reset exits 4 saying the session is broken and naming reload and session stop")
def us010_refused(ctx):
    v = part(ctx, "sp")
    s = v["never_status"]["session"]
    assert s["state"] == "broken", f"the session is {s['state']}"
    reason = (s.get("reason") or {}).get("message") or ""
    msg = ex(v["never"]["result"], AFTER, 1)["reason"]["message"]
    assert NEVER in reason or "reset" in reason.lower(), f"the session's reason is not the failed reset: {reason!r} (the check's: {msg!r})"
    a = v["never_reset"]
    said = text_of(a)
    assert a.code == 4, f"{a}, expected exit 4: {said[:400]}"
    for word in ("broken", "reload", "session stop"):
        assert word in said.lower(), f"the refusal does not say {word!r}: {said[:500]}"


@m9check("US-010 RST-03 shakedown reset on a ready session exits 0 and reports state done and its duration")
def us010_ready(ctx):
    v = part(ctx, "sp")
    a = v["reset_ready"]
    assert a.code == 0, f"{a}: {text_of(a)[:300]}"
    j = a.json()
    assert j.get("state") == "done" and isinstance(j.get("duration_s"), (int, float)), f"reset gave {j}"


@m9check("US-010 RST-04 shakedown reset with a cleanup that never ends reports state failed listing the mismatch, within the 15 s ceiling plus 10 s, and leaves the session broken")
def us010_reset_fails(ctx):
    v = part(ctx, "sp")
    assert ex(v["never2"]["result"], NEVER)["outcome"] == "pass", f"{NEVER} in the reloaded session: {kinds(v['never2']['result'])}"
    a, took = v["reset_fails"]
    j = a.json()
    assert j.get("state") == "failed", f"reset gave {j} ({a})"
    mism = j.get("mismatches") or []
    assert mism and any(NEVER in m or "cleanup" in m.lower() for m in mism), f"the mismatches do not name the cleanup: {mism}"
    assert took <= 15 + GRACE, f"the reset took {took:.1f} s"
    ctx.log(f"M9 US-010 3: reset exited {a.code} in {took:.1f} s: {mism}")
    assert v["reset_fails_status"]["session"]["state"] == "broken", f"the session is {v['reset_fails_status']['session']['state']}"


@m9check("US-010 RLD-01 shakedown reload restarts the game of the session the failed reset broke, and the session is ready again")
def us010_reload(ctx):
    v = part(ctx, "sp")
    a = v["reload"]
    assert a.code == 0, f"{a}: {text_of(a)[:400]}"
    j = a.json()
    assert j.get("state") == "done" and j.get("level") == "game-restart", f"reload gave {str(j)[:400]}"
    assert v["reloaded"]["session"]["state"] == "ready", f"the session is {v['reloaded']['session']['state']}"
