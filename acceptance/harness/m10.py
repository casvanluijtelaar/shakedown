r"""The checks of M10 The grappling-hook suite: its exit criteria 1, 2, 4, 5, 6 and 8 over arma-grappling-hook's
ported M00 checks, and the checks of the findings m10-1, m10-2, m10-3 and m10-5.

The ported suite is read from arma-grappling-hook's `acceptance/shakedown/` (default: the worktree of branch
`shakedown/m10`, or the `SHAKEDOWN_AGH_SUITE` variable) and copied into `m10port` in the work folder, so nothing is
written into that repository: its `shakedown.toml` and `tests/` as they are, and a `.mcp.json` as
docs/agent-tools.md gives it. The mod paths of that file are absolute, so the copy loads the same builds.

Per repeat (`--repeat`), with no session running before each part:

- the command line: `shakedown run --setup <s> --json` for dedicated-cba, sp-cba and sp-cba-ace, each with no
  session, so each starts one session, runs the setup's checks and stops it (the M10 demo); a thread lists the
  game processes every 2 s while it runs, so the check knows each process the run started;
- the agent: `shakedown mcp` started from the copy's `.mcp.json`, and per setup `session_start`, `run_start` with
  the setup, `run_status` until the run ends and `run_result`; in the same session the edit loop of NFR-PF-01 by
  the command line (a ported check edited, then run); in sp-cba one ported check changed to fail on purpose and
  diagnosed with `run_result`, `inspect` and `logs`; then `session_stop`, and the runner left to exit by itself;
- m10-1: the project `m10rel\repo\acceptance\shakedown` whose fixture is `..\..\build\@skd_fixture`, a copy of
  the installed `@skd_fixture`, run with `run --setup` on a dedicated server with two clients and in singleplayer.

The checks of m10-2, m10-3 and m10-5 run in the sessions of the generated project: m10-2 in `ded` and `cba`,
m10-3 in `cba`, m10-5 in `ded`. A part whose game start is lost to `Error 20 reading file` of a base-game file
starts again, at most three times in all.
"""

import json
import os
import re
import shutil
import threading
import time
import traceback
from pathlib import Path

from . import driver
from .core import batch, check, execution
from .m03 import command_line
from .m08 import game_folder, mission_copies, ports_held, steam_running
from .mcp import Mcp, structured

ACC = Path(__file__).resolve().parent.parent
SUITE = Path(os.environ.get("SHAKEDOWN_AGH_SUITE", r"C:\Users\Shadow\Documents\arma-grappling-hook"
                            r"\acceptance\shakedown"))
ORDER = ["dedicated-cba", "sp-cba", "sp-cba-ace"]
# plan/milestones/m10.md, exit criterion 7: the checks each setup runs.
EXPECTED = {
    "sp-cba": ["CLB-02-velocity", "CTL-01-defaults", "CTL-01-hold", "CTL-01-keys", "US-009-1", "US-009-1-freed",
               "US-009-2", "US-064-1", "US-064-2"],
    "sp-cba-ace": ["US-009-2", "US-064-1", "US-064-2"],
    "dedicated-cba": ["MP-01-drawn", "US-009-1-freed"],
}
MODS = {"sp-cba": ["cba", "agh"], "sp-cba-ace": ["cba", "ace", "agh"], "dedicated-cba": ["cba", "agh"]}
NAMES = {"sp-cba": ["p1"], "sp-cba-ace": ["p1"], "dedicated-cba": ["server", "p1", "p2"]}
ENV_KINDS = {"machine-lost", "session-broken", "reset-failed", "reload-failed", "run-stopped"}
EDIT = "CTL-01-keys"          # the check the edit loop edits: its body reads three keybinds on p1
DIAG = "CTL-01-hold-diagnose"  # CTL-01-hold changed to expect p1 to rise while climb down is held
DIAG_STEP = "climb down acts while held"
RUN_CEILING = 1500


# ================================================================ the project copy and the parts

def keep(ctx):
    if not hasattr(ctx, "m10_keep"):
        ctx.m10_keep = {}
    return ctx.m10_keep


def port_project(ctx):
    store = keep(ctx)
    if "port" in store:
        return store["port"]
    assert (SUITE / "shakedown.toml").is_file(), f"no ported suite at {SUITE}; give SHAKEDOWN_AGH_SUITE"
    project = ctx.work / "m10port"
    if project.exists():
        shutil.rmtree(project)
    shutil.copytree(SUITE / "tests", project / "tests")
    shutil.copy2(SUITE / "shakedown.toml", project / "shakedown.toml")
    (project / ".mcp.json").write_text(json.dumps(
        {"mcpServers": {"shakedown": {"command": "shakedown", "args": ["mcp"], "timeout": 70000}}}, indent=2),
        encoding="utf-8")
    store["port"] = (project, driver.Shakedown(ctx.sh.exe, project, ctx.log))
    return store["port"]


def machine_fault(root, since):
    sessions = Path(root) / ".shakedown" / "sessions"
    for rpt in sessions.glob("*/machines/*/*.rpt") if sessions.is_dir() else []:
        if rpt.stat().st_mtime >= since - 1:
            for line in rpt.read_text(encoding="utf-8", errors="replace").splitlines():
                if "Error 20 reading file" in line:
                    return line.strip()[:200]
    return None


def once(ctx, key, root, fn):
    """Runs a part once per repeat and keeps its value or its error; the checks of that repeat share it."""
    store = keep(ctx)
    k = (key, getattr(ctx, "repeat_n", 1))
    if k not in store:
        for attempt in range(1, 4):
            began = time.time()
            try:
                store[k] = fn(ctx)
                fault = None
                value = store[k]
                if isinstance(value, dict) and value.get("start_failed"):
                    fault = machine_fault(root, began)
                if not fault:
                    break
            except Exception as e:  # noqa: BLE001
                ctx.log(f"M10 part {k}: {traceback.format_exc()}")
                store[k] = e
                fault = machine_fault(root, began)
                if not fault:
                    break
            if attempt == 3:
                break
            ctx.log(f"M10 part {k}: a game process could not read a base-game file ({fault}); the machine, "
                    f"not the framework. Starting the part again, attempt {attempt + 1} of 3")
    value = store[k]
    if isinstance(value, Exception):
        raise AssertionError(f"the M10 part {key} failed: {value!r}")
    return value


def need_quiet(sh):
    assert steam_running(), "no steam process runs; a game start would be refused"
    busy = driver.game_pids()
    assert not busy, f"game processes run before the part starts: {sorted(busy)}"
    st = sh("status", "--json", ceiling=30).json()
    assert st.get("session") is None, f"status shows a session before the part starts: {(st['session'] or {}).get('id')}"


class Watch:
    """Lists the game processes every 2 s while a command runs, and the command line of each new one."""

    def __init__(self):
        self.before = driver.game_pids()
        self.seen, self.cmdlines = set(), {}
        self.done = threading.Event()
        self.thread = threading.Thread(target=self._loop, daemon=True)
        self.thread.start()

    def _loop(self):
        for _ in range(3600):
            for pid in driver.game_pids() - self.before - self.seen:
                self.seen.add(pid)
                try:
                    self.cmdlines[pid] = command_line(pid)
                except Exception:  # noqa: BLE001
                    self.cmdlines[pid] = ""
            if self.done.wait(2):
                return

    def stop(self):
        self.done.set()
        self.thread.join(timeout=60)
        return self.seen


def result_of(out):
    folder = Path(out["result_file"]).parent
    return json.loads((folder / "result.json").read_text(encoding="utf-8")), folder


def cli_run(ctx, sh, setup):
    """`run --setup <setup>` with no session; what it printed, its result, and every game process it left."""
    need_quiet(sh)
    runs_before = set(sh.run_folders())
    watch = Watch()
    try:
        answer = sh("run", "--setup", setup, "--json", ceiling=RUN_CEILING)
    finally:
        seen = watch.stop()
    returned = time.monotonic()
    alive_at_return = sorted(p for p in seen if driver.alive(p))
    left = {}
    for pid in alive_at_return:
        took = driver.wait_gone(pid, 30)
        left[pid] = took
        if took is None:
            ctx.log(f"M10: game process {pid} still runs 30 s after run --setup {setup} returned; stopping it by id")
            driver.kill(pid)
    st = sh("status", "--json", ceiling=30).json()
    new_runs = sorted(set(sh.run_folders()) - runs_before)
    rec = {"answer": answer, "seen": sorted(seen), "cmdlines": watch.cmdlines, "alive_at_return": alive_at_return,
           "left": left, "status_session": st.get("session"), "new_runs": new_runs,
           "ports": ports_held(), "returned": returned}
    try:
        rec["out"] = answer.json()
        rec["result"], rec["folder"] = result_of(rec["out"])
    except (AssertionError, KeyError, OSError, ValueError) as e:
        rec["error"] = f"{answer}: {e}; stderr {answer.stderr[-800:]}"
        rec["start_failed"] = True
    ctx.runner_pid = driver.runner_pid() or ctx.runner_pid
    return rec


def cli_part(setup):
    def part(ctx):
        project, sh = port_project(ctx)
        return cli_run(ctx, sh, setup)
    return part


def cli(ctx, setup):
    project, _ = port_project(ctx)
    rec = once(ctx, ("cli", setup), project, cli_part(setup))
    assert "result" in rec, f"run --setup {setup} left no result: {rec.get('error')}"
    return rec


# ---------------------------------------------------------------- the agent part

def wait_ready(mcp, session, ceiling=400):
    for _ in range(ceiling // 5):
        if session.get("state") in ("ready", "failed"):
            return session
        time.sleep(5)
        st = structured(mcp.call("status", {})[0])
        session = st.get("session") or {}
    return session


def wait_run(mcp, run_id, ceiling=RUN_CEILING):
    began = time.monotonic()
    last = None
    while time.monotonic() - began < ceiling:
        last = structured(mcp.call("run_status", {"run_id": run_id, "wait_s": 45}, ceiling=70)[0])
        if last.get("state") in ("finished", "stopped"):
            return last
    raise AssertionError(f"the run {run_id} did not end within {ceiling} s: {str(last)[:300]}")


def edit_loop(ctx, project, sh, setup):
    """NFR-PF-01: edit a ported check and run it at once in the ready session; the time from the edit to its outcome."""
    path = project / "tests" / f"{EDIT}.sqf"
    original = path.read_text(encoding="utf-8")
    name = f"edited {setup} repeat {getattr(ctx, 'repeat_n', 1)}"
    text = original.replace("under Grappling hooks\n", f"under Grappling hooks ({name})\n", 1)
    if setup != "sp-cba":
        text = text.replace("// @setups sp-cba\n", f"// @setups sp-cba {setup}\n", 1)
    assert text != original and name in text, "the edit loop could not edit the check's @check line"
    try:
        path.write_text(text, encoding="utf-8")
        edited = time.monotonic()
        answer = sh("run", EDIT, "--json", ceiling=120)
        took = time.monotonic() - edited
    finally:
        path.write_text(original, encoding="utf-8")
    out = answer.json()
    result, _ = result_of(out)
    return {"answer": answer, "took": took, "result": result, "name": name}


def diagnose(ctx, project, mcp):
    """One ported check changed to fail, run through run_start, and looked into with run_result, inspect and logs."""
    original = (project / "tests" / "CTL-01-hold.sqf").read_text(encoding="utf-8")
    wrong = original.replace(
        '"<", _z1 - 0.5, "p1 climbs down 0.5 m while the key is held"',
        '">", _z1 + 0.5, "p1 climbs up 0.5 m while climb down is held (changed on purpose)"', 1)
    assert wrong != original, "the diagnosis could not change CTL-01-hold's climb down assertion"
    path = project / "tests" / f"{DIAG}.sqf"
    path.write_text(wrong, encoding="utf-8")
    try:
        started = structured(mcp.call("run_start", {"ids": [DIAG]})[0])
        status = wait_run(mcp, started["run_id"], 300)
        res = structured(mcp.call("run_result", {"run_id": started["run_id"]})[0])
        insp = structured(mcp.call("inspect", {"machine": "p1"})[0])
        logs = structured(mcp.call("logs", {"machine": "p1", "run_id": started["run_id"], "check_id": DIAG,
                                            "last": 50})[0])
    finally:
        path.unlink()
    result = json.loads(Path(res["result_file"]).read_text(encoding="utf-8")) if "result_file" in res else None
    return {"status": status, "result_tool": res, "inspect": insp, "logs": logs, "result": result}


def agent_part(ctx):
    project, sh = port_project(ctx)
    need_quiet(sh)
    rec = {"setups": {}}
    seen = set()
    watch = Watch()
    try:
        with Mcp(project, ctx.install, ctx.log) as mcp:
            for setup in ORDER:
                one = rec["setups"][setup] = {}
                session = structured(mcp.call("session_start", {"setup": setup}, ceiling=70)[0])
                session = wait_ready(mcp, session)
                one["session"] = session
                if session.get("state") != "ready":
                    one["error"] = f"session_start {setup} ended {session.get('state')}: {str(session.get('reason'))[:600]}"
                    rec["start_failed"] = True
                    structured(mcp.call("session_stop", {}, ceiling=70)[0])
                    continue
                started = structured(mcp.call("run_start", {"setup": setup})[0])
                one["run_start"] = started
                one["run_status"] = wait_run(mcp, started["run_id"])
                one["run_result"] = res = structured(mcp.call("run_result", {"run_id": started["run_id"]})[0])
                one["result"] = json.loads(Path(res["result_file"]).read_text(encoding="utf-8"))
                one["edit"] = edit_loop(ctx, project, sh, setup)
                if setup == "sp-cba":
                    one["diagnose"] = diagnose(ctx, project, mcp)
                one["stop"] = structured(mcp.call("session_stop", {}, ceiling=70)[0])
                one["alive_after_stop"] = sorted(p for p in watch.seen if driver.alive(p))
                seen |= watch.seen
    finally:
        seen |= watch.stop()
        for pid in seen:
            if driver.alive(pid) and driver.wait_gone(pid, 30) is None:
                ctx.log(f"M10: game process {pid} still runs after the agent part; stopping it by id")
                driver.kill(pid)
        try:
            if (sh("status", "--json", ceiling=30).json().get("session") or {}).get("state") not in (None, "stopped"):
                sh("session", "stop", "--json", ceiling=180)
        except Exception as e:  # noqa: BLE001
            ctx.log(f"M10: status or session stop after the agent part: {e!r}")
    rp = driver.runner_pid()
    rec["runner_pid"] = rp
    rec["runner_gone_s"] = driver.wait_gone(rp, 75) if rp else 0.0
    if rp and rec["runner_gone_s"] is None:
        ctx.log(f"M10: the runner {rp} still runs 75 s after the agent's last session stopped; stopping it by id")
        driver.kill(rp)
    return rec


def agent(ctx):
    project, _ = port_project(ctx)
    rec = once(ctx, "agent", project, agent_part)
    return rec


def agent_setup(ctx, setup):
    one = agent(ctx)["setups"].get(setup) or {}
    assert "result" in one, f"the agent's {setup} part left no result: {one.get('error')}"
    return one


# ---------------------------------------------------------------- m10-1

def rel_project(ctx):
    store = keep(ctx)
    if "rel" in store:
        return store["rel"]
    game = game_folder(ctx.project)
    root = ctx.work / "m10rel" / "repo"
    if root.exists():
        shutil.rmtree(root)
    project = root / "acceptance" / "shakedown"
    shutil.copytree(ACC / "m10", project / "tests")
    shutil.copytree(ctx.install / "fixtures" / "@skd_fixture", root / "build" / "@skd_fixture")
    (project / "shakedown.toml").write_text(
        f"game = '{game}'\n\n[mods.fixture]\npath = '..\\..\\build\\@skd_fixture'\npatch = \"skdfx_main\"\n\n"
        "[setups.ded]\nmode = \"dedicated\"\nclients = 2\nmods = [\"fixture\"]\n\n"
        "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n", encoding="utf-8")
    store["rel"] = (project, driver.Shakedown(ctx.sh.exe, project, ctx.log))
    return store["rel"]


def rel(ctx, setup):
    project, sh = rel_project(ctx)
    rec = once(ctx, ("rel", setup), project, lambda c: cli_run(c, sh, setup))
    assert "result" in rec, f"run --setup {setup} of the m10-1 project left no result: {rec.get('error')}"
    return rec


# ================================================================ judging

def ids_of(result):
    return sorted(c["check_id"] for c in result["checks"])


def outcomes(result):
    return {c["check_id"]: c["outcome"] for c in result["checks"]}


def bad_checks(result):
    return [f"{c['check_id']}: {c['outcome']} {(c.get('reason') or {}).get('kind')}: "
            f"{str((c.get('reason') or {}).get('message'))[:400]} (step {c['evidence'].get('failed_step')})"
            for c in result["checks"] if c["outcome"] != "pass"]


def headers(project):
    out = {}
    for f in sorted((project / "tests").glob("*.sqf")):
        h = {}
        for line in f.read_text(encoding="utf-8").splitlines():
            m = re.match(r"\s*//\s*@(\w+)\s+(.*)", line)
            if not m:
                if line.strip():
                    break
                continue
            h[m.group(1)] = m.group(2).strip()
        out[f.stem] = (h, f.read_text(encoding="utf-8"))
    return out


def judge_ec1(result, setup):
    sid = result["session"].get("id")
    assert sid and re.fullmatch(r"s-\d{8}-\d{6}.*", sid), f"the result names no session id: {sid!r}"
    names = [m["name"] for m in result["session"]["machines"]]
    assert sorted(names) == sorted(NAMES[setup]), f"the machines are {names}, expected {NAMES[setup]}"
    starts = {m["name"]: m["start"] for m in result["session"]["machines"]}
    assert all(s == 1 for s in starts.values()), \
        f"a machine was started again during the run, though no ported check says @reset mission: {starts}"
    return sid


def judge_ec4(ctx, rec_result, setup):
    result = rec_result
    errors = ctx.validate(result)
    assert not errors, f"the result file is not valid against result.schema.json: {errors[:5]}"
    assert ids_of(result) == EXPECTED[setup], f"the run executed {ids_of(result)}, expected {EXPECTED[setup]}"
    for c in result["checks"]:
        assert c["outcome"] in ("pass", "fail", "invalid"), f"{c['check_id']} ended {c['outcome']}"
        if c["outcome"] != "pass":
            r = c.get("reason") or {}
            assert r.get("kind") and r.get("message"), f"{c['check_id']} ended {c['outcome']} without a reason: {r}"
        for e in c["errors"]:
            assert e["check_id"] in (None, c["check_id"]), \
                f"{c['check_id']} lists an error stamped with another check: {e['check_id']}: {e['text'][:200]}"
            if not e["expected"] and c["outcome"] == "fail" and (c.get("reason") or {}).get("kind") == "script-error":
                assert c["evidence"].get("failed_step"), f"{c['check_id']} failed script-error with no failed step"
    s = result["summary"]
    assert s["total"] == s["pass"] + s["fail"] + s["invalid"] + s["not_run"], f"the summary does not add up: {s}"
    for m in result["session"]["machines"]:
        got = {x["mod"]: x["version"] for x in m["mods"]}
        missing = [n for n in MODS[setup] if not got.get(n)]
        assert not missing, f"{m['name']} records no loaded version of {missing} (INT-STM-03): {m['mods']}"
    bad = bad_checks(result)
    assert not bad, (f"a ported check did not pass where the PowerShell suite passed on the same build "
                     f"(exit criterion 7): {bad}")


def judge_ec8(result, setup):
    """NFR-RL-01 and NFR-RL-02 over one ported run, and SPK-14's 5 s margin to each passing check's ceiling."""
    timeouts = {}
    for c in result["checks"]:
        kind = (c.get("reason") or {}).get("kind")
        assert not (c["outcome"] == "fail" and kind in ENV_KINDS), \
            f"{c['check_id']} ended fail with the environment cause {kind} (NFR-RL-01)"
        assert kind not in ("reset-failed", "session-broken", "machine-lost"), \
            f"{c['check_id']} did not see the baseline on every machine: {kind}: {(c.get('reason') or {}).get('message')}"
    for m in result["session"]["machines"]:
        assert m["end_state"] == "ready", f"{m['name']} ended the run {m['end_state']} (NFR-RL-02)"
    return timeouts


def margin(ctx, project, result):
    h = headers(project)
    close = []
    for c in result["checks"]:
        if c["outcome"] == "pass":
            timeout = int(h[c["check_id"]][0].get("timeout", 120))
            if c["duration_s"] > timeout - 5:
                close.append(f"{c['check_id']} took {c['duration_s']:.1f} s of its {timeout} s")
    return close


# ================================================================ exit criteria over the command-line runs

def portcheck(name):
    return check(name, "start", repeat=True)


@portcheck("M10 EC6 CR-05 RUN-04 run --setup dedicated-cba of the ported suite starts one session, runs it and stops it: when it returns no game process it started runs, no game port is held, no mission copy is left and status shows no session")
def ec6_dedicated(ctx):
    judge_ec6(ctx, "dedicated-cba")


@portcheck("M10 EC6 CR-05 RUN-04 run --setup sp-cba of the ported suite leaves no game process it started running when it returns, and status shows no session")
def ec6_sp(ctx):
    judge_ec6(ctx, "sp-cba")


@portcheck("M10 EC6 CR-05 RUN-04 run --setup sp-cba-ace of the ported suite leaves no game process it started running when it returns, and status shows no session")
def ec6_ace(ctx):
    judge_ec6(ctx, "sp-cba-ace")


def judge_ec6(ctx, setup):
    rec = cli(ctx, setup)
    expected = 3 if setup == "dedicated-cba" else 1
    assert len(rec["seen"]) == expected, f"the run started {len(rec['seen'])} game processes, expected {expected}: {rec['seen']}"
    assert not rec["alive_at_return"], \
        f"game processes the run started still ran when it returned: {rec['alive_at_return']}; gone after {rec['left']}"
    assert rec["status_session"] is None, f"status shows a session after the run: {str(rec['status_session'])[:300]}"
    assert not rec["ports"], f"game ports held after the run: {rec['ports']}"
    game = game_folder(port_project(ctx)[0])
    left = mission_copies(game, rec["result"]["session"]["id"])
    assert not left, f"mission copies of the session left in MPMissions: {left}"


@portcheck("M10 EC1 CR-01 CR-02 each ported run of dedicated-cba, sp-cba and sp-cba-ace shows one session id with server, p1 and p2 or p1 alone, every machine on start number 1 throughout, and leaves one run folder")
def ec1(ctx):
    for setup in ORDER:
        rec = cli(ctx, setup)
        judge_ec1(rec["result"], setup)
        assert len(rec["new_runs"]) == 1, f"run --setup {setup} left {len(rec['new_runs'])} run folders: {rec['new_runs']}"
        assert rec["answer"].code in (0, 1, 2), f"{rec['answer']}"


@portcheck("M10 EC4 CR-23 CR-31 CR-51 INT-STM-03 run --setup dedicated-cba of the ported suite writes one valid result file in which US-009-1-freed and MP-01-drawn each end pass, fail or invalid with a reason, every machine records the CBA and arma-grappling-hook versions it loaded, and both pass as on the PowerShell suite")
def ec4_dedicated(ctx):
    rec = cli(ctx, "dedicated-cba")
    judge_ec4(ctx, rec["result"], "dedicated-cba")
    assert rec["answer"].code == 0, f"{rec['answer']}, expected exit 0 with every check passed"


@portcheck("M10 EC4 CR-23 CR-31 CR-51 INT-STM-03 run --setup sp-cba of the ported suite writes one valid result file in which the nine checks each end with a reason when not passed, p1 records the CBA and arma-grappling-hook versions, and all nine pass as on the PowerShell suite")
def ec4_sp(ctx):
    rec = cli(ctx, "sp-cba")
    judge_ec4(ctx, rec["result"], "sp-cba")
    assert rec["answer"].code == 0, f"{rec['answer']}, expected exit 0 with every check passed"


@portcheck("M10 EC4 CR-23 CR-31 CR-51 INT-STM-03 INT-ACE-02 run --setup sp-cba-ace of the ported suite writes one valid result file in which US-009-2, US-064-1 and US-064-2 pass with ACE loaded, p1 records the CBA, ACE and arma-grappling-hook versions, and no reset fails")
def ec4_ace(ctx):
    rec = cli(ctx, "sp-cba-ace")
    judge_ec4(ctx, rec["result"], "sp-cba-ace")
    assert rec["answer"].code == 0, f"{rec['answer']}, expected exit 0 with every check passed"


@portcheck("M10 EC8 NFR-RL-01 NFR-RL-02 SPK-14 over the ported command-line runs no check ends fail for an environment cause, every check sees the baseline with no machine lost, every machine ends ready, and no passing check comes within 5 s of its ceiling")
def ec8_cli(ctx):
    project, _ = port_project(ctx)
    close = []
    for setup in ORDER:
        rec = cli(ctx, setup)
        judge_ec8(rec["result"], setup)
        close += margin(ctx, project, rec["result"])
    assert not close, f"passing checks within 5 s of their ceiling (SPK-14): {close}"


@check("M10 EC2 CR-10 CR-11 CR-12 CR-20 CR-22 CR-50 the ported project is SQF checks and a shakedown.toml only, every check's header names its story or requirement, list reads all ten with no error, and the checks build a scene, act through the mod's keybinds and functions on p1, wait with ceilings and assert on every client", "offline")
def ec2(ctx):
    project, sh = port_project(ctx)
    files = sorted(p.relative_to(SUITE).as_posix() for p in SUITE.rglob("*")
                   if p.is_file() and ".shakedown" not in p.parts)
    other = [f for f in files if not (f == "shakedown.toml" or (f.startswith("tests/") and f.endswith(".sqf")))]
    assert not other, f"the ported project holds files other than shakedown.toml and SQF checks: {other}"
    answer = sh("list", "--json", ceiling=60)
    listed = answer.json()["checks"]
    assert answer.code == 0, f"{answer}"
    errs = [(c["id"], c["error"]) for c in listed if c["error"]]
    assert not errs, f"list names header mistakes: {errs}"
    assert sorted(c["id"] for c in listed) == sorted(set(sum(EXPECTED.values(), []))), \
        f"list gives {sorted(c['id'] for c in listed)}"
    for setup, ids in EXPECTED.items():
        got = sorted(c["id"] for c in listed if c["setups"] is None or setup in c["setups"])
        assert got == ids, f"the checks naming {setup} are {got}, expected {ids}"
    h = headers(project)
    for cid, (head, text) in h.items():
        assert re.match(r"[A-Z][A-Z0-9]*(-[A-Z0-9]+)*-?\d* ", head.get("check", "")) and re.search(r"\d", head["check"].split()[0]), \
            f"{cid}'s @check names no story or requirement ID: {head.get('check')!r}"
        assert not re.search(r"^\s*#\s*(include|define|ifdef|ifndef)", text, re.M), f"{cid} uses the preprocessor"
        assert head.get("reset", "in-place") == "in-place", f"{cid} says @reset {head.get('reset')}"
    rope = [cid for cid in h if cid not in ("CTL-01-keys", "CTL-01-defaults")]
    for cid in rope:
        text = h[cid][1]
        for need in ("skd_fnc_create", "skd_fnc_place", "skd_fnc_step"):
            assert need in text, f"{cid} builds no scene with {need}"
        assert re.search(r"skd_fnc_(waitUntil|expectOn|expectEach)", text), f"{cid} has no wait with a ceiling"
        assert re.search(r"agh_fnc_\w+|skd_fnc_pressKey", text), f"{cid} does not act through the mod"
    assert "skd_fnc_pressKey" in h["CTL-01-hold"][1], "CTL-01-hold presses no keybind with skd_fnc_pressKey"
    for cid in EXPECTED["dedicated-cba"]:
        text = h[cid][1]
        assert "skd_fnc_expectEach" in text and re.search(r'"(clients|all)"', text),             f"{cid} asserts on no machine but the lead"


# ================================================================ the agent (exit criterion 5)

@portcheck("M10 EC5 CR-27 CR-53 AGT-01 the ported suite run by an agent through shakedown mcp, started from the project's .mcp.json, with session_start, run_start, run_status and run_result per setup, reaches the outcomes the command line reached in the same repeat, check by check")
def ec5_same(ctx):
    for setup in ORDER:
        one = agent_setup(ctx, setup)
        got = outcomes(one["result"])
        res = one["run_result"]
        assert res["state"] == "finished", f"the agent's {setup} run ended {res['state']}"
        assert {o["check_id"]: o["outcome"] for o in res["outcomes"]} == got, \
            f"run_result's outcomes differ from its result file: {res['outcomes']} vs {got}"
        cli_out = outcomes(cli(ctx, setup)["result"])
        assert got == cli_out, f"{setup}: the agent's outcomes {got} differ from the command line's {cli_out}"
        errors = ctx.validate(one["result"])
        assert not errors, f"the agent's {setup} result file is not valid: {errors[:5]}"


@portcheck("M10 EC5 CR-53 INS-01 the agent diagnoses a ported check changed to fail: run_result names the step and the assertion, and inspect of p1 shows the player below the tower's top it stood on, the opposite of what the changed assertion expected")
def ec5_diagnose(ctx):
    d = agent_setup(ctx, "sp-cba").get("diagnose")
    assert d, "the agent's diagnosis did not run"
    res = d["result_tool"]
    lines = [o for o in res["outcomes"] if o["check_id"] == DIAG]
    assert len(lines) == 1 and lines[0]["outcome"] == "fail", f"run_result gives {res['outcomes']}"
    o = lines[0]
    assert (o.get("reason") or {}).get("kind") == "assertion", f"the changed check ended {o.get('reason')}"
    assert DIAG_STEP in (o.get("failed_step") or []), f"run_result names the failed step {o.get('failed_step')}"
    assert "changed on purpose" in o["reason"]["message"], f"the reason does not quote the assertion: {o['reason']}"
    ex = execution(d["result"], DIAG)
    lip = [a for a in ex["evidence"]["attached"] if a.get("label") == "lip"]
    assert lip, f"the changed check attached no lip evidence: {ex['evidence']['attached'][:3]}"
    top_z = lip[0]["value"][0][2]
    player = d["inspect"].get("player")
    assert player and player.get("alive"), f"inspect of p1 gives no living player: {player}"
    assert player["position"][2] < top_z - 0.5, \
        f"inspect gives p1 at z {player['position'][2]:.2f}, not below the top at {top_z:.2f}"
    assert d["inspect"]["machine"]["state"] == "ready", f"inspect gives p1 {d['inspect']['machine']['state']}"
    assert d["logs"]["lines"], "logs gives no line of p1 for the changed check"


@portcheck("M10 EC5 EC6 CR-05 after the agent's session_stop of each setup no game process of that session runs, and the runner exits by itself within 75 s of the last stop")
def ec5_nothing_left(ctx):
    rec = agent(ctx)
    for setup in ORDER:
        one = rec["setups"].get(setup) or {}
        assert one.get("stop", {}).get("state") == "stopped", f"session_stop of {setup} gave {str(one.get('stop'))[:200]}"
        assert not one.get("alive_after_stop"), f"game processes ran after session_stop of {setup}: {one['alive_after_stop']}"
    assert rec["runner_gone_s"] is not None, f"the runner {rec['runner_pid']} still ran 75 s after the last session stopped"


@portcheck("M10 EC1 CR-01 CR-02 the agent's session of each setup keeps one session id across the agent's run, the edit loop's run and the diagnosis, with every machine on start number 1")
def ec1_agent(ctx):
    for setup in ORDER:
        one = agent_setup(ctx, setup)
        sid = one["session"]["id"]
        results = [one["result"], one["edit"]["result"]] + ([one["diagnose"]["result"]] if one.get("diagnose") else [])
        for r in results:
            assert r["session"]["id"] == sid, f"{setup}: a run shows session {r['session']['id']}, the agent started {sid}"
            judge_ec1(r, setup)


@portcheck("M10 EC8 NFR-PF-01 with a session ready, a ported check edited and run at once reaches its outcome in under 30 s, its body under 10 s, in singleplayer and on the dedicated session with two clients; the result names the edited check")
def ec8_edit(ctx):
    for setup in ORDER:
        e = agent_setup(ctx, setup)["edit"]
        ex = execution(e["result"], EDIT)
        assert e["name"] in ex["name"], f"{setup}: the run read {ex['name']!r}, not the edited check"
        assert ex["outcome"] == "pass", f"{setup}: the edited check ended {ex['outcome']}: {ex.get('reason')}"
        body = sum(s["duration_s"] or 0 for s in ex["steps"] if len(s["path"]) == 1)
        assert body < 10, f"{setup}: the edited check's body took {body:.1f} s, not under 10 s"
        assert e["took"] < 30, f"{setup}: from the edit to the outcome took {e['took']:.1f} s, not under 30 s"
        ctx.log(f"M10 NFR-PF-01 {setup}: edit to outcome {e['took']:.1f} s, check {ex['duration_s']:.1f} s, body {body:.2f} s")


@portcheck("M10 EC8 NFR-RL-01 NFR-RL-02 SPK-14 over the agent's ported runs no check ends fail for an environment cause, every machine ends ready, and no passing check comes within 5 s of its ceiling")
def ec8_agent(ctx):
    project, _ = port_project(ctx)
    close = []
    for setup in ORDER:
        r = agent_setup(ctx, setup)["result"]
        judge_ec8(r, setup)
        close += margin(ctx, project, r)
        assert not bad_checks(r), f"{setup}: {bad_checks(r)}"
    assert not close, f"passing checks within 5 s of their ceiling (SPK-14): {close}"


@check("M10 EC8 SPK-14 NFR-PF-06 the spike is recorded with its pass condition met and NFR-PF-06 names it with the defaults 10 s, 120 s, 10 s, 180 s and 15 s, which the docs give", "offline")
def ec8_spk14(ctx):
    nfr = (ctx.repo / "spec" / "05-non-functional-requirements.md").read_text(encoding="utf-8")
    line = next(l for l in nfr.splitlines() if l.startswith("- **NFR-PF-06**"))
    for want in ("wait ceiling is 10 s", "check timeout 120 s", "grace 10 s", "start ceiling 180 s", "reset ceiling 15 s",
                 "SPK-14", "29-spike-default-ceilings.md"):
        assert want in line, f"NFR-PF-06 does not say {want!r}: {line}"
    spike = (ctx.repo / "research" / "29-spike-default-ceilings.md").read_text(encoding="utf-8")
    assert "It passed" in spike, "research 29 does not record that SPK-14 passed"
    docs = "\n".join(p.read_text(encoding="utf-8") for p in (ctx.repo / "docs").glob("*.md"))
    for want in ("default 10", "120 s", "15 s", "180"):
        assert want in docs, f"the docs do not give {want!r}"


# ================================================================ m10-1

@portcheck("US-001 US-006 CFG-01 m10-1 a project in acceptance\\shakedown whose mod path is ..\\..\\build\\@skd_fixture starts a dedicated session with two clients through run --setup, the mod loads on every machine, its version is recorded, and no game command line holds ..")
def m10_1_ded(ctx):
    judge_rel(ctx, "ded", ["server", "p1", "p2"])


@portcheck("US-001 US-006 CFG-01 m10-1 the same project starts a singleplayer session through run --setup, the mod loads on p1 and no game command line holds ..")
def m10_1_sp(ctx):
    judge_rel(ctx, "sp", ["p1"])


def judge_rel(ctx, setup, names):
    rec = rel(ctx, setup)
    assert rec["answer"].code == 0, f"{rec['answer']}: {bad_checks(rec['result'])}"
    r = rec["result"]
    assert sorted(m["name"] for m in r["session"]["machines"]) == sorted(names)
    for m in r["session"]["machines"]:
        v = {x["mod"]: x["version"] for x in m["mods"]}.get("fixture")
        assert v, f"{m['name']} records no version of the fixture: {m['mods']}"
    lines = [c for c in rec["cmdlines"].values() if c]
    assert len(lines) == len(names), f"read {len(lines)} game command lines of {len(names)}: {rec['cmdlines']}"
    assert any(r"build\@skd_fixture" in c for c in lines), f"no game command line names the fixture build: {lines}"
    for c in lines:
        assert "\\..\\" not in c and "\\.\\" not in c, f"a game command line holds . or ..: {c[:600]}"
    assert not rec["alive_at_return"], f"game processes ran when run --setup returned: {rec['alive_at_return']}"


# ================================================================ m10-2, m10-3, m10-5 in the generated project's sessions

M10_2 = ["m10/US-008-m10-2-client-cleanup-then-fail", "m10/US-008-m10-2-after"]
M10_2C = ["m10/US-008-m10-2-control-lead-cleanup", "m10/US-008-m10-2-control-after", "m10/US-008-m10-2-after"]
M10_3 = "m10/US-021-m10-3-read-what-the-key-returned"
M10_5 = "m10/US-013-m10-5-wait-then-on"


@batch("ded")
def ded_batch(ctx, checks):
    needed = {i for c in checks for i in c.needs}
    store = ctx.m10_ded = {}
    if set(M10_2) & needed:
        store["m10-2"] = ctx.run(M10_2)
    if M10_2C[0] in needed:
        store["m10-2c"] = ctx.run(M10_2C)
    if M10_5 in needed:
        store["m10-5"] = ctx.run([M10_5], ceiling=400)


@batch("cba")
def cba_batch(ctx, checks):
    needed = {i for c in checks for i in c.needs}
    store = ctx.m10_cba = {}
    if set(M10_2) & needed:
        store["m10-2"] = ctx.run(M10_2)
    if M10_3 in needed:
        store["m10-3"] = ctx.run([M10_3])


def got(ctx, setup, key):
    store = getattr(ctx, f"m10_{setup}", {})
    assert key in store, f"the {key} run in {setup} did not happen this repeat; see harness.log"
    return store[key]


def judge_m10_2(ctx, setup):
    answer, out, result = got(ctx, setup, "m10-2")
    first, after = execution(result, M10_2[0]), execution(result, M10_2[1])
    assert first["outcome"] == "fail" and first["reason"]["kind"] == "explicit", \
        f"the first check ended {first['outcome']} {first.get('reason')}, expected fail explicit"
    assert after["outcome"] == "pass", \
        f"the check after it ended {after['outcome']}: {(after.get('reason') or {}).get('message')}"
    assert answer.code == 1, f"{answer}, expected exit 1"


@check("US-008 RST-01 m10-2 on a dedicated session a cleanup that names p1 runs on p1: after a check that made p1 unhurtable, set a variable there and failed, the next check finds p1 damageable and the variable gone",
       "session", needs=M10_2, setup="ded")
def m10_2_ded(ctx):
    judge_m10_2(ctx, "ded")


@check("US-008 RST-01 m10-2 the control on a dedicated session: a cleanup naming no machine runs on the server, so p1 keeps the allowDamage and variable the check set through the reset, and a later check's cleanup naming p1 puts them back",
       "session", needs=M10_2C, setup="ded")
def m10_2_control(ctx):
    answer, out, result = got(ctx, "ded", "m10-2c")
    bad = [f"{c['check_id']}: {c['outcome']} {(c.get('reason') or {}).get('message')}" for c in result["checks"] if c["outcome"] != "pass"]
    assert not bad, f"the control did not hold: {bad}"


@check("US-008 RST-01 m10-2 in singleplayer a cleanup that names p1 undoes p1's allowDamage and variable after a check that failed",
       "session", needs=M10_2, setup="cba")
def m10_2_sp(ctx):
    judge_m10_2(ctx, "cba")


@check("US-013 DM-14 m10-5 on a dedicated session each of 20 crates waited for on every client right after skd_fnc_create is found there in place within the 10 s ceiling, and skd_fnc_on then reads it there, not objNull",
       "session", needs=[M10_5], setup="ded")
def m10_5(ctx):
    answer, out, result = got(ctx, "ded", "m10-5")
    ex = execution(result, M10_5)
    assert ex["outcome"] == "pass", f"ended {ex['outcome']}: {(ex.get('reason') or {}).get('message')} (step {ex['evidence'].get('failed_step')})"
    waits = [a["value"] for a in ex["evidence"]["attached"] if a.get("label", "").startswith("seconds each wait")]
    assert waits and len(waits[0]) == 40, f"the check attached {waits}"
    ctx.log(f"M10 m10-5: the longest wait for a just-created crate on a client took {max(waits[0]):.2f} s")


@check("US-021 ACT-02 m10-3 the way docs/check-api.md gives to read what a keybind's code returned runs the fixture's signal down code once and reads false from arma-grappling-hook's let go off a rope",
       "session", needs=[M10_3], setup="cba")
def m10_3(ctx):
    answer, out, result = got(ctx, "cba", "m10-3")
    ex = execution(result, M10_3)
    assert ex["outcome"] == "pass", f"ended {ex['outcome']}: {(ex.get('reason') or {}).get('message')} (step {ex['evidence'].get('failed_step')})"
