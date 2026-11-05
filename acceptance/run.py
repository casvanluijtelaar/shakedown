"""Runs the acceptance suite. See acceptance/README.md for how to use it."""

import argparse
import json
import os
import shutil
import sys
import tempfile
import time
import traceback
from datetime import datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
sys.path.insert(0, str(HERE))

from harness import driver, m00  # noqa: E402
from harness.core import CHECKS, Context  # noqa: E402

DEFAULT_GAME = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3"


def find_exe(install):
    if install:
        return Path(install) / "shakedown.exe"
    on_path = shutil.which("shakedown")
    return Path(on_path) if on_path else REPO / "target" / "shakedown" / "shakedown.exe"


def make_project(work, game):
    project = work / "project"
    shutil.copytree(HERE / "checks", project / "checks")
    (project / "shakedown.toml").write_text(
        f"game = '{game}'\n\n[setups.sp]\nmode = \"singleplayer\"\nmods = []\n", encoding="utf-8")
    return project


def set_syntax(project, fixed):
    path = project / "checks" / (m00.SYNTAX + ".sqf")
    lines = path.read_text(encoding="utf-8").split("\n")
    lines[m00.SYNTAX_LINE - 1] = m00.SYNTAX_FIXED if fixed else m00.SYNTAX_BROKEN
    path.write_text("\n".join(lines), encoding="utf-8")


def main():
    ap = argparse.ArgumentParser(description="Shakedown acceptance suite")
    ap.add_argument("--check", action="append", default=[],
                    help="run only checks whose name contains this text; repeat for several")
    ap.add_argument("--repeat", type=int, default=1, help="run the in-session checks this many times in one session")
    ap.add_argument("--install", default=os.environ.get("SHAKEDOWN_INSTALL"), help="the install folder holding shakedown.exe")
    ap.add_argument("--game", default=os.environ.get("SHAKEDOWN_GAME", DEFAULT_GAME), help="the Arma 3 folder")
    ap.add_argument("--work", help="the folder for the generated project and the report (default: a new temp folder)")
    ap.add_argument("--list", action="store_true", help="print the check names and exit")
    args = ap.parse_args()

    selected = [c for c in CHECKS if not args.check or any(f in c.name for f in args.check)]
    if args.list:
        for c in selected:
            print(f"[{c.phase}] {c.name}")
        return 0
    if not selected:
        print("No check name contains any of the --check values; --list prints the names.", file=sys.stderr)
        return 3

    exe = find_exe(args.install)
    if not exe.is_file():
        print(f"No shakedown.exe at {exe}. Run `python tools/install.py` or give --install.", file=sys.stderr)
        return 3

    work = Path(args.work) if args.work else Path(tempfile.mkdtemp(prefix="skd-acceptance-"))
    work.mkdir(parents=True, exist_ok=True)
    logfile = (work / "harness.log").open("a", encoding="utf-8")

    def log(text):
        line = f"{datetime.now():%H:%M:%S} {text}"
        print(line, flush=True)
        logfile.write(line + "\n")
        logfile.flush()

    project = make_project(work, args.game)
    ctx = Context(driver.Shakedown(exe, project, log), project, REPO, log)
    results = []  # (repeat, name, ok, message)

    def run_check(c, repeat):
        try:
            c.fn(ctx)
            results.append((repeat, c.name, True, ""))
            log(f"PASS  {c.name}")
        except AssertionError as e:
            results.append((repeat, c.name, False, str(e)))
            log(f"FAIL  {c.name}\n      {e}")
        except Exception as e:  # a harness error is reported as a failure of the check
            results.append((repeat, c.name, False, f"harness error: {e!r}"))
            log(f"FAIL  {c.name}\n      harness error: {traceback.format_exc()}")

    log(f"shakedown {exe}, project {project}")

    for c in [c for c in selected if c.phase == "offline"]:
        run_check(c, 1)

    in_session = [c for c in selected if c.phase in ("session", "end")]
    if in_session:
        busy = driver.pids("arma3_x64.exe") | driver.pids("arma3server_x64.exe")
        if busy or driver.runner_pid() and driver.alive(driver.runner_pid()):
            log(f"Another game ({sorted(busy)}) or runner ({driver.runner_pid()}) is running; stop it first. "
                "No in-session check ran.")
            return 4
        ctx.game_pids_before = driver.pids("arma3_x64.exe")
        try:
            run_session(ctx, selected, args.repeat, run_check, log)
        finally:
            cleanup(ctx, log)
        for c in [c for c in selected if c.phase == "end"]:
            run_check(c, 1)
        rp = ctx.runner_pid
        if rp and driver.alive(rp):
            log(f"waiting up to 75 s for the runner {rp} the suite started to exit by itself")
            if driver.wait_gone(rp, 75) is None:
                log(f"the runner {rp} still runs; stopping it by id")
                driver.kill(rp)

    failed = [r for r in results if not r[2]]
    (work / "report.json").write_text(json.dumps(
        [{"repeat": r, "check": n, "pass": ok, "message": m} for r, n, ok, m in results], indent=1), encoding="utf-8")
    log(f"{len(results) - len(failed)} passed, {len(failed)} failed. Report: {work / 'report.json'}")
    return 1 if failed else 0


def run_session(ctx, selected, repeat, run_check, log):
    session_checks = [c for c in selected if c.phase == "session"]
    answer = ctx.sh("session", "start", "--setup", "sp", "--json", ceiling=400)
    ctx.runner_pid = driver.runner_pid()
    ctx.session = answer.json() if answer.stdout.strip() else None
    if ctx.session:
        ctx.game_pids = {m["pid"] for m in ctx.session["machines"]}
    if answer.code != 0 or not ctx.session or ctx.session.get("state") != "ready":
        raise RuntimeError(f"the session did not start: {answer}; {answer.stderr[:500]}")
    needed = {i for c in session_checks for i in c.needs}
    ids = [i for i in m00.ORDER if i in needed]
    for n in range(1, repeat + 1):
        log(f"--- repeat {n} of {repeat}")
        set_syntax(ctx.project, fixed=False)
        ctx.batch, ctx.fixed = None, None
        if ids:
            ctx.batch = ctx.run(ids)
        if any(c.name == m00.FIX_CHECK for c in session_checks):
            set_syntax(ctx.project, fixed=True)
            ctx.fixed = ctx.run([m00.SYNTAX])
            set_syntax(ctx.project, fixed=False)
        for c in session_checks:
            run_check(c, n)


def cleanup(ctx, log):
    """Stops the session, then any game process the session started that is still alive."""
    if ctx.session is None and not (driver.pids("arma3_x64.exe") - ctx.game_pids_before):
        return
    ctx.stop_answer = ctx.sh("session", "stop", "--json", ceiling=180)
    ctx.stopped_at = time.monotonic()
    ctx.alive_after_stop = {pid for pid in ctx.game_pids if driver.alive(pid)}
    started = (driver.pids("arma3_x64.exe") - ctx.game_pids_before) | ctx.game_pids
    for pid in started:
        if driver.wait_gone(pid, 30) is None:
            log(f"game process {pid} still runs after session stop; stopping it by id")
            driver.kill(pid)


if __name__ == "__main__":
    sys.exit(main())
