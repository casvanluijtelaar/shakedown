r"""The checks of M12 Shared helpers and mod events: US-070, US-071, US-072 and US-073, and DEF-27's criteria.

Each part starts and stops a session of its own, once per repeat (`--repeat`), with no session running
before it. The project `m12` in the work folder holds the checks of `engineering/acceptance/m12/` in its `tests\`, so
its library folder is the default `tests\_lib\` (`rope.sqf`, `kit.sqf`, and `sub\deep.sqf`, which is in a
subfolder and so neither a library file nor a check), and two setups: `ded`, a dedicated server with two
clients loading CBA and both fixtures, and `sp`, singleplayer loading `@skd_fixture` alone. The project
`m12key` holds `engineering/acceptance/m12key/`: `lib = "helpers"` and one check, run in singleplayer. Per repeat:

- `list` from the command line and `list_checks` through MCP in `m12`, with no session (US-070 2).
- `ded`: a run with no selection (US-070 1 and 2, US-072, US-073); `rope.sqf` edited and `shakedown reload`
  with no file named, then the library checks again (US-070 3); `rope.sqf` holding `1 +`, a reload and a run
  of three checks (US-071 1); the file fixed, a reload and the same run (US-071 2). The game processes and
  start numbers are compared over each reload.
- `sp`: the two library checks and the plain one (US-070 1).
- `m12key` `sp`: the check of the `lib` key (US-070 4).

Every game process a part saw is stopped by its id if it outlives the stop. A part whose game start is lost
to `Error 20 reading file` of a base-game file is started again, at most three times in all.
"""

import json
import re
import shutil
import time
import traceback
from pathlib import Path

from . import driver
from .core import check
from .m08 import game_folder
from .m11 import (execution, got, machines, need_quiet, passed, result_of, run, stage, start, status_of,
                  step_of, stop, utc)
from .mcp import Mcp, structured

ACC = Path(__file__).resolve().parent.parent
LIB_A, LIB_B, PLAIN = "US-070-lib-a", "US-070-lib-b", "US-071-plain"
RECORD, NEXT = "US-072-1-record", "US-072-2-next"
CEILING, UNRECORDED = "US-073-ceiling", "US-073-unrecorded"
KEY = "US-070-key"
DED_CHECKS = [LIB_A, LIB_B, PLAIN, RECORD, NEXT, CEILING, UNRECORDED]
LIB_RUN = [LIB_A, PLAIN, LIB_B]
DED_MACHINES = ["server", "p1", "p2"]
ROPE = "createHashMapFromArray [\n    [\"version\", {v}],\n    [\"double\", {{ (_this select 0) * 2 }}]\n]\n"
BROKEN = "1 +"


# ================================================================ the projects and the parts

def keep(ctx):
    if not hasattr(ctx, "m12_keep"):
        ctx.m12_keep = {}
    return ctx.m12_keep


def mcp_json(project):
    (project / ".mcp.json").write_text(json.dumps(
        {"mcpServers": {"shakedown": {"command": "shakedown", "args": ["mcp"], "timeout": 70000}}}, indent=2),
        encoding="utf-8")


def projects(ctx):
    store = keep(ctx)
    if "projects" in store:
        return store["projects"]
    game = game_folder(ctx.project)
    cba = cba_folder(ctx.project)
    fixture_cba = Path(ctx.install) / "fixtures" / "@skd_fixture_cba"
    mods = (f"[mods.cba]\npath = '{cba}'\npatch = \"cba_main\"\n\n"
            f"[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n\n"
            f"[mods.fixture-cba]\npath = '{fixture_cba}'\npatch = \"skdfx_cba\"\n\n")
    m12 = ctx.work / "m12"
    if m12.exists():
        shutil.rmtree(m12)
    shutil.copytree(ACC / "m12", m12 / "tests")
    (m12 / "shakedown.toml").write_text(
        f"game = '{game}'\n\n{mods}"
        "[setups.ded]\nmode = \"dedicated\"\nclients = 2\nmods = [\"cba\", \"fixture\", \"fixture-cba\"]\n\n"
        "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n", encoding="utf-8")
    mcp_json(m12)
    key = ctx.work / "m12key"
    if key.exists():
        shutil.rmtree(key)
    shutil.copytree(ACC / "m12key", key)
    (key / "shakedown.toml").write_text(
        f"game = '{game}'\nlib = \"helpers\"\n\n[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n\n"
        "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n", encoding="utf-8")
    store["projects"] = {"m12": (m12, driver.Shakedown(ctx.sh.exe, m12, ctx.log)),
                         "m12key": (key, driver.Shakedown(ctx.sh.exe, key, ctx.log))}
    return store["projects"]


def cba_folder(project):
    text = (Path(project) / "shakedown.toml").read_text(encoding="utf-8")
    m = re.search(r"\[mods\.cba\]\npath = '([^']+)'", text)
    assert m, "the generated project's shakedown.toml names no CBA folder"
    return Path(m.group(1))


def rope_file(project):
    return project / "tests" / "_lib" / "rope.sqf"


def machine_fault(ctx, since):
    for name in ("m12", "m12key"):
        root = ctx.work / name / ".shakedown" / "sessions"
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
                ctx.log(f"M12 part {k}: {traceback.format_exc()}")
                store[k] = e
                fault = machine_fault(ctx, began)
                if not fault or attempt == 3:
                    break
                ctx.log(f"M12 part {k}: a game process could not read a base-game file ({fault}); the machine, "
                        f"not the framework. Starting the part again, attempt {attempt + 1} of 3")
    value = store[k]
    if isinstance(value, Exception):
        raise AssertionError(f"the M12 part {key} failed: {value!r}")
    return value


def reload(ctx, sh):
    """`shakedown reload --json` with no file named, with status and the game processes before and after."""
    before = status_of(sh)
    pids_before = driver.game_pids()
    answer = sh("reload", "--json", ceiling=300)
    out = answer.json() if answer.stdout.strip() else {}
    ctx.log(f"M12 reload: exit {answer.code}, {json.dumps(out)[:600]}")
    return {"answer": answer, "out": out, "before": before, "after": status_of(sh),
            "pids_before": pids_before, "pids_after": driver.game_pids()}


def list_part(ctx):
    project, sh = projects(ctx)["m12"]
    answer = sh("list", "--json", ceiling=60)
    with Mcp(project, ctx.install, ctx.log) as m:
        result, _ = m.call("list_checks", {}, ceiling=60)
        tool = structured(result)
    return {"cli": answer, "tool": tool}


def ded_part(ctx):
    project, sh = projects(ctx)["m12"]
    rope = rope_file(project)
    rope.write_text(ROPE.format(v=1), encoding="utf-8")
    need_quiet(sh)
    pids, out = set(), {}
    try:
        out["session"] = start(sh, "ded", pids)
        stage(ctx, out, "all", lambda: result_of(sh("run", "--json", ceiling=900)))
        rope.write_text(ROPE.format(v=2), encoding="utf-8")
        stage(ctx, out, "reload_edit", lambda: reload(ctx, sh))
        stage(ctx, out, "edited", lambda: run(sh, [LIB_A, LIB_B]))
        rope.write_text(BROKEN, encoding="utf-8")
        stage(ctx, out, "reload_broken", lambda: reload(ctx, sh))
        stage(ctx, out, "broken", lambda: run(sh, LIB_RUN))
        rope.write_text(ROPE.format(v=3), encoding="utf-8")
        stage(ctx, out, "reload_fixed", lambda: reload(ctx, sh))
        stage(ctx, out, "fixed", lambda: run(sh, LIB_RUN))
        out["pids"] = set(pids)
    finally:
        stop(ctx, sh, pids, [])
        rope.write_text(ROPE.format(v=1), encoding="utf-8")
    return out


def sp_part(ctx):
    project, sh = projects(ctx)["m12"]
    rope_file(project).write_text(ROPE.format(v=1), encoding="utf-8")
    need_quiet(sh)
    pids = set()
    try:
        start(sh, "sp", pids)
        return {"run": run(sh, LIB_RUN)}
    finally:
        stop(ctx, sh, pids, [])


def key_part(ctx):
    project, sh = projects(ctx)["m12key"]
    need_quiet(sh)
    pids = set()
    try:
        start(sh, "sp", pids)
        return {"run": run(sh, [KEY]), "list": sh("list", "--json", ceiling=60)}
    finally:
        stop(ctx, sh, pids, [])


def ded(ctx, key):
    return got(once(ctx, "ded", ded_part), key)


def attached(ex):
    return {e["label"]: e["value"] for e in ex["evidence"]["attached"]}


def versions_are(ex, expected, names):
    """The library checks attach the version their library gave in the body and on each machine."""
    a = attached(ex)
    want = {"lead": expected, **{f"on {n}": expected for n in names}}
    have = {k: a.get(k) for k in want}
    assert have == want, f"{ex['check_id']}: the library gave {have}, expected {want}; outcome {ex['outcome']} {ex['reason']}"


def check_error(ex):
    assert ex["outcome"] == "fail" and ex["reason"] and ex["reason"]["kind"] == "check-error", \
        f"{ex['check_id']} ended {ex['outcome']} {ex['reason']}, expected fail check-error"
    return ex["reason"]["message"]


# ================================================================ US-070

@check("US-070 API-06 DEF-27 tests/_lib/rope.sqf is callable as call skd_lib_rope in the body of two checks and in code sent to server, p1 and p2 on a dedicated session with two clients", "start", repeat=True)
def us070_ded(ctx):
    result = ded(ctx, "all")["result"]
    for cid in (LIB_A, LIB_B):
        ex = execution(result, cid)
        passed(ex)
        versions_are(ex, 1, DED_MACHINES)


@check("US-070 API-06 DEF-27 tests/_lib/rope.sqf is callable as call skd_lib_rope in the body of two checks and in code sent to p1 in singleplayer", "start", repeat=True)
def us070_sp(ctx):
    result = once(ctx, "sp", sp_part)["run"]["result"]
    for cid in (LIB_A, LIB_B):
        ex = execution(result, cid)
        passed(ex)
        versions_are(ex, 1, ["p1"])
    passed(execution(result, PLAIN))


def no_library_in(ids, where):
    bad = [i for i in ids if i.startswith("_lib") or "rope" in i or "kit" in i or "deep" in i]
    assert not bad, f"{where} takes library files as checks: {bad}"
    assert sorted(ids) == sorted(DED_CHECKS), f"{where} lists {sorted(ids)}, expected the checks {sorted(DED_CHECKS)}"


@check("US-070 API-06 DEF-27 shakedown list and list_checks through MCP list no file of the library folder as a check, also none in a subfolder of it", "start", repeat=True)
def us070_list(ctx):
    lst = once(ctx, "list", list_part)
    assert lst["cli"].code == 0, f"{lst['cli']}: {lst['cli'].stderr[:300]}"
    no_library_in([c["id"] for c in lst["cli"].json()["checks"]], "shakedown list")
    no_library_in([c["id"] for c in lst["tool"]["checks"]], "list_checks")


@check("US-070 API-06 RUN-02 a run with no selection on the dedicated session runs no file of the library folder", "start", repeat=True)
def us070_run_all(ctx):
    r = ded(ctx, "all")
    result = r["result"]
    ids = [c["check_id"] for c in result["checks"]] + [n.get("check_id", "") for n in result["not_run"]]
    no_library_in(ids, "the run with no selection")
    assert result["run"]["selection"]["ids"] == [], f"the run's selection: {result['run']['selection']}"
    assert [c["check_id"] for c in result["checks"]] == sorted(DED_CHECKS), \
        f"the run executed {[c['check_id'] for c in result['checks']]}"


@check("US-070 API-06 RLD-01 RLD-04 DEF-27 after rope.sqf is edited shakedown reload lists it at level none, restarts no game, and the next checks use the edited helper on every machine", "start", repeat=True)
def us070_reload(ctx):
    rl = ded(ctx, "reload_edit")
    out = rl["out"]
    assert rl["answer"].code == 0, f"{rl['answer']}: {rl['answer'].stderr[:400]}"
    assert out.get("state") == "done" and out.get("level") == "none", f"the reload: {out}"
    files = [f for f in out.get("files", []) if Path(f["path"]).name == "rope.sqf"]
    assert files and all(f["level"] == "none" for f in files), f"the reload's files: {out.get('files')}"
    assert all(Path(f["path"]).parent.name == "_lib" for f in files), f"the reload names {files}"
    assert rl["pids_before"] == rl["pids_after"], f"game processes {sorted(rl['pids_before'])} became {sorted(rl['pids_after'])}"
    b, a = machines(rl["before"]), machines(rl["after"])
    assert {n: (m["start"], m["pid"]) for n, m in b.items()} == {n: (m["start"], m["pid"]) for n, m in a.items()}, \
        f"the machines went from {b} to {a}"
    result = ded(ctx, "edited")["result"]
    for cid in (LIB_A, LIB_B):
        ex = execution(result, cid)
        passed(ex)
        versions_are(ex, 2, DED_MACHINES)
    ctx.log(f"US-070: reload of the library took {out.get('duration_s')} s, machines {out.get('machines')}")


@check("US-070 API-06 CFG-01 a project whose shakedown.toml sets lib = \"helpers\" takes its library from helpers/", "start", repeat=True)
def us070_key(ctx):
    k = once(ctx, "key", key_part)
    passed(execution(k["run"]["result"], KEY))
    assert k["list"].code == 0, f"{k['list']}"
    ids = [c["id"] for c in k["list"].json()["checks"]]
    assert ids == [KEY], f"list in the project with lib = \"helpers\" gives {ids}"


# ================================================================ US-071

LINE = re.compile(r"\bline 1\b")


@check("US-071 API-06 DM-04 with tests/_lib/rope.sqf holding 1 + every check of a run ends fail check-error before its body, naming the file, its line and the engine's message, and the run goes on to its end", "start", repeat=True)
def us071_broken(ctx):
    rl = ded(ctx, "reload_broken")
    ctx.log(f"US-071: the reload of the broken file: exit {rl['answer'].code}, state {rl['out'].get('state')}, "
            f"level {rl['out'].get('level')}")
    r = ded(ctx, "broken")
    result = r["result"]
    assert result["run"]["state"] == "finished", f"the run ended {result['run']['state']}"
    assert [c["check_id"] for c in result["checks"]] == LIB_RUN, f"the run executed {[c['check_id'] for c in result['checks']]}"
    for ex in result["checks"]:
        msg = check_error(ex)
        assert re.search(r"tests[\\/]_lib[\\/]rope\.sqf", msg), f"{ex['check_id']}: the message does not name tests/_lib/rope.sqf: {msg}"
        assert LINE.search(msg), f"{ex['check_id']}: the message does not name line 1: {msg}"
        rest = re.sub(r"tests[\\/]_lib[\\/]rope\.sqf|line 1|does not compile|shakedown reload|every check|fix", "", msg, flags=re.I)
        assert len(re.findall(r"[A-Za-z]{3,}", rest)) >= 3, f"{ex['check_id']}: the message holds no engine message: {msg}"
        assert ex["steps"] == [], f"{ex['check_id']}: steps ran: {[s['path'] for s in ex['steps']]}"
    ctx.log(f"US-071: {result['checks'][0]['reason']['message']}")


@check("US-071 API-06 RLD-04 once rope.sqf is fixed and shakedown reload is run the next run's checks reach their own outcomes", "start", repeat=True)
def us071_fixed(ctx):
    rl = ded(ctx, "reload_fixed")
    assert rl["answer"].code == 0 and rl["out"].get("level") == "none", f"{rl['answer']}: {rl['out']}"
    assert rl["pids_before"] == rl["pids_after"], f"game processes {sorted(rl['pids_before'])} became {sorted(rl['pids_after'])}"
    result = ded(ctx, "fixed")["result"]
    for cid in LIB_RUN:
        passed(execution(result, cid))
    for cid in (LIB_A, LIB_B):
        versions_are(execution(result, cid), 3, DED_MACHINES)


# ================================================================ US-072

def events_of(ex, name):
    return [e for e in ex["events"] if e["name"].lower() == name.lower()]


@check("US-072 EVT-01 skd_fnc_record of the fixture's CBA event on server, p1 and p2 and skd_fnc_pressKey on p1: skd_fnc_waitEvent on p2 returns the event's arguments, p1's unit", "start", repeat=True)
def us072_wait(ctx):
    ex = execution(ded(ctx, "all")["result"], RECORD)
    passed(ex)
    for title in ("record on server p1 and p2", "p1 signals", "p2 waits for the signal"):
        assert step_of(ex, title)["state"] == "passed", f"{title}: {step_of(ex, title)}"
    assert attached(ex).get("args on p2"), f"no arguments attached: {attached(ex)}"


@check("US-072 EVT-01 OBS-05 RES-01 skd_fnc_events gives one event per machine with its arguments and clocks, and the result file holds the same events under the check", "start", repeat=True)
def us072_events(ctx):
    result = ded(ctx, "all")["result"]
    errors = ctx.validate(result)
    assert not errors, f"the result file is not valid: {errors[:5]}"
    ex = execution(result, RECORD)
    passed(ex)
    assert step_of(ex, "each machine kept one signal with its clocks")["state"] == "passed"
    sig = events_of(ex, "skdfx_signalled")
    assert sorted(e["machine"] for e in sig) == sorted(DED_MACHINES), f"the result file's skdfx_signalled events: {sig}"
    a = attached(ex)
    for e in sig:
        assert e["kind"] == "cba", f"{e}"
        assert e["at"]["utc"] and e["at"]["frame"] > 0 and e["at"]["tick"] > 0, f"the clocks of {e}"
        ev = a.get(f"event on {e['machine']}")
        assert ev and ev[1] == e["at"]["utc"], f"skd_fnc_events on {e['machine']} gave {ev}, the result file {e}"
        assert e["args"] == ev[0], f"on {e['machine']}: the result file's arguments {e['args']}, skd_fnc_events {ev[0]}"
    times = [utc(e["at"]["utc"]) for e in ex["events"]]
    assert times == sorted(times), f"the events are not in wall-clock order: {[e['at']['utc'] for e in ex['events']]}"


@check("US-072 EVT-01 an engine mission event, EntityKilled, is recorded on the server the same way", "start", repeat=True)
def us072_mission(ctx):
    ex = execution(ded(ctx, "all")["result"], RECORD)
    passed(ex)
    assert step_of(ex, "the server records EntityKilled")["state"] == "passed"
    killed = events_of(ex, "EntityKilled")
    assert len(killed) == 1 and killed[0]["machine"] == "server" and killed[0]["kind"] == "mission", \
        f"the result file's EntityKilled events: {killed}"


@check("US-072 EVT-01 in the next check no handler the recording added is left: the event fired before skd_fnc_record records nothing on any machine, and once recorded each fires once", "start", repeat=True)
def us072_next(ctx):
    ex = execution(ded(ctx, "all")["result"], NEXT)
    passed(ex)
    order = [c["check_id"] for c in ded(ctx, "all")["result"]["checks"]]
    assert order.index(NEXT) == order.index(RECORD) + 1, f"the check after the recording is not next: {order}"
    sig = events_of(ex, "skdfx_signalled")
    assert sorted(e["machine"] for e in sig) == sorted(DED_MACHINES) and len(events_of(ex, "EntityKilled")) == 1, \
        f"the next check's result file holds {[(e['machine'], e['name']) for e in ex['events']]}"


# ================================================================ US-073

@check("US-073 EVT-01 OBS-04 DM-03 a wait on p2 for a signal only p1 raised locally ends fail wait-ceiling after 3 s, naming the event and p2, with none recorded on p2 and one on p1", "start", repeat=True)
def us073_ceiling(ctx):
    ex = execution(ded(ctx, "all")["result"], CEILING)
    assert ex["outcome"] == "fail" and ex["reason"]["kind"] == "wait-ceiling", f"{ex['outcome']} {ex['reason']}"
    msg = ex["reason"]["message"]
    assert "skdfx_signalled" in msg and re.search(r"\bp2\b", msg), f"the message does not name the event and p2: {msg}"
    m_p2 = re.search(r"\bp2: ([^;.]*)", msg)
    m_p1 = re.search(r"\bp1: ([^;.]*)", msg)
    assert m_p2 and re.search(r"none|\b0\b", m_p2.group(1)), f"the message does not say p2 recorded none: {msg}"
    assert m_p1 and re.search(r"\b1\b", m_p1.group(1)), f"the message does not say p1 recorded one: {msg}"
    s = step_of(ex, "p2 waits 3 s for it")
    assert s["state"] == "failed", f"{s}"
    took = (utc(ex["ended_at"]) - utc(s["start"]["utc"])).total_seconds()
    assert 3 <= s["duration_s"] + 0.05 and took <= 3 + 3, f"the wait took {s['duration_s']} s, the check ended {took:.2f} s after it began"
    assert not any(st["path"] == ["the step after the wait"] for st in ex["steps"]), "the step after the wait ran"
    observed = json.dumps(ex["evidence"]["observed"])
    assert "p1" in observed and "p2" in observed, f"evidence.observed does not hold the counts per machine: {observed}"
    ctx.log(f"US-073: {msg} observed {observed}; the wait took {s['duration_s']} s")


@check("US-073 EVT-01 DM-04 a wait for an event the check did not record on that machine ends fail check-error saying it is not recorded there", "start", repeat=True)
def us073_unrecorded(ctx):
    ex = execution(ded(ctx, "all")["result"], UNRECORDED)
    msg = check_error(ex)
    assert "skdfx_signalled" in msg and re.search(r"\bp2\b", msg) and re.search(r"record", msg, re.I), \
        f"the message does not say the event is not recorded on p2: {msg}"
    assert not any(st["path"] == ["the step after the wait"] for st in ex["steps"]), "the step after the wait ran"
    s = step_of(ex, "p2 waits for an event it does not record")
    assert s["duration_s"] < 3, f"the wait took {s['duration_s']} s; a check-error comes at once"
    ctx.log(f"US-073: {msg}")
