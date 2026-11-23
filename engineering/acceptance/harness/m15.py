r"""The checks of M15 Hand over to the manual tester: US-083 and US-084.

Each part starts and stops what it needs itself, once per repeat (`--repeat`), with no session running before it,
in the project `m15` of its own in the work folder. Its `tests\` holds the checks of `engineering/acceptance/m15/`; its setups,
in this order, are `sp` (singleplayer) and `ded` (a dedicated server with two clients and a headless client), each
loading `@skd_fixture` alone.

Per repeat:

- `ded`: `run --setup ded US-083-crate US-083-after` with no session (US-083 1 to 3), then in the session it left:
  `exec` and `logs` of p1 and the server; `run US-083-baseline` (the next run takes it back); `run_start` of
  `US-083-crate US-083-after` through `shakedown mcp` with `run_status` and `run_result`; `exec`, then `reset`,
  then `exec`; `run --json US-083-crate`, `reload`, `status` and `run US-083-baseline`; `run US-084-server
  US-084-hc1 US-083-baseline` (US-084); `run US-083-crate` and `session stop` (US-083 4).
- `each`: `run --each-setup US-083-each US-083-after US-083-dedonly` while the game processes are listed every
  second, `status`, then `session stop` (RUN-07, MAN-01).

What the manual tester sees on screen is the owner's look (exit criterion 3); these checks judge what the contract
gives without looking: the output, the result file, `run_result`, `status`, positions through `exec`, and p1's RPT.
Every game process a part saw is stopped by its id if it outlives the part. A part whose game start is lost to
`Error 20 reading file` of a base-game file is started again, at most three times in all.
"""

import json
import re
import shutil
import time
import traceback
from pathlib import Path

from . import driver, schema
from .core import check
from .m08 import game_folder
from .m11 import result_of, status_of
from .m13 import Procs, mcp_json, need_quiet, read_result
from .mcp import Mcp, structured

ACC = Path(__file__).resolve().parent.parent
CRATE, AFTER, BASE = "US-083-crate", "US-083-after", "US-083-baseline"
SERVER, HC1 = "US-084-server", "US-084-hc1"
EACH, DEDONLY = "US-083-each", "US-083-dedonly"
ITEMS = [
    "An ammo crate stands 2 m in front of you.",
    "Looking at it, the action menu (mouse wheel) offers Open crate.",
    "You carry no first aid kit; opening the crate puts one in your inventory.",
]
STOP = "shakedown session stop"
RESULT = re.compile(r"Result: (\S.*?result\.json)")


# ================================================================ the project

def keep(ctx):
    if not hasattr(ctx, "m15_keep"):
        ctx.m15_keep = {}
    return ctx.m15_keep


def toml(game):
    return (f"game = '{game}'\n\n[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n\n"
            "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n\n"
            "[setups.ded]\nmode = \"dedicated\"\nclients = 2\nheadless = 1\nmods = [\"fixture\"]\n")


def project(ctx):
    store = keep(ctx)
    if "project" not in store:
        root = ctx.work / "m15"
        if root.exists():
            shutil.rmtree(root)
        shutil.copytree(ACC / "m15", root / "tests")
        mcp_json(root)
        (root / "shakedown.toml").write_text(toml(game_folder(ctx.project)), encoding="utf-8")
        store["project"] = (root, driver.Shakedown(ctx.sh.exe, root, ctx.log))
    return store["project"]


def machine_fault(root, since):
    sessions = root / ".shakedown" / "sessions"
    for rpt in sessions.glob("*/machines/*/*.rpt") if sessions.is_dir() else []:
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
        root, _ = project(ctx)
        for attempt in range(1, 4):
            began = time.time()
            try:
                store[k] = fn(ctx)
                break
            except Exception as e:  # noqa: BLE001
                ctx.log(f"M15 part {k}: {traceback.format_exc()}")
                store[k] = e
                fault = machine_fault(root, began)
                if not fault or attempt == 3:
                    break
                ctx.log(f"M15 part {k}: a game process could not read a base-game file ({fault}); the machine, "
                        f"not the framework. Starting the part again, attempt {attempt + 1} of 3")
    value = store[k]
    if isinstance(value, Exception):
        raise AssertionError(f"the M15 part {key} failed: {value!r}")
    return value


# ================================================================ helpers

def session_pids(st):
    return {m["pid"] for m in (st.get("session") or {}).get("machines", []) if m.get("pid")}


def tidy(ctx, sh, pids):
    """Stops a session the part left, then every game process it saw that is still alive, by its id."""
    try:
        st = status_of(sh)
        pids |= session_pids(st)
        if st.get("session"):
            sh("session", "stop", "--json", ceiling=180)
    except Exception as e:  # noqa: BLE001
        ctx.log(f"M15: status or session stop after the part: {e!r}")
    for pid in set(pids):
        if driver.alive(pid) and driver.wait_gone(pid, 30) is None:
            ctx.log(f"M15: game process {pid} still runs; stopping it by id")
            driver.kill(pid)
    ctx.runner_pid = driver.runner_pid() or ctx.runner_pid


def text_run(sh, *args, ceiling=900):
    answer = sh("run", *args, ceiling=ceiling)
    paths = RESULT.findall(answer.stdout)
    return {"answer": answer, "result": read_result(paths[-1]) if paths else None, "path": paths[-1] if paths else None}


def value_of(answer, machine):
    out = answer.json()
    res = out["results"][machine]
    assert res["state"] == "done", f"{answer}: exec on {machine} ended {res}"
    return res["value"]


def scene(sh):
    """p1's position ASL and the count of skdfx_Crate objects on the server, through exec."""
    pos = value_of(sh("exec", "--machine", "p1", "--code", "getPosASL player", "--json", ceiling=60), "p1")
    crates = value_of(sh("exec", "--machine", "server", "--code", "count (allMissionObjects \"skdfx_Crate\")",
                         "--json", ceiling=60), "server")
    return {"pos": pos, "crates": crates}


# Every control on p1's displays whose text starts with the box's title, with its position and parent group, the
# safe zone, and the default layout's HUD elements from the game's own config (CfgUIGrids, preset Arma3).
BOX_SQF = (
    'private _hit = []; { { if ((ctrlText _x) find "Shakedown: look for" == 0) then { _hit pushBack '
    '[ctrlPosition _x, str (ctrlParentControlsGroup _x), ctrlShown _x] } } forEach allControls _x } forEach allDisplays; '
    'private _v = configFile >> "CfgUIGrids" >> "IGUI" >> "Presets" >> "Arma3" >> "Variables"; '
    'private _hud = []; { private _a = getArray (_v >> _x); if (count _a > 0) then { _hud pushBack [_x, '
    '(_a select 0) apply { if (_x isEqualType "") then { call compile _x } else { _x } }] } } forEach '
    '["grid_menu", "grid_comm", "grid_vehicle", "grid_weapon", "grid_stance", "grid_stamina", "grid_hint"]; '
    '[_hit, [safeZoneX, safeZoneY, safeZoneW, safeZoneH], _hud]'
)
# The three items of the hand over as p1 sees them: the crate, its distance and direction, its action Open crate
# with the action's radius, and p1's first aid kits.
ITEMS_SQF = (
    'private _c = nearestObjects [player, ["skdfx_Crate"], 10]; '
    'if (count _c == 0) exitWith { [0] }; private _crate = _c select 0; '
    'private _ids = (actionIDs _crate) select { ((_crate actionParams _x) select 0) isEqualTo "Open crate" }; '
    '[count _c, player distance _crate, player getRelDir _crate, _ids, '
    'if (count _ids > 0) then { (_crate actionParams (_ids select 0)) select 8 } else { -1 }, '
    '{ _x == "FirstAidKit" } count items player]'
)
# Fires the crate's action Open crate as the action menu does when the player selects it.
FIRE_SQF = (
    'private _crate = (nearestObjects [player, ["skdfx_Crate"], 10]) select 0; '
    'private _id = (actionIDs _crate) select ((actionIDs _crate) findIf '
    '{ ((_crate actionParams _x) select 0) isEqualTo "Open crate" }); player action ["User", _crate, _id]; _id'
)
KITS_SQF = '{ _x == "FirstAidKit" } count items player'


def on_p1(sh, code):
    return value_of(sh("exec", "--machine", "p1", "--code", code, "--json", ceiling=60), "p1")


def items_part(sh):
    """Reads each item on p1, fires Open crate and polls p1's first aid kits, at most 20 times over about 10 s."""
    out = {"before": on_p1(sh, ITEMS_SQF), "fired": on_p1(sh, FIRE_SQF), "kits": []}
    for _ in range(20):
        out["kits"].append(on_p1(sh, KITS_SQF))
        if out["kits"][-1] >= 1:
            break
        time.sleep(0.5)
    return out


def dist2d(pos, x, y):
    return ((pos[0] - x) ** 2 + (pos[1] - y) ** 2) ** 0.5


# ================================================================ the parts

def ded_part(ctx):
    root, sh = project(ctx)
    need_quiet(sh)
    pids, out = set(), {}
    try:
        # US-083 1 to 3: a run that starts its own session and hands over.
        out["first"] = text_run(sh, "--setup", "ded", CRATE, AFTER, ceiling=900)
        out["status"] = status_of(sh)
        pids |= session_pids(out["status"])
        out["game_after_first"] = driver.game_pids()
        out["scene_first"] = scene(sh)
        out["logs"] = sh("logs", "--machine", "p1", "--grep", "crate|first aid", "--last", "100", ceiling=60)
        # m15-1 and m15-2: the box's place, then the three items, on p1 in the situation handed over.
        out["box_first"] = on_p1(sh, BOX_SQF)
        out["items"] = items_part(sh)
        ctx.log(f"M15 ded box {out['box_first']}\nitems {out['items']}")
        ctx.log(f"M15 ded first: {out['first']['answer'].stdout}\np1's log lines: {out['logs'].stdout[-1500:]}")
        # US-083 4: the next run takes the session back.
        out["base_run"] = result_of(sh("run", BASE, "--json", ceiling=600))
        # US-083 1 through the agent tools.
        with Mcp(root, ctx.install, ctx.log) as m:
            tools = {t["name"]: t for t in m.tools()}
            out["run_result_schema"] = tools.get("run_result", {}).get("outputSchema")
            start = structured(m.call("run_start", {"ids": [CRATE, AFTER]}, ceiling=70)[0])
            st = None
            for _ in range(20):  # wait_s 30 each, at most 10 min
                st = structured(m.call("run_status", {"run_id": start["run_id"], "wait_s": 30}, ceiling=70)[0])
                if st["state"] in ("finished", "stopped"):
                    break
            out["mcp"] = {"start": start, "status": st,
                          "result": structured(m.call("run_result", {"run_id": start["run_id"]}, ceiling=70)[0])}
        out["scene_mcp"] = scene(sh)
        # US-083 4: reset takes it back.
        out["reset"] = sh("reset", "--json", ceiling=120)
        out["scene_reset"] = scene(sh)
        out["box_reset"] = on_p1(sh, BOX_SQF)
        # run --json hands over; then reload takes it back.
        out["json_run"] = result_of(sh("run", CRATE, AFTER, "--json", ceiling=600))
        out["reload"] = sh("reload", "--json", ceiling=400)
        out["status_reload"] = status_of(sh)
        pids |= session_pids(out["status_reload"])
        out["base_reload"] = result_of(sh("run", BASE, "--json", ceiling=600))
        # US-084: hand overs to machines with no screen.
        out["refused"] = result_of(sh("run", SERVER, HC1, BASE, "--json", ceiling=600))
        out["status_refused"] = status_of(sh)
        # US-083 4: session stop takes it back.
        out["last"] = text_run(sh, CRATE, ceiling=600)
        st = status_of(sh)
        pids |= session_pids(st)
        out["status_last"] = st
        out["stop"] = sh("session", "stop", "--json", ceiling=180)
        out["status_stopped"] = status_of(sh)
        out["left"] = {pid for pid in pids if driver.wait_gone(pid, 30) is None}
    finally:
        tidy(ctx, sh, pids)
    return out


def each_part(ctx):
    root, sh = project(ctx)
    need_quiet(sh)
    pids, out = set(), {}
    try:
        procs = Procs()
        try:
            out["answer"] = sh("run", "--each-setup", EACH, AFTER, DEDONLY, ceiling=900)
            out["status"] = status_of(sh)
            pids |= session_pids(out["status"])
            out["scene"] = value_of(sh("exec", "--machine", "p1", "--code", "getPosASL player", "--json",
                                       ceiling=60), "p1")
            out["box"] = on_p1(sh, BOX_SQF)
        finally:
            procs.stop()
        out["procs"] = procs
        pids |= procs.pids()
        out["results"] = [read_result(p) for p in RESULT.findall(out["answer"].stdout)]
        ctx.log(f"M15 each: {out['answer']}\n{out['answer'].stdout}\nprocesses {procs.describe()}")
        out["stop"] = sh("session", "stop", "--json", ceiling=180)
        out["status_stopped"] = status_of(sh)
    finally:
        tidy(ctx, sh, pids)
    return out


def ded(ctx):
    return once(ctx, "ded", ded_part)


def each(ctx):
    return once(ctx, "each", each_part)


# ================================================================ judging

def valid(ctx, result, what):
    errors = ctx.validate(result)
    assert not errors, f"{what}: the result file is not valid against result.schema.json: {errors[:5]}"


def only(result, cid):
    found = [c for c in result["checks"] if c["check_id"] == cid]
    assert len(found) == 1, f"the result file holds {len(found)} executions of {cid}: {[c['check_id'] for c in result['checks']]}"
    return found[0]


def handed_over_file(ctx, result, what):
    """The hand over in a result file of CRATE then AFTER: CRATE pass with handed_over, AFTER not run handed-over."""
    valid(ctx, result, what)
    ex = only(result, CRATE)
    assert ex["outcome"] == "pass", f"{what}: {CRATE} ended {ex['outcome']} {ex.get('reason')}"
    assert ex.get("handed_over") == {"machine": "p1", "items": ITEMS}, f"{what}: handed_over {ex.get('handed_over')}"
    others = [c for c in result["checks"] if c.get("handed_over")]
    assert len(others) == 1, f"{what}: {len(others)} executions carry handed_over"
    assert [c["check_id"] for c in result["checks"]] == [CRATE], \
        f"{what}: executed {[c['check_id'] for c in result['checks']]}, expected only {CRATE}"
    assert [(n["check_id"], n["reason"]) for n in result["not_run"]] == [(AFTER, "handed-over")], \
        f"{what}: not_run {result['not_run']}"


def handed_over_answer(ho, sid, what):
    assert ho is not None, f"{what}: handed_over is null"
    want = {"check_id": CRATE, "repeat": 1, "machine": "p1", "items": ITEMS, "session_id": sid, "stop": STOP}
    assert ho == want, f"{what}: handed_over {ho}, expected {want}"


def at_hand_over(sc, what):
    d = dist2d(sc["pos"], 150, 150)
    assert d < 1.5, f"{what}: p1 stands {d:.2f} m from where the check put it, at {sc['pos']}"
    assert sc["crates"] >= 1, f"{what}: the server sees {sc['crates']} skdfx_Crate objects"


def passed(r, cid, what):
    a = r["answer"]
    ex = only(r["result"], cid)
    assert ex["outcome"] == "pass", f"{what}: {cid} ended {ex['outcome']} {ex.get('reason')}; {a}"
    assert a.code == 0, f"{what}: {a}"


# ================================================================ US-083

@check("US-083 MAN-01 RUN-04 run --setup ded of a check that builds the crate scene and hands p1 over exits 0, its line says handed over to p1, and after it the command prints p1, the three items numbered in order, the session id that stays up and shakedown session stop", "start", repeat=True)
def us083_printed(ctx):
    d = ded(ctx)
    a = d["first"]["answer"]
    assert a.code == 0, f"{a}: {a.stdout[-800:]} {a.stderr[-300:]}"
    text = a.stdout
    lines = text.splitlines()
    at = [i for i, ln in enumerate(lines) if CRATE in ln and re.match(r"^\s*pass\b", ln)]
    assert at, f"no pass line of {CRATE}: {text[-800:]}"
    i = at[0]
    said = lines[i] if "handed over to p1" in lines[i] else (lines[i + 1] if i + 1 < len(lines) else "")
    assert "handed over to p1" in said, \
        f"neither the check's line nor the one under it says handed over to p1: {lines[i:i + 2]}"
    after = "\n".join(lines[i + 1:])
    pos = 0
    for n, item in enumerate(ITEMS, 1):
        m = re.compile(rf"^\s*{n}\.\s+{re.escape(item)}\s*$", re.M).search(after, pos)
        assert m, f"item {n} {item!r} is not printed numbered after the earlier ones: {after}"
        pos = m.end()
    sid = (d["status"].get("session") or {}).get("id")
    assert sid and sid in after, f"the session id {sid} is not printed after the check's line: {after}"
    assert STOP in after, f"{STOP!r} is not printed: {after}"
    assert re.search(r"\bp1\b", after.split("1.")[0]), f"the machine p1 is not named before the items: {after}"
    assert AFTER not in [ln.split()[1] for ln in lines if re.match(r"^\s*(pass|fail|invalid)\s", ln)], \
        f"the check after the hand over has a line: {text}"


@check("US-083 MAN-01 RES-01 the result file of the hand over is valid, its pass execution of the crate check carries handed_over with p1 and the three items, and the check after it is not run with reason handed-over", "start", repeat=True)
def us083_result_file(ctx):
    d = ded(ctx)
    r = d["first"]["result"]
    assert r is not None, f"the command printed no result file: {d['first']['answer'].stdout[-500:]}"
    handed_over_file(ctx, r, "run --setup ded")
    handed_over_file(ctx, d["json_run"]["result"], "run --json against the session")


@check("US-083 MAN-01 AGT-02 run_result of a run_start that hands over gives handed_over with the check, repeat 1, p1, the three items, the session id and shakedown session stop, valid against run_result's outputSchema; run --json gives the same", "start", repeat=True)
def us083_run_result(ctx):
    d = ded(ctx)
    mc = d["mcp"]
    assert mc["status"]["state"] == "finished", f"run_status ended {mc['status']}"
    res = mc["result"]
    sid = (d["status_reload"].get("session") or {}).get("id")
    handed_over_answer(res.get("handed_over"), sid, "run_result")
    assert res["summary"]["pass"] == 1 and res["summary"]["not_run"] == 1 and res["summary"]["total"] == 2, \
        f"run_result summary {res['summary']}"
    assert [(n["check_id"], n["reason"]) for n in res["not_run"]] == [(AFTER, "handed-over")], f"not_run {res['not_run']}"
    sch = d["run_result_schema"]
    assert sch, "tools/list gives run_result no outputSchema"
    errors = schema.validate(res, sch)
    assert not errors, f"run_result is not valid against its outputSchema: {errors[:5]}"
    handed_over_file(ctx, read_result(res["result_file"]), "run_start")
    out = d["json_run"]["out"]
    handed_over_answer(out.get("handed_over"), sid, "run --json")
    assert d["json_run"]["answer"].code == 0, f"{d['json_run']['answer']}"


@check("US-083 MAN-01 RST-07 after each hand over p1 stands where the check put it and the server sees the crate it made: no reset followed the check", "start", repeat=True)
def us083_no_reset(ctx):
    d = ded(ctx)
    at_hand_over(d["scene_first"], "after run --setup ded")
    at_hand_over(d["scene_mcp"], "after run_start")


@check("US-083 MAN-01 p1's RPT holds the three items of the hand over (the box on p1's screen is the owner's look)", "start", repeat=True)
def us083_rpt(ctx):
    d = ded(ctx)
    a = d["logs"]
    assert a.code == 0, f"{a}: {a.stderr[-300:]}"
    for item in ITEMS:
        assert item in a.stdout, f"p1's log lines do not hold {item!r}: {a.stdout[-1500:]}"


@check("US-083 RUN-04 STP-02 SES-06 a run --setup that started its own session leaves it up after the hand over: status shows it ready on setup ded with the session id the command printed, and every machine's process runs", "start", repeat=True)
def us083_stays_up(ctx):
    d = ded(ctx)
    s = d["status"].get("session")
    assert s, f"status shows no session after the hand over: {d['status']}"
    assert s["state"] == "ready", f"the session is {s['state']}"
    assert s["setup"] == "ded" or (isinstance(s["setup"], dict) and s["setup"].get("name") == "ded"), \
        f"the session's setup {s['setup']}"
    names = sorted(m["name"] for m in s["machines"])
    assert names == ["hc1", "p1", "p2", "server"], f"machines {names}"
    for m in s["machines"]:
        assert m["pid"] in d["game_after_first"], f"{m['name']}'s process {m['pid']} does not run: {d['game_after_first']}"
    assert s["id"] in d["first"]["answer"].stdout, f"the command did not print the session id {s['id']}"
    r = d["first"]["result"]
    assert r and r["session"]["id"] == s["id"], f"the result file's session {r and r['session']['id']} is not {s['id']}"


@check("US-083 MAN-01 RST-01 the next run after a hand over takes the session back: its check starts from the baseline, with no crate left and p1 back from where the hand over put it, and passes", "start", repeat=True)
def us083_next_run(ctx):
    d = ded(ctx)
    passed(d["base_run"], BASE, "the run after run --setup's hand over")
    valid(ctx, d["base_run"]["result"], "the run after the hand over")
    assert d["base_run"]["out"].get("handed_over") is None, f"handed_over {d['base_run']['out'].get('handed_over')}"


@check("US-083 MAN-01 RST-03 shakedown reset after a hand over is done, and after it the crate is gone and p1 is back from where the hand over put it", "start", repeat=True)
def us083_reset(ctx):
    d = ded(ctx)
    a = d["reset"]
    assert a.code == 0 and a.json()["state"] == "done", f"{a}: {a.stdout[:500]}"
    sc = d["scene_reset"]
    assert sc["crates"] == 0, f"the server sees {sc['crates']} skdfx_Crate objects after the reset"
    dd = dist2d(sc["pos"], 150, 150)
    assert dd > 2, f"p1 stands {dd:.2f} m from where the hand over put it after the reset, at {sc['pos']}"


@check("US-083 MAN-01 RLD-01 shakedown reload after a hand over exits 0 and leaves the session ready, and the next run passes from the baseline", "start", repeat=True)
def us083_reload(ctx):
    d = ded(ctx)
    a = d["reload"]
    out = a.json()
    assert a.code == 0 and out["state"] == "done", f"{a}: {a.stdout[:600]}"
    s = d["status_reload"].get("session")
    assert s and s["state"] == "ready", f"the session after the reload: {s and s['state']}"
    passed(d["base_reload"], BASE, "the run after the reload")


@check("US-083 MAN-01 SES-07 STP-02 shakedown session stop after a hand over ends the session: status shows none and no game process of it runs", "start", repeat=True)
def us083_stop(ctx):
    d = ded(ctx)
    last = d["last"]
    assert last["answer"].code == 0 and last["result"] and only(last["result"], CRATE).get("handed_over"), \
        f"the last hand over: {last['answer']} {last['answer'].stdout[-400:]}"
    s = d["status_last"].get("session")
    assert s and s["state"] == "ready", f"the session after the last hand over: {s and s['state']}"
    a = d["stop"]
    assert a.code == 0, f"{a}: {a.stdout[:400]} {a.stderr[:300]}"
    assert d["status_stopped"].get("session") is None, f"status after session stop: {d['status_stopped'].get('session')}"
    assert not d["left"], f"game processes still run after session stop: {sorted(d['left'])}"


@check("US-083 RUN-07 MAN-01 run --each-setup of a check of sp that hands over and a check of ded: sp's session stays up and ready with p1 where the check put it, no dedicated server starts, the command prints the hand over and exits 0", "start", repeat=True)
def us083_each(ctx):
    e = each(ctx)
    a = e["answer"]
    assert a.code == 0, f"{a}: {a.stdout[-800:]} {a.stderr[-300:]}"
    images = [r[0] for r in e["procs"].seen.values()]
    assert images == ["arma3_x64.exe"], f"game processes seen: {e['procs'].describe()}"
    s = e["status"].get("session")
    assert s and s["state"] == "ready", f"status after the command: {s}"
    setup = s["setup"]["name"] if isinstance(s["setup"], dict) else s["setup"]
    assert setup == "sp", f"the session left up is of setup {setup}"
    dd = dist2d(e["scene"], 150, 150)
    assert dd < 1.5, f"p1 stands {dd:.2f} m from where the check put it, at {e['scene']}"
    text = a.stdout
    for n, item in enumerate(ITEMS, 1):
        assert re.search(rf"^\s*{n}\.\s+{re.escape(item)}\s*$", text, re.M), f"item {n} is not printed: {text}"
    assert s["id"] in text and STOP in text, f"the session id {s['id']} or {STOP!r} is not printed: {text}"
    sp = [r for r in e["results"] if r["session"]["setup"]["name"] == "sp"]
    assert len(sp) == 1, f"result files printed: {[r['session']['setup']['name'] for r in e['results']]}"
    valid(ctx, sp[0], "sp's result file")
    ex = only(sp[0], EACH)
    assert ex["outcome"] == "pass" and ex.get("handed_over") == {"machine": "p1", "items": ITEMS}, \
        f"{EACH}: {ex['outcome']} handed_over {ex.get('handed_over')}"
    assert [(n["check_id"], n["reason"]) for n in sp[0]["not_run"]] == [(AFTER, "handed-over")], \
        f"sp's not_run {sp[0]['not_run']}"
    assert e["stop"].code == 0 and e["status_stopped"].get("session") is None, \
        f"session stop after the command: {e['stop']}; status {e['status_stopped'].get('session')}"


HUD = {"grid_menu", "grid_comm", "grid_vehicle", "grid_weapon", "grid_stance", "grid_stamina", "grid_hint"}


def box_in_column(box, what):
    """Judges the box against MAN-01's place in grid units and against the default layout's HUD elements."""
    hits, (sx, sy, sw, sh_), hud = box
    assert len(hits) == 1, f"{what}: {len(hits)} controls on p1's displays start with 'Shakedown: look for': {hits}"
    (x, y, w, h), parent, shown = hits[0]
    assert shown, f"{what}: the box is not shown"
    assert parent == "No control", f"{what}: the box lies in a controls group {parent}; its position is relative"
    ar = min(sw / sh_, 1.2)
    gw, gh = ar / 40, ar / 1.2 / 25
    tol = 0.1 * gw
    left, right = sx + 11.5 * gw, sx + sw - 15.3 * gw
    space = right - left
    want_w = min(space, 24 * gw)
    want_x = left + (space - want_w) / 2
    assert x >= left - tol and x + w <= right + tol, (
        f"{what}: the box spans x {x:.4f} to {x + w:.4f}, outside the column {left:.4f} to {right:.4f} (11.5 units "
        f"off the safe zone's left edge, 15.3 off its right; grid unit {gw:.5f})")
    assert w <= 24 * gw + tol, f"{what}: the box is {w / gw:.2f} units wide, more than 24"
    assert abs(x - want_x) <= tol and abs(w - want_w) <= tol, (
        f"{what}: the box at x {x:.4f} wide {w:.4f} is not centred in the column: expected x {want_x:.4f} "
        f"wide {want_w:.4f}")
    assert abs(y - (sy + 0.5 * gh)) <= 0.1 * gh, (
        f"{what}: the box's top {y:.4f} is not half a unit below the safe zone's top {sy:.4f} (unit {gh:.5f})")
    names = {n for n, _ in hud}
    assert names == HUD, f"{what}: the game's config gives the default layout's {sorted(names)}, not {sorted(HUD)}"
    for name, (hx, hy, hw, hh) in hud:
        # Apart in x is apart at any height of the box; the box grows downwards with its items.
        apart = x + w <= hx + 1e-4 or hx + hw <= x + 1e-4
        assert apart, (f"{what}: the box x {x:.4f} to {x + w:.4f} overlaps the default layout's {name} "
                       f"x {hx:.4f} to {hx + hw:.4f}")


@check("US-083 MAN-01 m15-1 the look-for box on p1's screen, read through exec, lies in MAN-01's top-centre column: from 11.5 grid units off the safe zone's left edge to 15.3 off its right, at most 24 wide and centred, half a unit below the top, apart from the default layout's action menu, communication menu, vehicle info, weapon info, stance, stamina bar and hints, in dedicated and in singleplayer; the reset removes it", "start", repeat=True)
def us083_box(ctx):
    d = ded(ctx)
    ctx.log(f"m15-1 box after run --setup ded: {d['box_first']}")
    box_in_column(d["box_first"], "dedicated, after run --setup")
    assert d["box_reset"][0] == [], f"the box is still on p1's screen after shakedown reset: {d['box_reset'][0]}"
    e = each(ctx)
    ctx.log(f"m15-1 box after run --each-setup in sp: {e['box']}")
    box_in_column(e["box"], "singleplayer, after run --each-setup")


@check("US-083 MAN-01 m15-2 each of the three items holds on p1 at the hand over: one crate stands 2 m in front of p1, its action Open crate has p1 in its radius, p1 carries no first aid kit, and firing Open crate as the action menu does puts one first aid kit in p1's inventory", "start", repeat=True)
def us083_items_hold(ctx):
    d = ded(ctx)
    it = d["items"]
    b = it["before"]
    assert b[0] == 1, f"p1 sees {b[0]} skdfx_Crate within 10 m: {b}"
    _, dist, rel, ids, radius, kits = b
    assert 1.5 <= dist <= 2.5, f"item 1: the crate stands {dist:.2f} m from p1, not 2 m"
    assert rel <= 30 or rel >= 330, f"item 1: the crate is {rel:.0f} degrees off p1's facing, not in front"
    assert len(ids) == 1, f"item 2: the crate carries {len(ids)} actions titled Open crate on p1: {ids}"
    assert dist < radius, f"item 2: p1 stands {dist:.2f} m from the crate, outside the action's radius {radius}"
    assert kits == 0, f"item 3: p1 carries {kits} first aid kits before opening the crate"
    assert it["kits"] and it["kits"][-1] == 1, (
        f"item 3: after firing Open crate p1 carries first aid kits {it['kits']} (polled about every 0.5 s)")


# ================================================================ US-084

def refused(ctx, cid, machine):
    d = ded(ctx)
    r = d["refused"]
    valid(ctx, r["result"], "the run of the refused hand overs")
    ex = only(r["result"], cid)
    reason = ex.get("reason") or {}
    assert ex["outcome"] == "fail" and reason.get("kind") == "check-error", f"{cid}: {ex['outcome']} {reason}"
    msg = reason.get("message", "")
    for name in (machine, "p1", "p2"):
        assert re.search(rf"\b{name}\b", msg), f"{cid}: the message does not name {name}: {msg!r}"
    assert "skd_fnc_handOver" in msg, f"{cid}: the message does not name skd_fnc_handOver: {msg!r}"
    assert not ex.get("handed_over"), f"{cid}: handed_over {ex.get('handed_over')}"
    ctx.log(f"US-084 {cid}: {msg}")


@check("US-084 MAN-01 DM-04 skd_fnc_handOver on server on a dedicated session with two clients and a headless client ends fail check-error naming skd_fnc_handOver, server, p1 and p2, and is not handed over", "start", repeat=True)
def us084_server(ctx):
    refused(ctx, SERVER, "server")


@check("US-084 MAN-01 DM-04 skd_fnc_handOver on hc1 on a dedicated session with two clients and a headless client ends fail check-error naming skd_fnc_handOver, hc1, p1 and p2, and is not handed over", "start", repeat=True)
def us084_hc1(ctx):
    refused(ctx, HC1, "hc1")


@check("US-084 MAN-01 RST-01 after the two refused hand overs the run goes on with its reset: the next check starts from the baseline and passes, the run has no handed_over and nothing not run, exits 1, and the session stays ready", "start", repeat=True)
def us084_goes_on(ctx):
    d = ded(ctx)
    r = d["refused"]
    a, out, res = r["answer"], r["out"], r["result"]
    assert [c["check_id"] for c in res["checks"]] == [SERVER, HC1, BASE], f"executed {[c['check_id'] for c in res['checks']]}"
    ex = only(res, BASE)
    assert ex["outcome"] == "pass", f"{BASE} after the refused hand overs: {ex['outcome']} {ex.get('reason')}"
    assert out.get("handed_over") is None, f"handed_over {out.get('handed_over')}"
    assert res["not_run"] == [], f"not_run {res['not_run']}"
    assert a.code == 1, f"{a}: expected exit 1 for a run with a fail"
    s = d["status_refused"].get("session")
    assert s and s["state"] == "ready", f"the session after the run: {s and s['state']}"
