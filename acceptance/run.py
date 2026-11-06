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

from harness import driver, m00, m01, m02, m03  # noqa: E402,F401  (importing a module registers its checks)
from harness.core import BATCHES, CHECKS, SETUPS, Context  # noqa: E402

DEFAULT_GAME = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3"
DEFAULT_CBA = r"C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\450814997"
DEFAULT_AGH = r"C:\Users\Shadow\Documents\arma-grappling-hook\.hemttout\build"


def find_exe(install):
    if install:
        return Path(install) / "shakedown.exe"
    on_path = shutil.which("shakedown")
    return Path(on_path) if on_path else REPO / "target" / "shakedown" / "shakedown.exe"


def make_project(work, game, install, cba, agh):
    """The project: checks/ copied, the numbered copies of the templates, and four setups.

    sp loads @skd_fixture alone, so it is a setup without CBA; cba loads CBA, both fixtures and
    arma-grappling-hook's built mod, as fixture-mod.md and the M1 demo say. ded is a dedicated
    server with two clients and a headless client loading the same mods as cba; ded30 is ded
    with a start ceiling of 30 s, which a start of about 45 s cannot meet (US-004).
    """
    project = work / "project"
    shutil.copytree(HERE / "checks", project / "checks")
    template = (HERE / "templates" / "NFR-RL-02-start-state.sqf").read_text(encoding="utf-8")
    for n in range(1, 21):
        (project / "checks" / "m01" / f"NFR-RL-02-start-state-{n:02}.sqf").write_text(
            template.replace("{N}", f"{n:02}"), encoding="utf-8")
    template = (HERE / "templates" / "US-008-start-state.sqf").read_text(encoding="utf-8")
    (project / "checks" / "us008").mkdir(exist_ok=True)
    for n in range(1, 22):
        name = (f"US-008 NFR-RL-02 check {n:02} of 20 sees the start state on every machine, then changes the scene"
                if n <= 20 else "US-008 the check after the 20th sees the start state on every machine")
        (project / "checks" / "us008" / f"US-008-start-state-{n:02}.sqf").write_text(
            template.replace("{NAME}", name).replace("{CHANGE}", "true" if n <= 20 else "false"), encoding="utf-8")
    fixture = Path(game) / "@skd_fixture"
    fixture_cba = Path(install) / "fixtures" / "@skd_fixture_cba"
    (project / "shakedown.toml").write_text(
        f"game = '{game}'\n\n"
        f"[mods.cba]\npath = '{cba}'\npatch = \"cba_main\"\n\n"
        f"[mods.fixture]\npath = '{fixture}'\npatch = \"skdfx_main\"\n\n"
        f"[mods.fixture-cba]\npath = '{fixture_cba}'\npatch = \"skdfx_cba\"\n\n"
        f"[mods.agh]\npath = '{agh}'\n\n"
        "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n\n"
        "[setups.cba]\nmode = \"singleplayer\"\nmods = [\"cba\", \"fixture\", \"fixture-cba\", \"agh\"]\n\n"
        "[setups.ded]\nmode = \"dedicated\"\nclients = 2\nheadless = 1\n"
        "mods = [\"cba\", \"fixture\", \"fixture-cba\", \"agh\"]\n\n"
        "[setups.ded30]\nmode = \"dedicated\"\nclients = 2\nheadless = 1\nstart_timeout = 30\n"
        "mods = [\"cba\", \"fixture\", \"fixture-cba\", \"agh\"]\n",
        encoding="utf-8")
    return project


def main():
    ap = argparse.ArgumentParser(description="Shakedown acceptance suite")
    ap.add_argument("--check", action="append", default=[],
                    help="run only checks whose name contains this text; repeat for several")
    ap.add_argument("--repeat", type=int, default=1, help="run the in-session checks this many times in each session")
    ap.add_argument("--install", default=os.environ.get("SHAKEDOWN_INSTALL"), help="the install folder holding shakedown.exe")
    ap.add_argument("--game", default=os.environ.get("SHAKEDOWN_GAME", DEFAULT_GAME), help="the Arma 3 folder")
    ap.add_argument("--cba", default=os.environ.get("SHAKEDOWN_CBA", DEFAULT_CBA), help="the CBA_A3 mod folder")
    ap.add_argument("--agh", default=os.environ.get("SHAKEDOWN_AGH", DEFAULT_AGH),
                    help="arma-grappling-hook's built mod folder")
    ap.add_argument("--work", help="the folder for the generated project and the report (default: a new temp folder)")
    ap.add_argument("--list", action="store_true", help="print the check names and exit")
    args = ap.parse_args()
    sys.stdout.reconfigure(errors="replace")  # an RPT line in a message may hold a character the console cannot show

    selected = [c for c in CHECKS if not args.check or any(f in c.name for f in args.check)]
    if args.list:
        for c in selected:
            where = f" {c.setup}" if c.phase == "session" else ""
            print(f"[{c.phase}{where}] {c.name}")
        return 0
    if not selected:
        print("No check name contains any of the --check values; --list prints the names.", file=sys.stderr)
        return 3

    exe = find_exe(args.install).resolve()
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

    project = make_project(work, args.game, exe.parent, args.cba, args.agh)
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

    in_session = [c for c in selected if c.phase in ("start", "session", "end")]
    if in_session:
        busy = driver.game_pids()
        if busy or driver.runner_pid() and driver.alive(driver.runner_pid()):
            log(f"Another game ({sorted(busy)}) or runner ({driver.runner_pid()}) is running; stop it first. "
                "No in-session check ran.")
            return 4
        for c in [c for c in selected if c.phase == "start"]:
            run_check(c, 1)
        setups = [s for s in SETUPS if any(c.phase == "session" and c.setup == s for c in selected)]
        if any(c.phase == "end" for c in selected) and "sp" not in setups:
            setups.append("sp")
        for setup in setups:
            ctx.game_pids_before = driver.game_pids()
            ctx.session, ctx.game_pids, ctx.session_runs, ctx.start_answer = None, set(), [], None
            checks = [c for c in selected if c.phase == "session" and c.setup == setup]
            try:
                run_session(ctx, setup, checks, args.repeat, run_check, log)
            except Exception as e:  # a session that did not start fails every check of its setup
                log(f"session of setup {setup}: {traceback.format_exc()}")
                for c in checks:
                    results.append((1, c.name, False, f"harness error: the session of setup {setup} failed: {e!r}"))
            finally:
                cleanup(ctx, setup, log)
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


def run_session(ctx, setup, checks, repeat, run_check, log):
    answer = ctx.sh("session", "start", "--setup", setup, "--json", ceiling=400)
    ctx.start_answer = answer
    ctx.runner_pid = driver.runner_pid()
    ctx.session = answer.json() if answer.stdout.strip() else None
    if ctx.session:
        ctx.game_pids = {m["pid"] for m in ctx.session["machines"]}
        ctx.sessions[setup] = ctx.session
    if answer.code != 0 or not ctx.session or ctx.session.get("state") != "ready":
        raise RuntimeError(f"the session did not start: {answer}; {answer.stderr[:500]}")
    for n in range(1, repeat + 1):
        log(f"--- setup {setup}, repeat {n} of {repeat}")
        ctx.m01, ctx.m02, ctx.m03 = {}, {}, {}
        for batch_setup, fn in BATCHES:
            if batch_setup == setup:
                try:
                    fn(ctx, checks)
                except Exception:  # the checks that read this batch fail on what it did not store
                    log(f"batch {fn.__name__}: {traceback.format_exc()}")
        for c in checks:
            run_check(c, n)


def cleanup(ctx, setup, log):
    """Stops the session, then any game process the session started that is still alive."""
    if ctx.session is None and not (driver.game_pids() - ctx.game_pids_before):
        return
    ctx.stop_answer = ctx.sh("session", "stop", "--json", ceiling=180)
    ctx.stopped_at = time.monotonic()
    ctx.alive_after_stop = {pid for pid in ctx.game_pids if driver.alive(pid)}
    ctx.stops[setup] = (ctx.stop_answer, set(ctx.alive_after_stop))
    started = (driver.game_pids() - ctx.game_pids_before) | ctx.game_pids
    for pid in started:
        if driver.wait_gone(pid, 30) is None:
            log(f"game process {pid} still runs after session stop; stopping it by id")
            driver.kill(pid)


if __name__ == "__main__":
    sys.exit(main())
