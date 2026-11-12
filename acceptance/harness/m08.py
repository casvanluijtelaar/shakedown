"""The checks of M8 Walk away: US-033, US-034, US-036 and US-063.

Two parts:

- The start checks of M8 run once per repeat (`--repeat`), each with no session running, each in a
  project of its own in the work folder: `walk` holds the three checks of `acceptance/walk/` and the
  setup `dedicated-2` (a dedicated server with two clients loading @skd_fixture alone); `ceil` holds the
  eight ceiling checks of `templates/US-063-ceiling.sqf` and the setup `sp`. Per repeat they drive:
  `run --setup dedicated-2` to its end (US-033), the same run stopped with Ctrl+C in its second check
  (US-034), `run --setup sp` of the ceiling checks (US-063), and `session start` then `session stop`, and
  `session_start` then `session_stop` through MCP (US-036).
- The sp batch drives the running singleplayer session of the generated project, per repeat: three
  checks stopped during the second by Ctrl+C, by Ctrl+Break, by `run stop` and by `run_stop` (US-034),
  and the eight ceiling checks run from the command line and through MCP (US-063).

Ctrl+C and Ctrl+Break go to a process this module starts, through console_signal.py in a console of
its own, never to the desktop. Every game process a part started is stopped by its id if it outlives it.
"""

import json
import os
import shutil
import subprocess
import sys
import time
import traceback
from datetime import datetime
from pathlib import Path

from . import driver
from .core import batch, check
from .m06 import finish, starts, states, text_of, utc
from .mcp import Mcp, structured

HERE = Path(__file__).resolve().parent
ACC = HERE.parent
FIRST, SLOW, THIRD = "m08/US-034-1-first", "m08/US-034-2-slow", "m08/US-034-3-third"
STOP3 = [FIRST, SLOW, THIRD]
CEIL = [f"m08/US-063-ceiling-{n}" for n in range(1, 9)]
WALK = ["US-033-1-first", "US-033-2-slow", "US-033-3-third"]
WALK_SLOW = "US-033-2-slow"
CEIL_OWN = [f"US-063-ceiling-{n}" for n in range(1, 9)]
BASE_PORT = 2402            # docs/project-file.md: the default base_port
CEILING_S = 8 * 1 + 60      # RUN-06: the sum of eight timeouts of 1 s, plus 60 s
CREATE_NEW_CONSOLE = 0x00000010
STARTF_USESHOWWINDOW, SW_HIDE = 0x1, 0


def write_ceiling_checks(folder, setups=None):
    template = (ACC / "templates" / "US-063-ceiling.sqf").read_text(encoding="utf-8")
    folder.mkdir(parents=True, exist_ok=True)
    for n in range(1, 9):
        text = template.replace("{N}", str(n))
        if setups:
            text = text.replace("// @timeout 1", f"// @setups {setups}\n// @timeout 1")
        (folder / f"US-063-ceiling-{n}.sqf").write_text(text, encoding="utf-8")


def project_files(project):
    """The eight ceiling checks in the generated project, for the sp session."""
    write_ceiling_checks(Path(project) / "checks" / "m08", setups="sp")


# ================================================================ helpers

def game_folder(project):
    for line in (Path(project) / "shakedown.toml").read_text(encoding="utf-8").splitlines():
        if line.startswith("game = "):
            return Path(line.split("=", 1)[1].strip().strip("'\""))
    raise AssertionError("the shakedown.toml has no game line")


def mission_copies(game, sid):
    folder = Path(game) / "MPMissions"
    return sorted(p.name for p in folder.glob(f"*{sid}*")) if sid and folder.is_dir() else []


def ports_held(base=BASE_PORT):
    """[(proto, port, pid)] of every socket a process holds on a port from base to base + 4."""
    out = subprocess.run(["netstat", "-ano"], capture_output=True, text=True, timeout=60).stdout
    held = []
    for line in out.splitlines():
        parts = line.split()
        if len(parts) < 4 or parts[0] not in ("TCP", "UDP"):
            continue
        try:
            port, pid = int(parts[1].rsplit(":", 1)[1]), int(parts[-1])
        except ValueError:
            continue
        if base <= port <= base + 4 and pid != 0:
            held.append((parts[0], port, pid))
    return sorted(set(held))


def status_of(sh):
    return sh("status", "--json", ceiling=30).json()


def pids_of(st):
    return {m["pid"] for m in (st.get("session") or {}).get("machines", []) if m.get("pid")}


def steam_running():
    return bool(driver.pids("steam.exe"))


def need_quiet(sh):
    """M8's own-session parts need no game and no session."""
    assert steam_running(), "no steam process runs; a game start would be refused"
    busy = driver.game_pids()
    assert not busy, f"game processes run before the part starts: {sorted(busy)}"
    st = status_of(sh)
    assert st.get("session") is None, f"status shows a session before the part starts: {(st['session'] or {}).get('id')}"


def stop_leftovers(ctx, sh, pids):
    """Stops a session left by a part that failed and every game process it started, by id."""
    try:
        st = status_of(sh)
        if st.get("session"):
            ctx.log("M8: a session is left; stopping it")
            sh("session", "stop", "--json", ceiling=180)
    except Exception as e:  # noqa: BLE001
        ctx.log(f"M8: status or session stop after the part: {e!r}")
    for pid in set(pids):
        if driver.alive(pid) and driver.wait_gone(pid, 30) is None:
            ctx.log(f"M8: game process {pid} still runs; stopping it by id")
            driver.kill(pid)


def run_folder_of(sh, before, out):
    if isinstance(out, dict) and out.get("result_file"):
        return Path(out["result_file"]).parent
    new = sorted(set(sh.run_folders()) - set(before))
    assert len(new) == 1, f"{len(new)} new run folders, expected 1: {new}"
    return sh.project / ".shakedown" / "runs" / new[0]


def read_result(folder):
    p = Path(folder) / "result.json"
    assert p.is_file(), f"the run folder {folder} holds no result.json"
    return json.loads(p.read_text(encoding="utf-8"))


def watch(ctx, sh, alive, ceiling, until=None):
    """Polls status twice a second while alive() holds, at most `ceiling` s, collecting every game
    process id and session id it shows. Returns (pids, session ids, the status that met until, or None)."""
    pids, sids, met = set(), set(), None
    for _ in range(int(ceiling * 2)):
        if not alive():
            break
        try:
            st = status_of(sh)
        except AssertionError:
            st = {}
        pids |= pids_of(st)
        if st.get("session"):
            sids.add(st["session"]["id"])
        if until and met is None and until(st):
            met = st
            break
        time.sleep(0.5)
    return pids, sids, met


class Console:
    """`shakedown <args>` in a hidden console of its own, through console_signal.py, which sends it
    Ctrl+C (mode c) or Ctrl+Break to its process group (mode break) when `send` is called."""

    def __init__(self, ctx, project, exe, mode, args, tag):
        d = ctx.work / "m08" / tag
        d.mkdir(parents=True, exist_ok=True)
        self.go, self.state, self.out, self.err = d / "go", d / "state.json", d / "stdout.txt", d / "stderr.txt"
        for p in (self.go, self.state):
            p.unlink(missing_ok=True)
        si = subprocess.STARTUPINFO()
        si.dwFlags |= STARTF_USESHOWWINDOW
        si.wShowWindow = SW_HIDE
        self.proc = subprocess.Popen(
            [sys.executable, "-I", str(HERE / "console_signal.py"), mode, str(self.go), str(self.state),
             str(self.out), str(self.err), str(exe), *args],
            cwd=project, creationflags=CREATE_NEW_CONSOLE, startupinfo=si)
        self.args = list(args)
        ctx.log(f"M8: `shakedown {' '.join(args)}` started in a console of its own, helper pid {self.proc.pid}, mode {mode}")

    def alive(self):
        return self.proc.poll() is None

    def send(self):
        self.sent = time.time()
        self.go.write_text("go")

    def finish(self, ceiling=400):
        try:
            self.proc.wait(timeout=ceiling)
        except subprocess.TimeoutExpired:
            st = self.read_state()
            for pid in ([st["pid"]] if st.get("pid") else []) + [self.proc.pid]:
                driver.kill(pid)
            raise AssertionError(f"`shakedown {' '.join(self.args)}` did not end within {ceiling} s; stopped it by id")
        st = self.read_state()
        out = self.out.read_text(encoding="utf-8", errors="replace")
        err = self.err.read_text(encoding="utf-8", errors="replace")
        return st, driver.Answer(self.args, st.get("code"), out, err, 0.0)

    def read_state(self):
        try:
            return json.loads(self.state.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            return {}


# ================================================================ judging helpers

def judge_stopped(ctx, result, reason, ran, stopped, not_run, what):
    """A stopped run: `ran` passed, `stopped` invalid / run-stopped, `not_run` listed run-stopped."""
    errors = ctx.validate(result)
    assert not errors, f"{what}: the result file is not valid against result.schema.json: {errors[:5]}"
    run = result["run"]
    assert run["state"] == "stopped", f"{what}: run.state {run['state']!r}, expected stopped"
    assert run["stopped_reason"] == reason, f"{what}: run.stopped_reason {run['stopped_reason']!r}, expected {reason}"
    got = [(e["check_id"], e["outcome"], (e["reason"] or {}).get("kind")) for e in result["checks"]]
    want = [(c, "pass", None) for c in ran] + [(stopped, "invalid", "run-stopped")]
    assert got == want, f"{what}: the executions are {got}, expected {want}"
    nr = [(n["check_id"], n["reason"]) for n in result["not_run"]]
    assert nr == [(c, "run-stopped") for c in not_run], f"{what}: not_run {nr}, expected {not_run} with run-stopped"
    s = result["summary"]
    want_s = {"total": len(ran) + 1 + len(not_run), "pass": len(ran), "fail": 0, "invalid": 1, "not_run": len(not_run)}
    assert s == want_s, f"{what}: summary {s}, expected {want_s}"


def first_started(result):
    return utc(result["checks"][0]["started_at"])


# ================================================================ own-session parts (start, per repeat)

def machine_fault(ctx, since):
    """The `Error 20 reading file` line of an RPT written since `since` in an M8 project, or None."""
    for name in ("walk", "ceil"):
        root = ctx.work / name / ".shakedown" / "sessions"
        for rpt in root.glob("*/machines/*/*.rpt") if root.is_dir() else []:
            if rpt.stat().st_mtime >= since - 1:
                for line in rpt.read_text(encoding="utf-8", errors="replace").splitlines():
                    if "Error 20 reading file" in line:
                        return line.strip()[:200]
    return None


def keep(ctx):
    if not hasattr(ctx, "m08_keep"):
        ctx.m08_keep = {}
    return ctx.m08_keep


def once(ctx, key, fn):
    """Runs fn once per repeat and keeps its value or its error; the checks of that repeat share it."""
    store = keep(ctx)
    k = (key, getattr(ctx, "repeat_n", 1))
    if k not in store:
        for attempt in range(1, 4):
            began = time.time()
            try:
                store[k] = fn(ctx)
                break
            except Exception as e:  # noqa: BLE001
                ctx.log(f"M8 step {k}: {traceback.format_exc()}")
                store[k] = e
                fault = machine_fault(ctx, began)
                if not fault or attempt == 3:
                    break
                ctx.log(f"M8 step {k}: a game process could not read a base-game file ({fault}); the machine, "
                        f"not the framework (playbook). Starting the part again, attempt {attempt + 1} of 3")
    value = store[k]
    if isinstance(value, Exception):
        raise AssertionError(f"the M8 step {key} failed: {value!r}")
    return value


def own_project(ctx, name):
    """The project `walk` (setup dedicated-2, the three checks of acceptance/walk) or `ceil` (setup sp, the
    eight ceiling checks), made once per suite run, with the .mcp.json of docs/agent-tools.md."""
    store = keep(ctx)
    if name in store:
        return store[name]
    game = game_folder(ctx.project)
    project = ctx.work / name
    if project.exists():
        shutil.rmtree(project)
    checks = project / "checks"
    if name == "walk":
        shutil.copytree(ACC / "walk", checks)
        setup = "[setups.dedicated-2]\nmode = \"dedicated\"\nclients = 2\nheadless = 0\nmods = [\"fixture\"]\n"
    else:
        write_ceiling_checks(checks)
        setup = "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n"
    (project / "shakedown.toml").write_text(
        f"game = '{game}'\n\n[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n\n" + setup,
        encoding="utf-8")
    (project / ".mcp.json").write_text(json.dumps(
        {"mcpServers": {"shakedown": {"command": "shakedown", "args": ["mcp"], "timeout": 70000}}}, indent=2),
        encoding="utf-8")
    # A git repository of its own, so `git status` shows what a run writes into the project (CFG-02).
    subprocess.run(["git", "init", "-q"], cwd=project, capture_output=True, timeout=60)
    sh = driver.Shakedown(ctx.sh.exe, project, ctx.log)
    store[name] = (project, sh, game)
    return store[name]


def git_status(project):
    return subprocess.run(["git", "status", "--porcelain", "--untracked-files=all"], cwd=project,
                          capture_output=True, text=True, timeout=60).stdout.splitlines()


def files_outside_shakedown(project):
    return sorted(p.relative_to(project).as_posix() for p in Path(project).rglob("*")
                  if p.is_file() and not p.relative_to(project).parts[0] in (".shakedown", ".git"))


def after_state(sh, game, sids):
    return {"games": driver.game_pids(), "status": status_of(sh), "ports": ports_held(),
            "copies": [c for sid in sids for c in mission_copies(game, sid)]}


def full_run(ctx):
    project, sh, game = own_project(ctx, "walk")
    need_quiet(sh)
    out = {"git_before": git_status(project), "files_before": files_outside_shakedown(project)}
    before = sh.run_folders()
    args = ("run", "--setup", "dedicated-2", "--json")
    bg = sh.start(*args)
    pids = set()
    try:
        pids, sids, _ = watch(ctx, sh, lambda: bg.poll() is None, 600)
        out["answer"] = finish(ctx, bg, args, ceiling=120)
        out["pids"], out["sids"] = pids, sids
        out.update(after_state(sh, game, sids))
        out["folder"] = run_folder_of(sh, before, out["answer"].json() if out["answer"].stdout.strip() else None)
        out["git_after"] = git_status(project)
        out["files_after"] = files_outside_shakedown(project)
        gi = project / ".shakedown" / ".gitignore"
        out["gitignore"] = gi.read_text(encoding="utf-8") if gi.is_file() else None
        ctx.log(f"M8 US-033 full run: {out['answer']}; status session {out['status'].get('session')}; "
                f"games {sorted(out['games'])}; ports {out['ports']}; copies {out['copies']}")
        return out
    finally:
        stop_leftovers(ctx, sh, pids)


def ctrlc_own(ctx):
    project, sh, game = own_project(ctx, "walk")
    need_quiet(sh)
    before = sh.run_folders()
    con = Console(ctx, project, ctx.sh.exe, "c", ["run", "--setup", "dedicated-2", "--json"], f"own-c-{ctx.repeat_n}")
    pids = set()
    try:
        pids, sids, met = watch(ctx, sh, con.alive, 400,
                                until=lambda st: (st.get("run") or {}).get("current_check") == WALK_SLOW)
        out = {"met": met}
        if met:
            con.send()
        state, answer = con.finish(300)
        more, more_sids, _ = watch(ctx, sh, lambda: False, 1)
        out.update(state=state, answer=answer, pids=pids | pids_of(met or {}), sids=sids)
        out.update(after_state(sh, game, sids))
        out["folder"] = run_folder_of(sh, before, None)
        ctx.log(f"M8 US-034 own Ctrl+C: exit {answer.code}, sent_at {state.get('sent_at')}; games {sorted(out['games'])}; "
                f"status session {out['status'].get('session')}")
        return out
    finally:
        stop_leftovers(ctx, sh, pids)


def ceiling_own(ctx):
    project, sh, game = own_project(ctx, "ceil")
    need_quiet(sh)
    before = sh.run_folders()
    args = ("run", "--setup", "sp", "--json")
    bg = sh.start(*args)
    pids = set()
    try:
        pids, sids, _ = watch(ctx, sh, lambda: bg.poll() is None, 600)
        out = {"answer": finish(ctx, bg, args, ceiling=120), "pids": pids, "sids": sids}
        out.update(after_state(sh, game, sids))
        out["folder"] = run_folder_of(sh, before, out["answer"].json() if out["answer"].stdout.strip() else None)
        ctx.log(f"M8 US-063 own sp ceiling run: {out['answer']}; games {sorted(out['games'])}")
        return out
    finally:
        stop_leftovers(ctx, sh, pids)


def stop_cli(ctx):
    project, sh, game = own_project(ctx, "walk")
    need_quiet(sh)
    pids = set()
    try:
        start = sh("session", "start", "--setup", "dedicated-2", "--json", ceiling=400)
        session = start.json()
        pids = {m["pid"] for m in session["machines"] if m.get("pid")}
        assert start.code == 0 and session["state"] == "ready", f"{start}: state {session.get('state')}"
        out = {"session": session, "ports_during": ports_held(), "copies_during": mission_copies(game, session["id"])}
        out["stop"] = sh("session", "stop", "--json", ceiling=180)
        out["alive"] = {p for p in pids if driver.alive(p)}
        out.update(after_state(sh, game, [session["id"]]))
        ctx.log(f"M8 US-036 session stop: {out['stop']}; alive {out['alive']}; ports {out['ports']} "
                f"(during {out['ports_during']}); copies {out['copies']} (during {out['copies_during']})")
        return out
    finally:
        stop_leftovers(ctx, sh, pids)


def stop_mcp(ctx):
    project, sh, game = own_project(ctx, "walk")
    need_quiet(sh)
    pids = set()
    try:
        with Mcp(project, ctx.install, ctx.log) as m:
            res, _ = m.call("session_start", {"setup": "dedicated-2"}, ceiling=70)
            session = structured(res)
            if session["state"] == "starting":
                _, _, met = watch(ctx, sh, lambda: True, 300,
                                  until=lambda st: (st.get("session") or {}).get("state") in ("ready", "failed"))
                session = (met or {}).get("session") or session
            pids = {mm["pid"] for mm in session["machines"] if mm.get("pid")}
            assert session["state"] == "ready", f"session_start: state {session['state']}, reason {session.get('reason')}"
            out = {"session": session, "ports_during": ports_held(), "copies_during": mission_copies(game, session["id"])}
            res, secs = m.call("session_stop", {}, ceiling=200)
            out["stop"], out["stop_s"] = structured(res), secs
            out["alive"] = {p for p in pids if driver.alive(p)}
            out.update(after_state(sh, game, [session["id"]]))
            res, _ = m.call("status", {}, ceiling=30)
            out["status_tool"] = structured(res)
        ctx.log(f"M8 US-036 session_stop: state {out['stop'].get('state')} in {out['stop_s']:.1f} s; alive {out['alive']}; "
                f"ports {out['ports']}; copies {out['copies']}")
        return out
    finally:
        stop_leftovers(ctx, sh, pids)


def no_game_no_session(v, what):
    assert not v["games"], f"{what}: game processes run after the run ended: {sorted(v['games'])}"
    assert v["status"].get("session") is None, \
        f"{what}: status shows a session after the run ended: {(v['status']['session'] or {}).get('state')}"
    assert v["pids"], f"{what}: status never showed a game process during the run, so the session was not watched"


@check("US-033 RUN-04 STP-02 run --setup dedicated-2 with no session starts one, runs every check of the project, stops it and exits 0; then no arma3_x64 or arma3server_x64 process runs and status shows no session", "start", repeat=True)
def us033_run(ctx):
    v = once(ctx, "full", full_run)
    a = v["answer"]
    assert a.code == 0, f"{a}: {text_of(a)[-600:]}"
    r = read_result(v["folder"])
    ids = [e["check_id"] for e in r["checks"]]
    assert ids == WALK, f"the run executed {ids}, expected every check of the project {WALK}"
    bad = [(e["check_id"], e["outcome"], e["reason"]) for e in r["checks"] if e["outcome"] != "pass"]
    assert not bad, f"checks that did not pass: {bad}"
    assert r["run"]["state"] == "finished", f"run.state {r['run']['state']}"
    no_game_no_session(v, "run --setup dedicated-2")


@check("US-033 STP-04 STP-02 after run --setup dedicated-2 no port from the base port to the base port plus 4 is held and no mission copy of its session is left in MPMissions", "start", repeat=True)
def us033_ports(ctx):
    v = once(ctx, "full", full_run)
    assert v["sids"], "status never showed the run's session"
    assert not v["ports"], f"ports held after the run (proto, port, pid): {v['ports']}"
    assert not v["copies"], f"mission copies left in MPMissions: {v['copies']}"


@check("US-033 RES-01 RES-04 LOG-04 the run folder holds one valid result.json, logs/<machine>.log for each machine and checks/<n>/<machine>.log for each check execution, and every path in result.json is relative to the run folder and exists", "start", repeat=True)
def us033_folder(ctx):
    v = once(ctx, "full", full_run)
    folder = v["folder"]
    results = list(Path(folder).rglob("result.json"))
    assert len(results) == 1, f"the run folder holds {len(results)} result.json files"
    r = read_result(folder)
    errors = ctx.validate(r)
    assert not errors, f"the result file is not valid against result.schema.json: {errors[:5]}"
    machines = sorted(m["name"] for m in r["session"]["machines"])
    assert machines == ["p1", "p2", "server"], f"the session's machines are {machines}"
    missing = [f"logs/{m}.log" for m in machines if not (folder / "logs" / f"{m}.log").is_file()]
    missing += [f"checks/{n}/{m}.log" for n in range(1, len(r["checks"]) + 1) for m in machines
                if not (folder / "checks" / str(n) / f"{m}.log").is_file()]
    assert not missing, f"log files missing from the run folder: {missing}"
    paths = list(r["logs"].values()) + [p for e in r["checks"] for p in e["evidence"]["logs"].values()]
    bad = [p for p in paths if os.path.isabs(p) or "\\" in p or not (folder / p).is_file()]
    assert paths and not bad, f"paths in result.json that are absolute or do not exist: {bad}"


@check("US-033 CFG-02 NFR-PV-03 .shakedown/.gitignore ignores everything in .shakedown and the framework wrote nothing else into the project", "start", repeat=True)
def us033_gitignore(ctx):
    v = once(ctx, "full", full_run)
    assert v["gitignore"] is not None, ".shakedown/.gitignore does not exist"
    assert v["gitignore"].strip() == "*", f".shakedown/.gitignore holds {v['gitignore']!r}, expected *"
    assert v["git_after"] == v["git_before"], \
        f"git status changed: before {v['git_before']}, after {v['git_after']}"
    assert v["files_after"] == v["files_before"], \
        f"files outside .shakedown changed: new {sorted(set(v['files_after']) - set(v['files_before']))}"


@check("US-034 RUN-05 RUN-04 STP-02 Ctrl+C on run --setup dedicated-2 during its second check stops it invalid run-stopped, the third not run, exit 2, and leaves no game process, no port held and no session", "start", repeat=True)
def us034_own(ctx):
    v = once(ctx, "ctrlc-own", ctrlc_own)
    assert v["met"], "status never showed the run at its second check"
    assert isinstance(v["state"].get("sent_at"), float), f"Ctrl+C was not sent: {v['state']}"
    assert v["answer"].code == 2, f"`shakedown run --setup dedicated-2` exit {v['answer'].code}: {text_of(v['answer'])[-600:]}"
    judge_stopped(ctx, read_result(v["folder"]), "by-hand", WALK[:1], WALK_SLOW, WALK[2:], "the stopped run")
    no_game_no_session(v, "the run stopped by Ctrl+C")
    assert not v["ports"], f"ports held after the run (proto, port, pid): {v['ports']}"
    assert not v["copies"], f"mission copies left in MPMissions: {v['copies']}"


@check("US-063 RUN-06 STP-02 run --setup sp of the eight ceiling checks with no session stops at its ceiling, exits 2 with stopped_reason run-ceiling, and leaves no arma3_x64 process and status shows no session", "start", repeat=True)
def us063_own(ctx):
    v = once(ctx, "ceil-own", ceiling_own)
    a = v["answer"]
    assert a.code == 2, f"{a}: {text_of(a)[-600:]}"
    r = read_result(v["folder"])
    errors = ctx.validate(r)
    assert not errors, f"the result file is not valid: {errors[:5]}"
    assert r["run"]["state"] == "stopped" and r["run"]["stopped_reason"] == "run-ceiling", \
        f"run.state {r['run']['state']}, stopped_reason {r['run']['stopped_reason']}"
    no_game_no_session(v, "run --setup sp")
    sp = driver.pids("arma3_x64.exe")
    assert not sp, f"arma3_x64 processes run: {sorted(sp)}"


@check("US-036 SES-07 STP-04 session stop of dedicated-2 exits 0 and leaves no game process of the session, no port from the base port to the base port plus 4 held and no mission copy in MPMissions", "start", repeat=True)
def us036_cli(ctx):
    v = once(ctx, "stop-cli", stop_cli)
    assert v["ports_during"], "netstat showed no port of the session while it ran, so the port check sees nothing"
    assert v["copies_during"], "MPMissions held no mission copy of the session while it ran, so the copy check sees nothing"
    assert v["stop"].code == 0, f"{v['stop']}: {text_of(v['stop'])[-400:]}"
    assert not v["alive"], f"game processes of the session alive when session stop returned: {sorted(v['alive'])}"
    assert not v["games"], f"game processes run after session stop: {sorted(v['games'])}"
    assert not v["ports"], f"ports held after session stop (proto, port, pid): {v['ports']}"
    assert not v["copies"], f"mission copies left in MPMissions: {v['copies']}"
    assert v["status"].get("session") is None, f"status shows a session after session stop: {v['status']['session']}"


@check("US-036 SES-07 STP-04 session_stop through MCP returns the session with state stopped, status then shows no session, and no game process, port or mission copy of it is left", "start", repeat=True)
def us036_mcp(ctx):
    v = once(ctx, "stop-mcp", stop_mcp)
    assert v["stop"].get("state") == "stopped", f"session_stop returned state {v['stop'].get('state')}"
    assert v["stop"].get("id") == v["session"]["id"], f"session_stop returned {v['stop'].get('id')}, started {v['session']['id']}"
    assert v["status_tool"].get("session") is None, f"the status tool shows a session: {v['status_tool']['session']}"
    assert v["status"].get("session") is None, f"shakedown status shows a session: {v['status']['session']}"
    assert not v["alive"] and not v["games"], f"game processes after session_stop: {sorted(v['alive'] | v['games'])}"
    assert not v["ports"], f"ports held after session_stop (proto, port, pid): {v['ports']}"
    assert not v["copies"], f"mission copies left in MPMissions: {v['copies']}"


# ================================================================ the sp batch (existing session, per repeat)

def m8_step(ctx, key, fn):
    try:
        ctx.m08[key] = fn()
    except Exception as e:  # noqa: BLE001 - the checks that read this key fail with it
        ctx.m08[key] = e
        ctx.log(f"M8 batch step {key}: {traceback.format_exc()}")


def got(ctx, key):
    assert key in getattr(ctx, "m08", {}), f"the M8 batch did not reach {key}"
    value = ctx.m08[key]
    if isinstance(value, Exception):
        raise AssertionError(f"the M8 batch step {key} failed: {value!r}")
    return value


def at_slow(st):
    return (st.get("run") or {}).get("current_check") == SLOW


def by_console(ctx, mode):
    before = ctx.sh.run_folders()
    con = Console(ctx, ctx.project, ctx.sh.exe, mode, ["run", *STOP3, "--json"], f"sp-{mode}-{ctx.repeat_n}")
    _, _, met = watch(ctx, ctx.sh, con.alive, 120, until=at_slow)
    if met:
        con.send()
    state, answer = con.finish(200)
    folder = run_folder_of(ctx.sh, before, None)
    out = {"met": met, "state": state, "answer": answer, "result": read_result(folder), "after": status_of(ctx.sh)}
    ctx.log(f"M8 sp {mode}: exit {answer.code}, sent_at {state.get('sent_at')}, stdout {answer.stdout[:300]!r}")
    return out


def by_run_stop(ctx):
    before = ctx.sh.run_folders()
    args = (*STOP3, "--json")
    bg = ctx.sh.start("run", *args)
    _, _, met = watch(ctx, ctx.sh, lambda: bg.poll() is None, 120, until=at_slow)
    out = {"met": met}
    if met:
        out["stop"] = ctx.sh("run", "stop", "--json", ceiling=120)
        new = sorted(set(ctx.sh.run_folders()) - set(before))
        out["file_at_return"] = len(new) == 1 and (ctx.sh.project / ".shakedown" / "runs" / new[0] / "result.json").is_file()
    out["answer"] = finish(ctx, bg, ("run", *args), ceiling=200)
    out["result"] = read_result(run_folder_of(ctx.sh, before, None))
    out["again"] = ctx.sh("run", "stop", "--json", ceiling=60)
    out["after"] = status_of(ctx.sh)
    return out


def by_tool(ctx, m):
    before = ctx.sh.run_folders()
    res, _ = m.call("run_start", {"ids": STOP3})
    rid = structured(res)["run_id"]
    met = None
    for _ in range(240):
        st = structured(m.call("run_status", {"run_id": rid})[0])
        if st.get("current_check") == SLOW:
            met = st
            break
        if st["state"] in ("finished", "stopped"):
            break
        time.sleep(0.5)
    out = {"met": met, "run_id": rid}
    if met:
        res, out["stop_s"] = m.call("run_stop", {"run_id": rid}, ceiling=70)
        out["stop"] = structured(res)
        new = sorted(set(ctx.sh.run_folders()) - set(before))
        out["file_at_return"] = len(new) == 1 and (ctx.sh.project / ".shakedown" / "runs" / new[0] / "result.json").is_file()
    out["run_result"] = structured(m.call("run_result", {"run_id": rid})[0])
    out["result"] = read_result(run_folder_of(ctx.sh, before, None))
    out["after"] = status_of(ctx.sh)
    return out


def ceil_cli(ctx):
    answer, out, result = ctx.run(CEIL, ceiling=300)
    return {"answer": answer, "out": out, "result": result, "after": status_of(ctx.sh)}


def ceil_tool(ctx, m):
    res, _ = m.call("run_start", {"ids": CEIL})
    rid = structured(res)["run_id"]
    st = None
    for _ in range(8):
        st = structured(m.call("run_status", {"run_id": rid, "wait_s": 50}, ceiling=70)[0])
        if st["state"] in ("finished", "stopped"):
            break
    rr = structured(m.call("run_result", {"run_id": rid})[0])
    result = json.loads(Path(rr["result_file"]).read_text(encoding="utf-8"))
    return {"status": st, "run_result": rr, "result": result, "after": status_of(ctx.sh)}


@batch("sp")
def m08_batch(ctx, checks):
    ctx.m08 = {}
    if not any(c.name.split(" ")[0] in ("US-034", "US-063") and c.setup == "sp" for c in checks):
        return
    ctx.repeat_n = getattr(ctx, "sp_repeat", 0) + 1
    ctx.sp_repeat = ctx.repeat_n
    ctx.m08["before"] = starts(status_of(ctx.sh))
    m8_step(ctx, "ctrlc", lambda: by_console(ctx, "c"))
    m8_step(ctx, "break", lambda: by_console(ctx, "break"))
    m8_step(ctx, "runstop", lambda: by_run_stop(ctx))
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        m8_step(ctx, "tool", lambda: by_tool(ctx, m))
        m8_step(ctx, "ceil_cli", lambda: ceil_cli(ctx))
        m8_step(ctx, "ceil_tool", lambda: ceil_tool(ctx, m))


def spcheck(name):
    return check(name, "session", needs=STOP3 + CEIL, setup="sp")


def stopped_by_hand(ctx, v, what):
    assert v["met"], f"{what}: status never showed the run at {SLOW}"
    judge_stopped(ctx, v["result"], "by-hand", [FIRST], SLOW, [THIRD], what)


@spcheck("US-034 RUN-05 RES-01 Ctrl+C on shakedown run during the second of three checks ends it invalid run-stopped, lists the third in not_run run-stopped, writes a valid result file with run.state stopped and stopped_reason by-hand, and exits 2")
def us034_ctrlc(ctx):
    v = got(ctx, "ctrlc")
    assert isinstance(v["state"].get("sent_at"), float), f"Ctrl+C was not sent: {v['state']}"
    stopped_by_hand(ctx, v, "Ctrl+C")
    assert v["answer"].code == 2, f"exit {v['answer'].code}: {text_of(v['answer'])[-500:]}"


@spcheck("US-034 RUN-05 Ctrl+Break to the process group of shakedown run during the second of three checks stops the run as Ctrl+C does, exit 2")
def us034_break(ctx):
    v = got(ctx, "break")
    assert isinstance(v["state"].get("sent_at"), float), f"Ctrl+Break was not sent: {v['state']}"
    stopped_by_hand(ctx, v, "Ctrl+Break")
    assert v["answer"].code == 2, f"exit {v['answer'].code}: {text_of(v['answer'])[-500:]}"


@spcheck("US-034 RUN-05 shakedown run stop from another console during the second of three checks returns once the result file is written with state stopped, the run exits 2, and with no run active run stop exits 4")
def us034_run_stop(ctx):
    v = got(ctx, "runstop")
    stopped_by_hand(ctx, v, "run stop")
    s = v["stop"]
    assert s.code == 0, f"{s}: {text_of(s)[-400:]}"
    assert s.json().get("state") == "stopped", f"run stop printed state {s.json().get('state')}"
    assert v["file_at_return"], "the result file did not exist when run stop returned"
    assert v["answer"].code == 2, f"the stopped run: {v['answer']}"
    assert v["again"].code == 4, f"run stop with no run active: {v['again']}, expected exit 4"


@spcheck("US-034 RUN-05 run_stop through MCP during the second of three checks returns state stopped once the result file is written, and the result file says what Ctrl+C gives")
def us034_tool(ctx):
    v = got(ctx, "tool")
    stopped_by_hand(ctx, v, "run_stop")
    assert v["stop"]["state"] == "stopped", f"run_stop returned state {v['stop']['state']}"
    assert v["file_at_return"], "the result file did not exist when run_stop returned"
    assert v["run_result"]["state"] == "stopped", f"run_result state {v['run_result']['state']}"
    assert v["run_result"]["summary"] == v["result"]["summary"], \
        f"run_result summary {v['run_result']['summary']}, the file's {v['result']['summary']}"


@spcheck("US-034 STP-02 a run against an existing session stopped by Ctrl+C, Ctrl+Break, run stop and run_stop leaves that session ready with every machine on the same start")
def us034_session(ctx):
    before = ctx.m08["before"]
    for key in ("ctrlc", "break", "runstop", "tool"):
        v = got(ctx, key)
        st = v["after"]
        assert (st.get("session") or {}).get("state") == "ready", f"after {key}: the session is {(st.get('session') or {}).get('state')}"
        assert starts(st) == before and set(states(st).values()) == {"ready"}, \
            f"after {key}: starts {starts(st)} states {states(st)}, before {before}"


@spcheck("US-063 RUN-06 DM-05 RES-01 eight checks of timeout 1 with a 14 s cleanup each stop between 68 s and 80 s after the first started: the check it was on invalid run-stopped, the rest not run, a valid result with stopped_reason run-ceiling, exit 2")
def us063_cli(ctx):
    v = got(ctx, "ceil_cli")
    r = v["result"]
    assert v["answer"].code == 2, f"{v['answer']}: {text_of(v['answer'])[-500:]}"
    n = len(r["checks"])
    assert 1 <= n <= 8, f"{n} executions"
    judge_stopped(ctx, r, "run-ceiling", CEIL[:n - 1], CEIL[n - 1], CEIL[n:], "the ceiling run")
    took = utc(r["checks"][-1]["reason"]["at"]) - first_started(r)
    ctx.log(f"M8 US-063: stopped {took:.1f} s after the first check started, at {CEIL[n - 1]}; "
            f"run.ended_at {r['run']['ended_at']}")
    assert CEILING_S <= took <= 80, f"the run stopped {took:.1f} s after its first check started, expected 68 to 80 s"
    assert "ceiling" in r["checks"][-1]["reason"]["message"], f"the reason: {r['checks'][-1]['reason']['message']!r}"


@spcheck("US-063 STP-02 after the ceiling run against a running session the session is ready and every machine keeps its start number")
def us063_session(ctx):
    before = ctx.m08["before"]
    for key in ("ceil_cli", "ceil_tool"):
        st = got(ctx, key)["after"]
        assert (st.get("session") or {}).get("state") == "ready", f"after {key}: the session is {(st.get('session') or {}).get('state')}"
        assert starts(st) == before, f"after {key}: starts {starts(st)}, before {before}"


@spcheck("US-063 RUN-06 through MCP run_start of the same eight checks is followed by run_status state stopped, and run_result gives the same summary")
def us063_tool(ctx):
    v = got(ctx, "ceil_tool")
    cli = got(ctx, "ceil_cli")
    assert v["status"]["state"] == "stopped", f"run_status state {v['status']['state']}"
    assert v["result"]["run"]["stopped_reason"] == "run-ceiling", f"stopped_reason {v['result']['run']['stopped_reason']}"
    assert v["run_result"]["summary"] == v["result"]["summary"] == cli["result"]["summary"], \
        f"run_result {v['run_result']['summary']}, its file {v['result']['summary']}, the command line run {cli['result']['summary']}"
