r"""The checks of M13 Every setup, repeated: US-074, US-075, US-076, US-077, US-078 and US-080.

Each part starts and stops what it needs itself, once per repeat (`--repeat`), with no session running before
it. The projects, in the work folder:

- `m13each`: the checks of `acceptance/m13/each/` (`US-074-every` with no `@setups`, `US-074-two` naming `sp` and
  `dedicated-2`, `US-074-one` naming `sp-cba` and tagged `solo`) and three setups in this order: `sp`
  (singleplayer, `@skd_fixture`), `sp-cba` (singleplayer, CBA and both fixtures) and `dedicated-2` (a dedicated
  server with two clients, `@skd_fixture`).
- `m13miss`: the same checks and setups, but `sp-cba` also loads the mod `ghost`, whose folder does not exist.
- `m13rep`: the checks of `acceptance/m13/rep/` and two setups, `sp` (singleplayer) and `ded` (a dedicated server
  with two clients), each loading `@skd_fixture` alone.
- `m13bad`: one check and a `shakedown.toml` whose `slow_step` is `-1` or `"10"`.

Per repeat:

- `each`: `run --each-setup` in `m13each` while the game processes are listed every second (US-074 1 to 3), then
  `run --each-setup --tag solo --json` (US-074 4).
- `miss`: `run --each-setup --json` in `m13miss` (US-075 1, 3).
- `kill`: in `m13each`, `run_start` with `each_setup` through `shakedown mcp`, `run_status` read until it ends,
  and the game process of `sp-cba` ended by its id while that setup's session starts (US-074 5, US-075 2).
- `sp`: one `sp` session in `m13rep`: `run --each-setup` refused, `US-076-steady` and `US-076-coin` with
  `--repeat 10`, the coin again through `run_start` with `repeat` 10 (US-076); the two slow checks, then again
  with `slow_step = 15` written into `shakedown.toml` (US-078).
- `ded`: one `ded` session in `m13rep`: `US-077-wait --repeat 5`, with `p2` ended by its id during the third
  repeat (US-077 1).
- `ded2`: `run --setup ded US-077-wait --repeat 3` with no session, with `p2` ended by its id during the first
  repeat (US-077 2).
- `bad`: `list` and `run --setup sp` in `m13bad` for each wrong `slow_step` (US-080).

Every game process a part saw is stopped by its id if it outlives the part. A part whose game start is lost to
`Error 20 reading file` of a base-game file is started again, at most three times in all.
"""

import json
import re
import shutil
import subprocess
import threading
import time
import traceback
from datetime import datetime
from pathlib import Path

from . import driver
from .core import check
from .m08 import game_folder, steam_running
from .m09 import poll_step
from .m06 import finish
from .m11 import result_of, status_of
from .m12 import cba_folder
from .mcp import Mcp, structured

ACC = Path(__file__).resolve().parent.parent
EVERY, TWO, ONE = "US-074-every", "US-074-two", "US-074-one"
SETUPS = ["sp", "sp-cba", "dedicated-2"]
EXPECTED = {"sp": [EVERY, TWO], "sp-cba": [EVERY, ONE], "dedicated-2": [EVERY, TWO]}
STEADY, COIN = "US-076-steady", "US-076-coin"
SLOW_STEP, SLOW_CHECK = "US-078-slow-step", "US-078-slow-check"
WAIT, WAIT_STEP = "US-077-wait", "p2 holds for 8 s"
STEP12, STEP11 = "the step that waits 12 s", "the step that waits 11 s"
RESULT = re.compile(r"Result: (\S.*?result\.json)")


# ================================================================ the projects

def keep(ctx):
    if not hasattr(ctx, "m13_keep"):
        ctx.m13_keep = {}
    return ctx.m13_keep


def mcp_json(project):
    (project / ".mcp.json").write_text(json.dumps(
        {"mcpServers": {"shakedown": {"command": "shakedown", "args": ["mcp"], "timeout": 70000}}}, indent=2),
        encoding="utf-8")


def each_toml(game, cba, fixture_cba, ghost=None):
    mods = (f"[mods.cba]\npath = '{cba}'\npatch = \"cba_main\"\n\n"
            f"[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n\n"
            f"[mods.fixture-cba]\npath = '{fixture_cba}'\npatch = \"skdfx_cba\"\n\n")
    if ghost:
        mods += f"[mods.ghost]\npath = '{ghost}'\n\n"
    cba_mods = '["cba", "fixture", "fixture-cba"' + (', "ghost"]' if ghost else "]")
    return (f"game = '{game}'\n\n{mods}"
            "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n\n"
            f"[setups.sp-cba]\nmode = \"singleplayer\"\nmods = {cba_mods}\n\n"
            "[setups.dedicated-2]\nmode = \"dedicated\"\nclients = 2\nmods = [\"fixture\"]\n")


def rep_toml(game, slow_step=None):
    line = f"slow_step = {slow_step}\n" if slow_step is not None else ""
    return (f"game = '{game}'\n{line}\n[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n\n"
            "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n\n"
            "[setups.ded]\nmode = \"dedicated\"\nclients = 2\nmods = [\"fixture\"]\n")


def projects(ctx):
    store = keep(ctx)
    if "projects" in store:
        return store["projects"]
    game = game_folder(ctx.project)
    cba = cba_folder(ctx.project)
    fixture_cba = Path(ctx.install) / "fixtures" / "@skd_fixture_cba"
    made = {}
    for name, source in (("m13each", "each"), ("m13miss", "each"), ("m13rep", "rep"), ("m13bad", "rep")):
        project = ctx.work / name
        if project.exists():
            shutil.rmtree(project)
        shutil.copytree(ACC / "m13" / source, project / "tests")
        mcp_json(project)
        made[name] = (project, driver.Shakedown(ctx.sh.exe, project, ctx.log))
    (made["m13each"][0] / "shakedown.toml").write_text(each_toml(game, cba, fixture_cba), encoding="utf-8")
    ghost = made["m13miss"][0] / "no-such-mod" / "@ghost"
    (made["m13miss"][0] / "shakedown.toml").write_text(each_toml(game, cba, fixture_cba, ghost), encoding="utf-8")
    (made["m13rep"][0] / "shakedown.toml").write_text(rep_toml(game), encoding="utf-8")
    (made["m13bad"][0] / "shakedown.toml").write_text(rep_toml(game), encoding="utf-8")
    store["projects"] = made
    store["game"] = game
    store["ghost"] = ghost
    return made


def machine_fault(ctx, since):
    for name in ("m13each", "m13miss", "m13rep"):
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
                ctx.log(f"M13 part {k}: {traceback.format_exc()}")
                store[k] = e
                fault = machine_fault(ctx, began)
                if not fault or attempt == 3:
                    break
                ctx.log(f"M13 part {k}: a game process could not read a base-game file ({fault}); the machine, "
                        f"not the framework. Starting the part again, attempt {attempt + 1} of 3")
    value = store[k]
    if isinstance(value, Exception):
        raise AssertionError(f"the M13 part {key} failed: {value!r}")
    return value


# ================================================================ helpers

class Procs:
    """Lists the game processes about once a second in a thread of its own, at most `ceiling` s, and keeps, for
    each process id, its image and the first and last time it was seen."""

    def __init__(self, ceiling=1800):
        self.seen = {}           # pid -> [image, first, last]
        self.ceiling = ceiling
        self.done = threading.Event()
        self.thread = threading.Thread(target=self._loop, daemon=True)
        self.thread.start()

    def _loop(self):
        for _ in range(self.ceiling):
            now = time.monotonic()
            for image in ("arma3_x64.exe", "arma3server_x64.exe"):
                try:
                    found = driver.pids(image)
                except Exception:  # noqa: BLE001 - one missed sample is not a verdict
                    continue
                for pid in found:
                    rec = self.seen.setdefault(pid, [image, now, now])
                    rec[2] = now
            if self.done.wait(1):
                return

    def stop(self):
        self.done.set()
        self.thread.join(timeout=30)
        return self

    def pids(self):
        return set(self.seen)

    def groups(self, sizes):
        """The process ids in the order first seen, cut into groups of these sizes; asserts each group's last
        sighting is before the next group's first, so no two groups ran at once."""
        order = sorted(self.seen, key=lambda p: self.seen[p][1])
        assert len(order) == sum(sizes), \
            f"{len(order)} game processes were seen, expected {sum(sizes)} in groups of {sizes}: {self.describe()}"
        groups, i = [], 0
        for n in sizes:
            groups.append(order[i:i + n])
            i += n
        for a, b in zip(groups, groups[1:]):
            last = max(self.seen[p][2] for p in a)
            first = min(self.seen[p][1] for p in b)
            assert last < first, f"the processes {a} were still seen {last - first:.1f} s after {b} appeared: {self.describe()}"
        return groups

    def describe(self):
        t0 = min((r[1] for r in self.seen.values()), default=0)
        return {pid: (r[0], round(r[1] - t0, 1), round(r[2] - t0, 1)) for pid, r in self.seen.items()}


def need_quiet(sh):
    assert steam_running(), "no steam process runs; a game start would be refused"
    busy = driver.game_pids()
    assert not busy, f"game processes run before the part starts: {sorted(busy)}"
    st = status_of(sh)
    assert st.get("session") is None, f"status shows a session before the part starts: {(st['session'] or {}).get('id')}"


def tidy(ctx, sh, pids):
    """Stops a session the part left, then every game process it saw that is still alive, by its id."""
    try:
        if status_of(sh).get("session"):
            sh("session", "stop", "--json", ceiling=180)
    except Exception as e:  # noqa: BLE001
        ctx.log(f"M13: status or session stop after the part: {e!r}")
    for pid in set(pids):
        if driver.alive(pid) and driver.wait_gone(pid, 30) is None:
            ctx.log(f"M13: game process {pid} still runs; stopping it by id")
            driver.kill(pid)
    ctx.runner_pid = driver.runner_pid() or ctx.runner_pid


def after(sh):
    return {"status": status_of(sh), "game": driver.game_pids()}


def read_result(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def runs_made(sh, before):
    return sorted(set(sh.run_folders()) - set(before))


def kill_pid(ctx, pid, who):
    subprocess.run(["taskkill", "/PID", str(pid), "/F"], capture_output=True, timeout=30)
    ctx.log(f"M13: ended {who}, pid {pid}, by its id with taskkill /F")
    return time.time()


def machines_of(st):
    return {m["name"]: m for m in (st.get("session") or {}).get("machines", [])}


def quiet(sh):
    return driver.Shakedown(sh.exe, sh.project, lambda text: None)


def summary_of(counts):
    """DM-09 read from the counts."""
    ran = {k: counts[k] for k in ("pass", "fail", "invalid")}
    if not any(ran.values()):
        return "not-run"
    if ran["pass"] and ran["fail"]:
        return "flaky"
    if ran["pass"]:
        return "pass"
    if ran["fail"]:
        return "fail"
    return "invalid"


def counts_in(result, check_id):
    c = {"pass": 0, "fail": 0, "invalid": 0, "not_run": 0}
    for ex in result["checks"]:
        if ex["check_id"] == check_id:
            c[ex["outcome"]] += 1
    c["not_run"] = sum(1 for n in result["not_run"] if n["check_id"] == check_id)
    return c


def valid(ctx, result, what):
    errors = ctx.validate(result)
    assert not errors, f"{what}: the result file is not valid against result.schema.json: {errors[:5]}"


def lines(text):
    return [ln.rstrip() for ln in text.splitlines()]


# ================================================================ the parts

def each_part(ctx):
    project, sh = projects(ctx)["m13each"]
    need_quiet(sh)
    out, seen = {}, set()
    try:
        before = sh.run_folders()
        procs = Procs()
        try:
            out["answer"] = sh("run", "--each-setup", ceiling=1500)
        finally:
            procs.stop()
            seen |= procs.pids()
        out["procs"] = procs
        out["after"] = after(sh)
        out["folders"] = runs_made(sh, before)
        out["results"] = {}
        for path in RESULT.findall(out["answer"].stdout):
            out["results"][path] = read_result(path)
        ctx.log(f"M13 each: {out['answer']}; folders {out['folders']}; processes {procs.describe()}")
        before = sh.run_folders()
        procs = Procs()
        try:
            out["solo"] = sh("run", "--each-setup", "--tag", "solo", "--json", ceiling=900)
        finally:
            procs.stop()
            seen |= procs.pids()
        out["solo_procs"] = procs
        out["solo_after"] = after(sh)
        out["solo_folders"] = runs_made(sh, before)
        ctx.log(f"M13 each solo: {out['solo']}; folders {out['solo_folders']}; processes {procs.describe()}")
    finally:
        tidy(ctx, sh, seen)
    return out


def miss_part(ctx):
    project, sh = projects(ctx)["m13miss"]
    need_quiet(sh)
    out, seen = {}, set()
    try:
        before = sh.run_folders()
        procs = Procs()
        try:
            out["answer"] = sh("run", "--each-setup", "--json", ceiling=1500)
        finally:
            procs.stop()
            seen |= procs.pids()
        out["procs"] = procs
        out["after"] = after(sh)
        out["folders"] = runs_made(sh, before)
        ctx.log(f"M13 miss: {out['answer']}; folders {out['folders']}; stdout {out['answer'].stdout[:1500]}")
    finally:
        tidy(ctx, sh, seen)
    return out


def kill_part(ctx):
    project, sh = projects(ctx)["m13each"]
    need_quiet(sh)
    out, seen = {"statuses": [], "killed": []}, set()

    def log(text):  # keeps every line but the polled run_status calls
        if "mcp run_status" not in text:
            ctx.log(text)
    try:
        with Mcp(project, ctx.install, log) as m:
            began = time.monotonic()
            result, took = m.call("run_start", {"each_setup": True}, ceiling=70)
            out["start"], out["start_s"] = structured(result), took
            run_id = out["start"]["run_id"]
            before_cba = set()
            for _ in range(3600):  # twice a second, at most 30 min
                st = structured(m.call("run_status", {"run_id": run_id}, ceiling=70)[0])
                out["statuses"].append((time.monotonic() - began, st))
                game = driver.game_pids()
                seen |= game
                runs = {r["setup"]: r for r in st.get("runs") or []}
                cba = runs.get("sp-cba", {}).get("state")
                if cba in (None, "waiting"):
                    before_cba |= game
                elif cba == "starting-session" and not out["killed"]:
                    new = game - before_cba
                    for pid in sorted(new):
                        out["killed"].append((pid, kill_pid(ctx, pid, "a game process of sp-cba during its start")))
                if st["state"] in ("finished", "stopped"):
                    break
                time.sleep(0.5)
            out["final"] = st
            out["result"] = structured(m.call("run_result", {"run_id": run_id}, ceiling=70)[0])
        out["after"] = after(sh)
        seen |= out["after"]["game"]
        ctx.log(f"M13 kill: start {json.dumps(out['start'])}; killed {out['killed']}; final {json.dumps(out['final'])[:800]}")
    finally:
        tidy(ctx, sh, seen)
    return out


def start_session(sh, setup, pids):
    answer = sh("session", "start", "--setup", setup, "--json", ceiling=400)
    session = answer.json() if answer.stdout.strip() else {}
    pids |= {m["pid"] for m in session.get("machines", []) if m.get("pid")}
    assert answer.code == 0 and session.get("state") == "ready", \
        f"{answer}: state {session.get('state')}, reason {str(session.get('reason'))[:600]}"
    return session


def text_run(sh, *args, ceiling=900):
    answer = sh("run", *args, ceiling=ceiling)
    paths = RESULT.findall(answer.stdout)
    return {"answer": answer, "result": read_result(paths[-1]) if paths else None, "path": paths[-1] if paths else None}


def sp_part(ctx):
    project, sh = projects(ctx)["m13rep"]
    game = keep(ctx)["game"]
    toml = project / "shakedown.toml"
    toml.write_text(rep_toml(game), encoding="utf-8")
    need_quiet(sh)
    pids, out = set(), {}
    try:
        out["session"] = start_session(sh, "sp", pids)
        out["each_refused"] = sh("run", "--each-setup", STEADY, ceiling=60)
        out["steady"] = result_of(sh("run", STEADY, "--repeat", "10", "--json", ceiling=900))
        out["coin"] = text_run(sh, COIN, "--repeat", "10")
        with Mcp(project, ctx.install, ctx.log) as m:
            start = structured(m.call("run_start", {"ids": [COIN], "repeat": 10}, ceiling=70)[0])
            st = None
            for _ in range(40):  # wait_s 30 each, at most 20 min
                st = structured(m.call("run_status", {"run_id": start["run_id"], "wait_s": 30}, ceiling=70)[0])
                if st["state"] in ("finished", "stopped"):
                    break
            out["mcp"] = {"start": start, "status": st,
                          "result": structured(m.call("run_result", {"run_id": start["run_id"]}, ceiling=70)[0])}
        out["slow10"] = text_run(sh, SLOW_STEP, SLOW_CHECK)
        toml.write_text(rep_toml(game, 15), encoding="utf-8")
        out["slow15"] = text_run(sh, SLOW_STEP, SLOW_CHECK)
        ctx.log(f"M13 sp: coin {out['coin']['answer'].stdout[-600:]}\nslow10 {out['slow10']['answer'].stdout}\n"
                f"slow15 {out['slow15']['answer'].stdout}")
    finally:
        toml.write_text(rep_toml(game), encoding="utf-8")
        tidy(ctx, sh, pids)
    return out


def wait_for_done(sh, bg, n, ceiling=300):
    """Polls status twice a second until the run has `n` executions done and the check running; the status."""
    q = quiet(sh)
    for _ in range(ceiling * 2):
        if bg.poll() is not None:
            return None
        try:
            st = status_of(q)
        except AssertionError:
            st = {}
        run = st.get("run") or {}
        if run.get("done") == n and run.get("current_check") == WAIT:
            return st
        time.sleep(0.5)
    return None


def ded_part(ctx):
    project, sh = projects(ctx)["m13rep"]
    need_quiet(sh)
    pids, out = set(), {}
    try:
        out["session"] = start_session(sh, "ded", pids)
        p2 = machines_of(status_of(sh))["p2"]["pid"]
        args = ("run", WAIT, "--repeat", "5", "--json")
        bg = sh.start(*args)
        out["at_two"] = wait_for_done(sh, bg, 2)
        found = poll_step(ctx, sh, bg, WAIT, WAIT_STEP) if out["at_two"] else None
        out["done_at_kill"] = ((status_of(sh).get("run") or {}).get("done")) if found else None
        out["killed_at"] = kill_pid(ctx, p2, "p2 during the third repeat") if found else None
        answer = finish(ctx, bg, args, ceiling=400)
        assert found, f"the third repeat never ran its wait on p2; the run: {answer}; {answer.stdout[:500]}"
        out["run"] = result_of(answer)
        out["broken"] = status_of(sh)
    finally:
        tidy(ctx, sh, pids)
    return out


def ded2_part(ctx):
    project, sh = projects(ctx)["m13rep"]
    need_quiet(sh)
    out, seen = {}, set()
    try:
        args = ("run", "--setup", "ded", WAIT, "--repeat", "3", "--json")
        bg = sh.start(*args)
        at = wait_for_done(sh, bg, 0, ceiling=600)
        found = poll_step(ctx, sh, bg, WAIT, WAIT_STEP) if at else None
        if found:
            st = status_of(sh)
            ms = machines_of(st)
            seen |= {m["pid"] for m in ms.values() if m.get("pid")}
            out["done_at_kill"] = (st.get("run") or {}).get("done")
            out["killed_at"] = kill_pid(ctx, ms["p2"]["pid"], "p2 during the first repeat")
        answer = finish(ctx, bg, args, ceiling=600)
        assert found, f"the first repeat never ran its wait on p2; the run: {answer}; {answer.stdout[:500]}"
        out["run"] = result_of(answer)
        out["after"] = after(sh)
        out["left"] = {pid for pid in seen if driver.alive(pid)}
    finally:
        tidy(ctx, sh, seen)
    return out


def bad_part(ctx):
    project, sh = projects(ctx)["m13bad"]
    game = keep(ctx)["game"]
    toml = project / "shakedown.toml"
    out = {}
    try:
        for name, value in (("minus", "-1"), ("string", '"10"')):
            text = rep_toml(game, value)
            line = text.splitlines().index(f"slow_step = {value}") + 1
            toml.write_text(text, encoding="utf-8")
            game_before = driver.game_pids()
            out[name] = {"line": line, "list": sh("list", ceiling=60),
                         "run": sh("run", "--setup", "sp", ceiling=120),
                         "run_session": sh("run", ceiling=120)}
            out[name]["game_before"], out[name]["game_after"] = game_before, driver.game_pids()
            out[name]["status"] = status_of(sh)
    finally:
        toml.write_text(rep_toml(game), encoding="utf-8")
    return out


# ================================================================ US-074

def each(ctx):
    return once(ctx, "each", each_part)


def by_setup(results):
    """The result files by their setup's name, and the setups in the order their runs started."""
    named = {r["session"]["setup"]["name"]: r for r in results}
    order = [r["session"]["setup"]["name"] for r in sorted(results, key=lambda r: r["run"]["started_at"])]
    return named, order


def attached(ex):
    return {e["label"]: e["value"] for e in ex["evidence"]["attached"]}


@check("US-074 RUN-07 run --each-setup starts a session of sp, sp-cba and dedicated-2 in the project file's order, one at a time, runs the checks that name each, and stops each before the next starts", "start", repeat=True)
def us074_order(ctx):
    e = each(ctx)
    named, order = by_setup(list(e["results"].values()))
    assert order == SETUPS, f"the setups ran in the order {order}, expected {SETUPS}; {e['answer'].stdout[-800:]}"
    for setup in SETUPS:
        r = named[setup]
        ran = [c["check_id"] for c in r["checks"]]
        assert sorted(ran) == sorted(EXPECTED[setup]), f"{setup} ran {ran}, expected {EXPECTED[setup]}"
        assert r["not_run"] == [], f"{setup} lists checks not run: {r['not_run']}"
        for ex in r["checks"]:
            assert ex["outcome"] == "pass", f"{setup} {ex['check_id']}: {ex['outcome']} {ex['reason']}"
        every = attached(next(c for c in r["checks"] if c["check_id"] == EVERY))
        assert every.get("multiplayer") is (setup == "dedicated-2"), f"{setup}: isMultiplayer read {every}"
        assert every.get("cba on p1") is (setup == "sp-cba"), f"{setup}: CBA on p1 read {every}"
    ids = [named[s]["session"]["id"] for s in SETUPS]
    assert len(set(ids)) == 3 and all(ids), f"the session ids {ids}"
    for a, b in zip(SETUPS, SETUPS[1:]):
        assert named[a]["run"]["ended_at"] <= named[b]["run"]["started_at"], \
            f"the run of {b} started at {named[b]['run']['started_at']}, before {a}'s ended at {named[a]['run']['ended_at']}"
    groups = e["procs"].groups([1, 1, 3])
    images = [sorted(e["procs"].seen[p][0] for p in g) for g in groups]
    assert images[2].count("arma3server_x64.exe") == 1, f"the third setup's processes {images[2]}"
    ctx.log(f"US-074: processes by setup {groups}, {e['procs'].describe()}")


@check("US-074 RUN-07 RES-01 run --each-setup leaves one run folder with a valid result.json per setup, prints one line per check execution and one per setup, and exits with the highest exit code of the three runs, 0", "start", repeat=True)
def us074_files(ctx):
    e = each(ctx)
    a = e["answer"]
    assert a.code == 0, f"{a}: {a.stdout[-600:]} {a.stderr[-300:]}"
    assert len(e["folders"]) == 3, f"the command made the run folders {e['folders']}, expected 3"
    folders = sorted(Path(p).parent.name for p in e["results"])
    assert folders == e["folders"], f"the setup lines name the result files of {folders}, the new run folders are {e['folders']}"
    for path, r in e["results"].items():
        valid(ctx, r, path)
    out = lines(a.stdout)
    execs = [ln for ln in out if re.match(r"^(pass|fail|invalid)\s+US-074-", ln)]
    assert len(execs) == 6, f"{len(execs)} lines of check executions, expected 6: {out}"
    setups = [m.group(1) for m in (re.match(r"^setup (\S+?):", ln) for ln in out) if m]
    assert setups == SETUPS, f"the setup lines name {setups}, expected {SETUPS}: {out}"


@check("US-074 STP-02 after run --each-setup no arma3_x64 or arma3server_x64 process runs and status shows no session", "start", repeat=True)
def us074_nothing_left(ctx):
    e = each(ctx)
    for key in ("after", "solo_after"):
        left = e[key]
        assert not left["game"], f"{key}: game processes run: {sorted(left['game'])}"
        assert left["status"].get("session") is None, f"{key}: status shows a session {left['status'].get('session')}"


@check("US-074 RUN-07 run --each-setup --tag solo starts only sp-cba, the one setup the check tagged solo names, and its --json gives the run_result of the whole command with a runs entry for it", "start", repeat=True)
def us074_solo(ctx):
    e = each(ctx)
    a = e["solo"]
    assert a.code == 0, f"{a}: {a.stdout[-600:]} {a.stderr[-300:]}"
    out = a.json()
    assert [r["setup"] for r in out["runs"]] == ["sp-cba"], f"runs: {[r.get('setup') for r in out['runs']]}"
    assert len(e["solo_folders"]) == 1, f"run folders made: {e['solo_folders']}"
    assert len(e["solo_procs"].seen) == 1, f"game processes seen: {e['solo_procs'].describe()}"
    assert out["summary"]["total"] == 1 and out["summary"]["pass"] == 1, f"summary {out['summary']}"
    assert [(o["check_id"], o["setup"], o["outcome"]) for o in out["outcomes"]] == [(ONE, "sp-cba", "pass")], \
        f"outcomes {out['outcomes']}"
    run = out["runs"][0]
    r = read_result(run["result_file"])
    valid(ctx, r, run["result_file"])
    assert r["session"]["setup"]["name"] == "sp-cba" and [c["check_id"] for c in r["checks"]] == [ONE], \
        f"the result file ran {[c['check_id'] for c in r['checks']]} in {r['session']['setup']['name']}"


# ================================================================ US-074 5 and US-075 2: through MCP, a start that fails

def kill(ctx):
    return once(ctx, "kill", kill_part)


@check("US-074 RUN-07 AGT-02 run_start with each_setup returns at once with an e- handle and the setups in order, and run_status names the setup it is in", "start", repeat=True)
def us074_mcp(ctx):
    k = kill(ctx)
    s = k["start"]
    assert k["start_s"] < 5, f"run_start took {k['start_s']:.1f} s"
    assert re.match(r"^e-\S+-\d+$", s["run_id"]), f"run_id {s['run_id']}"
    assert s.get("setups") == SETUPS, f"setups {s.get('setups')}"
    assert s["state"] in ("running", "starting-session"), f"state {s['state']}"
    assert s["total"] == 6, f"total {s['total']}, expected 6 executions over the three setups"
    named = [st.get("setup") for _, st in k["statuses"] if st["state"] not in ("finished", "stopped")]
    distinct = [x for i, x in enumerate(named) if x and (i == 0 or named[i - 1] != x)]
    assert distinct in (SETUPS, ["sp", "dedicated-2"]), f"run_status named the setups {distinct}"
    for _, st in k["statuses"]:
        assert [r["setup"] for r in st["runs"]] == SETUPS, f"run_status runs {st['runs']}"
        going = [r["setup"] for r in st["runs"] if r["state"] in ("starting-session", "running")]
        if st["state"] not in ("finished", "stopped") and going:
            assert st["setup"] == going[0], f"run_status names {st['setup']} while runs says {st['runs']}"
    assert k["final"]["setup"] is None, f"the ended run's setup is {k['final']['setup']}"


@check("US-075 RUN-07 SES-04 a game process of the second of three setups ended by its id during its start: its checks are not-run session-failed with the start's reason in its result file, the first and third setups run to their ends, and no game process is left", "start", repeat=True)
def us075_killed(ctx):
    k = kill(ctx)
    assert k["killed"], f"no game process of sp-cba was seen during its start; statuses {[st['runs'] for _, st in k['statuses']][-5:]}"
    states = [r["state"] for r in k["final"]["runs"]]
    assert states == ["finished", "session-failed", "finished"], f"runs ended {k['final']['runs']}"
    res = k["result"]
    runs = {r["setup"]: r for r in res["runs"]}
    assert list(runs) == SETUPS, f"run_result runs {list(runs)}"
    reason = runs["sp-cba"].get("session_reason")
    assert reason and re.search(r"exit", json.dumps(reason), re.I), f"sp-cba's session_reason {reason}"
    for setup in ("sp", "dedicated-2"):
        r = read_result(runs[setup]["result_file"])
        valid(ctx, r, runs[setup]["result_file"])
        assert sorted(c["check_id"] for c in r["checks"]) == sorted(EXPECTED[setup]) and \
            all(c["outcome"] == "pass" for c in r["checks"]), f"{setup}: {[(c['check_id'], c['outcome']) for c in r['checks']]}"
    path = runs["sp-cba"]["result_file"]
    r = read_result(path)
    valid(ctx, r, path)
    assert r["checks"] == [], f"sp-cba executed {[c['check_id'] for c in r['checks']]}"
    nr = sorted((n["check_id"], n["reason"]) for n in r["not_run"])
    assert nr == sorted((c, "session-failed") for c in EXPECTED["sp-cba"]), f"sp-cba's not_run {r['not_run']}"
    for n in r["not_run"]:
        assert n.get("message") and re.search(r"exit", n["message"], re.I), f"the not_run message {n}"
    assert r["session"]["id"], f"the failed start's result file names no session: {r['session']}"
    assert res["summary"]["not_run"] == 2 and res["summary"]["pass"] == 4, f"summary {res['summary']}"
    assert not k["after"]["game"], f"game processes run after the command: {sorted(k['after']['game'])}"
    assert k["after"]["status"].get("session") is None, f"status shows a session {k['after']['status'].get('session')}"
    ctx.log(f"US-075: the killed start's reason {json.dumps(reason)[:400]}")


# ================================================================ US-075 1, 3: a refused start

def miss(ctx):
    return once(ctx, "miss", miss_part)


@check("US-075 RUN-07 SES-03 the second of three setups naming a mod folder that does not exist is refused with that folder, its checks are not-run session-failed with that reason in a result file of its own, and the first and third setups run to their ends", "start", repeat=True)
def us075_refused(ctx):
    m = miss(ctx)
    ghost = keep(ctx)["ghost"]
    out = m["answer"].json()
    runs = {r["setup"]: r for r in out["runs"]}
    assert list(runs) == SETUPS, f"runs {list(runs)}"
    reason = json.dumps(runs["sp-cba"].get("session_reason"))
    assert "no-such-mod" in reason, f"sp-cba's session_reason does not name the folder {ghost}: {reason}"
    for setup in ("sp", "dedicated-2"):
        r = read_result(runs[setup]["result_file"])
        valid(ctx, r, setup)
        assert sorted(c["check_id"] for c in r["checks"]) == sorted(EXPECTED[setup]) and \
            all(c["outcome"] == "pass" for c in r["checks"]), f"{setup}: {[(c['check_id'], c['outcome']) for c in r['checks']]}"
    path = runs["sp-cba"]["result_file"]
    assert path, f"sp-cba has no result file: {runs['sp-cba']}"
    r = read_result(path)
    valid(ctx, r, path)
    assert r["checks"] == [] and r["session"]["id"] is None and r["session"]["machines"] == [], \
        f"the refused setup's result file: checks {r['checks']}, session {r['session']}"
    assert r["run"]["state"] == "finished", f"its run state {r['run']['state']}"
    nr = sorted((n["check_id"], n["reason"]) for n in r["not_run"])
    assert nr == sorted((c, "session-failed") for c in EXPECTED["sp-cba"]), f"sp-cba's not_run {r['not_run']}"
    for n in r["not_run"]:
        assert "no-such-mod" in (n.get("message") or ""), f"the not_run message does not name the folder: {n}"
    assert len(m["folders"]) == 3, f"run folders made: {m['folders']}"
    m["procs"].groups([1, 3])
    ctx.log(f"US-075: refused with {r['not_run'][0].get('message')}")


@check("US-075 RUN-07 STP-02 run --each-setup with the refused setup exits 2, and after it no game process runs and status shows no session", "start", repeat=True)
def us075_exit(ctx):
    m = miss(ctx)
    a = m["answer"]
    assert a.code == 2, f"{a}: {a.stdout[-400:]} {a.stderr[-300:]}"
    assert not m["after"]["game"], f"game processes run: {sorted(m['after']['game'])}"
    assert m["after"]["status"].get("session") is None, f"status shows a session {m['after']['status'].get('session')}"


# ================================================================ US-076, US-078 and RUN-07's refusal: one sp session

def sp(ctx):
    return once(ctx, "sp", sp_part)


def repeats_of(result, check_id):
    found = [r for r in result.get("repeats") or [] if r["check_id"] == check_id]
    assert len(found) == 1, f"the result file's repeats hold {len(found)} entries of {check_id}: {result.get('repeats')}"
    return found[0]


@check("RUN-07 run --each-setup with a session running is refused with exit code 4 naming session stop", "start", repeat=True)
def run07_refused(ctx):
    a = sp(ctx)["each_refused"]
    assert a.code == 4, f"{a}: {a.stdout[:300]} {a.stderr[:300]}"
    assert "session stop" in a.stdout + a.stderr, f"the refusal does not name session stop: {a.stdout[:300]} {a.stderr[:300]}"


@check("US-076 OUT-02 DM-09 RES-01 run US-076-steady --repeat 10 against a running session runs it ten times in a row, each its own execution, and the result file gives ten executions with repeat 1 to 10, run.repeat 10 and its summary with the count of each outcome", "start", repeat=True)
def us076_ten(ctx):
    s = sp(ctx)["steady"]
    r = s["result"]
    valid(ctx, r, "steady")
    assert r["run"]["repeat"] == 10, f"run.repeat {r['run']['repeat']}"
    ex = r["checks"]
    assert [(c["check_id"], c["repeat"]) for c in ex] == [(STEADY, n) for n in range(1, 11)], \
        f"executions {[(c['check_id'], c['repeat']) for c in ex]}"
    for a, b in zip(ex, ex[1:]):
        assert a["ended_at"] <= b["started_at"], f"repeat {b['repeat']} started before {a['repeat']} ended"
    rep = repeats_of(r, STEADY)
    assert rep["counts"] == counts_in(r, STEADY), f"repeats {rep}, the executions count {counts_in(r, STEADY)}"
    assert r["summary"]["total"] == 10, f"summary {r['summary']}"


@check("US-076 OUT-02 DM-09 a check whose ten repeats all pass is pass and the run exits 0", "start", repeat=True)
def us076_pass(ctx):
    s = sp(ctx)["steady"]
    assert s["answer"].code == 0, f"{s['answer']}"
    rep = repeats_of(s["result"], STEADY)
    assert rep["outcome"] == "pass" and rep["counts"] == {"pass": 10, "fail": 0, "invalid": 0, "not_run": 0}, f"{rep}"


@check("US-076 OUT-02 DM-09 the check that passes about one time in two repeated 10 times is flaky, its printed line gives the counts, and the run exits 1", "start", repeat=True)
def us076_flaky(ctx):
    c = sp(ctx)["coin"]
    r = c["result"]
    assert r, f"the run printed no result file: {c['answer'].stdout[-500:]}"
    valid(ctx, r, "coin")
    counts = counts_in(r, COIN)
    assert counts["pass"] and counts["fail"], \
        f"the coin fell the same way 10 times (about 1 in 500): {counts}; run the check again"
    rep = repeats_of(r, COIN)
    assert rep["outcome"] == "flaky" and rep["counts"] == counts, f"repeats {rep}, the executions count {counts}"
    assert c["answer"].code == 1, f"{c['answer']}"
    out = lines(c["answer"].stdout)
    summary = [ln for ln in out if re.match(rf"^flaky\s+{COIN}\b", ln)]
    assert len(summary) == 1, f"no single flaky line for {COIN}: {out}"
    nums = dict((k, int(v)) for v, k in re.findall(r"(\d+) (pass|fail|invalid|not run)", summary[0]))
    assert nums == {"pass": counts["pass"], "fail": counts["fail"], "invalid": 0, "not run": 0}, \
        f"the line {summary[0]!r}, the result file counts {counts}"
    marked = [ln for ln in out if re.match(rf"^(pass|fail)\s+{COIN} #\d+", ln)]
    assert len(marked) == 9, f"{len(marked)} execution lines marked #<repeat>, expected 9: {out}"


@check("US-076 OUT-02 AGT-02 run_start with repeat 10 through MCP gives in run_result the same summary as its result file, with the setup", "start", repeat=True)
def us076_mcp(ctx):
    m = sp(ctx)["mcp"]
    assert m["start"]["total"] == 10, f"run_start total {m['start']['total']}"
    res = m["result"]
    r = read_result(res["result_file"])
    valid(ctx, r, "the coin through MCP")
    counts = counts_in(r, COIN)
    want = {"check_id": COIN, "outcome": summary_of(counts), "counts": counts, "setup": "sp"}
    assert res.get("repeats") == [want], f"run_result repeats {res.get('repeats')}, expected {[want]}"
    assert repeats_of(r, COIN)["outcome"] == want["outcome"], f"the result file's repeats {r['repeats']}"
    assert [(o["repeat"], o["setup"]) for o in res["outcomes"]] == [(n, "sp") for n in range(1, 11)], \
        f"outcomes {[(o.get('repeat'), o.get('setup')) for o in res['outcomes']]}"
    assert all(isinstance(o.get("slow"), bool) for o in res["outcomes"]), f"outcomes lack slow: {res['outcomes'][:2]}"


def step_named(ex, title):
    found = [s for s in ex["steps"] if s["path"] == [title]]
    assert len(found) == 1, f"{ex['check_id']}: {len(found)} steps titled {title!r}: {[s['path'] for s in ex['steps']]}"
    return found[0]


def exec_of(result, cid):
    found = [c for c in result["checks"] if c["check_id"] == cid]
    assert len(found) == 1, f"{len(found)} executions of {cid}"
    return found[0]


@check("US-078 TIM-01 OBS-05 a step that waits 12 s on a condition that then holds ends pass, the result file flags that step slow with its duration, and the printed line names the slow step", "start", repeat=True)
def us078_slow_step(ctx):
    run = sp(ctx)["slow10"]
    r = run["result"]
    assert r, f"the run printed no result file: {run['answer'].stdout[-500:]}"
    valid(ctx, r, "slow10")
    assert r["run"]["slow_step_s"] == 10, f"run.slow_step_s {r['run']['slow_step_s']}"
    ex = exec_of(r, SLOW_STEP)
    assert ex["outcome"] == "pass", f"{ex['outcome']} {ex['reason']}"
    st = step_named(ex, STEP12)
    assert st["slow"] is True and 11.5 < st["duration_s"] < 20, f"the 12 s step: slow {st['slow']}, {st['duration_s']} s"
    assert step_named(ex, "the quick step")["slow"] is False, "the quick step is flagged slow"
    out = lines(run["answer"].stdout)
    i = next((n for n, ln in enumerate(out) if re.match(rf"^pass\s+{SLOW_STEP}\b", ln)), None)
    assert i is not None, f"no pass line for {SLOW_STEP}: {out}"
    block = " ".join(out[i:i + 4])
    assert STEP12 in block and re.search(r"slow", block) and re.search(r"\b1[1-9](\.\d+)? s", block), \
        f"the lines for {SLOW_STEP} do not name the slow step with its duration: {out[i:i + 4]}"


@check("US-078 TIM-01 CFG-01 with slow_step = 15 in shakedown.toml the same step is not flagged", "start", repeat=True)
def us078_threshold(ctx):
    run = sp(ctx)["slow15"]
    r = run["result"]
    assert r, f"the run printed no result file: {run['answer'].stdout[-500:]}"
    valid(ctx, r, "slow15")
    assert r["run"]["slow_step_s"] == 15, f"run.slow_step_s {r['run']['slow_step_s']}"
    ex = exec_of(r, SLOW_STEP)
    st = step_named(ex, STEP12)
    assert ex["outcome"] == "pass" and st["slow"] is False and st["duration_s"] < 15, \
        f"{ex['outcome']}; the 12 s step: slow {st['slow']}, {st['duration_s']} s"
    assert STEP12 not in run["answer"].stdout, f"the lines still name the 12 s step: {run['answer'].stdout}"


@check("US-078 TIM-01 a check with @timeout 20 that takes 11 s is flagged slow as a check, and one of @timeout 60 that takes 12 s is not", "start", repeat=True)
def us078_slow_check(ctx):
    for key in ("slow10", "slow15"):
        run = sp(ctx)[key]
        r = run["result"]
        assert r, f"{key}: the run printed no result file"
        ex = exec_of(r, SLOW_CHECK)
        assert ex["outcome"] == "pass" and ex["slow"] is True and 10 < ex["duration_s"] < 20, \
            f"{key}: {SLOW_CHECK} {ex['outcome']}, slow {ex['slow']}, {ex['duration_s']} s"
        other = exec_of(r, SLOW_STEP)
        assert other["slow"] is False and other["duration_s"] < 30, f"{key}: {SLOW_STEP} slow {other['slow']}, {other['duration_s']} s"
        line = next((ln for ln in lines(run["answer"].stdout) if re.match(rf"^pass\s+{SLOW_CHECK}\b", ln)), "")
        assert "slow" in line.replace(SLOW_CHECK, ""), f"{key}: the line of {SLOW_CHECK} does not say slow: {line!r}"


# ================================================================ US-077

@check("US-077 OUT-02 DM-05 DM-09 RES-01 p2 ended by its id during the third of five repeats on a dedicated session: the third is invalid machine-lost, the fourth and fifth not-run session-broken, the summary pass with 2 pass, 1 invalid and 2 not run, and the run exits 2", "start", repeat=True)
def us077_third(ctx):
    d = once(ctx, "ded", ded_part)
    assert d["done_at_kill"] == 2, f"p2 was ended with {d['done_at_kill']} repeats done"
    a, r = d["run"]["answer"], d["run"]["result"]
    valid(ctx, r, "US-077 1")
    got = [(c["repeat"], c["outcome"], (c["reason"] or {}).get("kind")) for c in r["checks"]]
    assert got == [(1, "pass", None), (2, "pass", None), (3, "invalid", "machine-lost")], f"executions {got}"
    assert (r["checks"][2]["reason"] or {}).get("machine") == "p2", f"the third's reason {r['checks'][2]['reason']}"
    nr = sorted((n["check_id"], n["repeat"], n["reason"]) for n in r["not_run"])
    assert nr == [(WAIT, 4, "session-broken"), (WAIT, 5, "session-broken")], f"not_run {r['not_run']}"
    rep = repeats_of(r, WAIT)
    assert rep == {"check_id": WAIT, "outcome": "pass", "counts": {"pass": 2, "fail": 0, "invalid": 1, "not_run": 2}}, \
        f"repeats {rep}"
    assert r["summary"] == {"total": 5, "pass": 2, "fail": 0, "invalid": 1, "not_run": 2}, f"summary {r['summary']}"
    assert a.code == 2, f"{a}"
    assert d["run"]["out"].get("repeats") == [dict(rep, setup="ded")], f"run --json repeats {d['run']['out'].get('repeats')}"


@check("US-077 OUT-02 DM-09 STP-02 p2 ended by its id in the first of three repeats of run --setup ded: every repeat that ran is invalid, the summary is invalid, the run exits 2 and no game process is left", "start", repeat=True)
def us077_all_invalid(ctx):
    d = once(ctx, "ded2", ded2_part)
    assert d["done_at_kill"] == 0, f"p2 was ended with {d['done_at_kill']} repeats done"
    a, r = d["run"]["answer"], d["run"]["result"]
    valid(ctx, r, "US-077 2")
    got = [(c["repeat"], c["outcome"], (c["reason"] or {}).get("kind")) for c in r["checks"]]
    assert got == [(1, "invalid", "machine-lost")], f"executions {got}"
    rep = repeats_of(r, WAIT)
    assert rep == {"check_id": WAIT, "outcome": "invalid", "counts": {"pass": 0, "fail": 0, "invalid": 1, "not_run": 2}}, \
        f"repeats {rep}"
    assert a.code == 2, f"{a}"
    assert not d["left"] and not d["after"]["game"], f"game processes run after the command: {sorted(d['after']['game'])}"
    assert d["after"]["status"].get("session") is None, f"status shows a session {d['after']['status'].get('session')}"


# ================================================================ US-080

def bad_case(ctx, name, shown):
    b = once(ctx, "bad", bad_part)[name]
    for key in ("list", "run", "run_session"):
        a = b[key]
        text = a.stdout + a.stderr
        assert a.code == 3, f"slow_step = {shown}: {a} {text[:300]}"
        assert "slow_step" in text and re.search(rf"\bline {b['line']}\b", text), \
            f"slow_step = {shown}: `{' '.join(a.args)}` does not name slow_step and line {b['line']}: {text[:300]}"
    assert b["game_after"] == b["game_before"], f"game processes {sorted(b['game_before'])} became {sorted(b['game_after'])}"
    assert b["status"].get("session") is None, f"status shows a session {b['status'].get('session')}"
    ctx.log(f"US-080 slow_step = {shown}: {(b['list'].stdout + b['list'].stderr).strip()[:300]}")


@check("US-080 TIM-01 CFG-01 slow_step = -1 makes list and run exit 3 naming slow_step and its line, and starts nothing", "start", repeat=True)
def us080_minus(ctx):
    bad_case(ctx, "minus", "-1")


@check("US-080 TIM-01 CFG-01 slow_step = \"10\" makes list and run exit 3 naming slow_step and its line, and starts nothing", "start", repeat=True)
def us080_string(ctx):
    bad_case(ctx, "string", '"10"')
