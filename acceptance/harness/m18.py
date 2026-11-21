r"""The checks of M18 The full health watch: US-093 and US-094's acceptance criteria.

Once per repeat, with no session running, in the project `m18` of its own in the work folder, whose `tests\` holds
the checks of `acceptance/m18/` and whose setup `ded` is a dedicated server with two clients loading `@skd_fixture`
alone. One `ded` session: `status --json`, `status` and the `status` tool once every machine has a frame rate and an
answer time; then `run US-094-starve US-094-after --json` in the background while `status --json` is read twice a
second in a thread of its own, and, as soon as the starved check runs, p2's process, by the id `status` gives,
suspended 480 ms of every 500 ms for 12 s, then resumed. `status` is read as text once while p2 is flagged. The
session is stopped by `session stop`, and every game process it started by its id if it outlives the stop. A part
lost to `Error 20 reading file` of a base-game file starts again, at most three times in all.
"""

import re
import shutil
import threading
import time
import traceback
from datetime import datetime
from pathlib import Path

from . import driver, schema, win
from .core import check, execution
from .m08 import game_folder
from .m11 import result_of, status_of
from .m13 import mcp_json, need_quiet, tidy
from .m17 import machine_fault
from .mcp import Mcp, structured

ACC = Path(__file__).resolve().parent.parent
STARVE, AFTER = "US-094-starve", "US-094-after"
NAMES = ["server", "p1", "p2"]
HEALTH = re.compile(r"health: (\d+(?:\.\d+)?) frames a second, answers in (\d+(?:\.\d+)?) s, "
                    r"mission time (\d+(?:\.\d+)?) s, frame (\d+)(, flagged (\S+) since (\S+))?")
STARVE_S, OFF_S, ON_S = 12.0, 0.48, 0.02


def keep(ctx):
    if not hasattr(ctx, "m18_keep"):
        ctx.m18_keep = {}
    return ctx.m18_keep


def project(ctx):
    store = keep(ctx)
    if "project" not in store:
        game = game_folder(ctx.project)
        root = ctx.work / "m18"
        if root.exists():
            shutil.rmtree(root)
        shutil.copytree(ACC / "m18", root / "tests")
        mcp_json(root)
        (root / "shakedown.toml").write_text(
            f"game = '{game}'\n\n[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n\n"
            "[setups.ded]\nmode = \"dedicated\"\nclients = 2\nmods = [\"fixture\"]\n", encoding="utf-8")
        store["project"] = (root, driver.Shakedown(ctx.sh.exe, root, ctx.log))
    return store["project"]


def machines(st):
    return {m["name"]: m for m in (st.get("session") or {}).get("machines", [])}


class Watch:
    """Reads `status --json` twice a second, at most `ceiling` s, keeping (wall time, p2's health, current check);
    reads `status` as text once, the first time p2 is flagged."""

    def __init__(self, sh, ceiling=400):
        self.sh = driver.Shakedown(sh.exe, sh.project, lambda text: None)
        self.seen, self.text, self.ceiling = [], None, ceiling
        self.done = threading.Event()
        self.thread = threading.Thread(target=self._loop, daemon=True)
        self.thread.start()

    def _loop(self):
        for _ in range(self.ceiling * 2):
            try:
                st = status_of(self.sh)
                h = machines(st).get("p2", {}).get("health")
                self.seen.append((time.time(), h, (st.get("run") or {}).get("current_check")))
                if self.text is None and h and h.get("flag"):
                    self.text = self.sh("status", ceiling=30).stdout
            except Exception:  # noqa: BLE001 - one missed read is not a verdict
                pass
            if self.done.wait(0.5):
                return

    def check_running(self, cid, alive, ceiling=120):
        for _ in range(ceiling * 10):
            if any(c == cid for _, _, c in self.seen):
                return True
            if not alive():
                return False
            time.sleep(0.1)
        return False

    def stop(self):
        self.done.set()
        self.thread.join(timeout=60)


def part(ctx):
    root, sh = project(ctx)
    need_quiet(sh)
    pids, out, suspended = set(), {}, []
    try:
        answer = sh("session", "start", "--setup", "ded", "--json", ceiling=400)
        session = answer.json() if answer.stdout.strip() else {}
        pids |= {m["pid"] for m in session.get("machines", []) if m.get("pid")}
        assert answer.code == 0 and session.get("state") == "ready", \
            f"{answer}: state {session.get('state')}, reason {str(session.get('reason'))[:600]}"
        st = None
        for _ in range(60):
            st = status_of(sh)
            ms = machines(st)
            if all(ms.get(n, {}).get("health", {}).get("frame_rate") is not None
                   and ms.get(n, {}).get("health", {}).get("answer_s") is not None for n in NAMES):
                break
            time.sleep(0.5)
        out["status"] = st
        out["text"] = sh("status", ceiling=30).stdout
        with Mcp(root, ctx.install, ctx.log) as m:
            tools = {t["name"]: t for t in m.tools()}
            out["tool_schema"] = tools.get("status", {}).get("outputSchema")
            out["tool"] = m.call("status", {}, ceiling=70)[0]
        p2 = machines(st)["p2"]["pid"]
        watch = Watch(sh)
        bg = sh.start("run", STARVE, AFTER, "--json")
        try:
            assert watch.check_running(STARVE, lambda: bg.poll() is None), \
                f"status never showed {STARVE} running; the run exited {bg.poll()}"
            out["starve_from"] = time.time()
            for _ in range(int(STARVE_S / (OFF_S + ON_S))):
                win.suspend(p2)
                suspended.append(p2)
                time.sleep(OFF_S)
                win.resume(p2)
                suspended.remove(p2)
                time.sleep(ON_S)
            out["starve_to"] = time.time()
            stdout, stderr = bg.communicate(timeout=300)
        finally:
            for pid in suspended:
                win.resume(pid)
            suspended.clear()
            watch.stop()
        out["seen"], out["flag_text"] = watch.seen, watch.text
        out["run"] = result_of(driver.Answer(["run"], bg.returncode, stdout, stderr, 0))
        ctx.log(f"M18: the run exited {bg.returncode}; p2 starved from {out['starve_from']:.1f} to {out['starve_to']:.1f}")
    finally:
        for pid in suspended:
            win.resume(pid)
        tidy(ctx, sh, pids)
    return out


def once(ctx):
    store = keep(ctx)
    k = getattr(ctx, "repeat_n", 1)
    if k not in store:
        root, _ = project(ctx)
        for attempt in range(1, 4):
            began = time.time()
            try:
                store[k] = part(ctx)
                break
            except Exception as e:  # noqa: BLE001
                ctx.log(f"M18 part: {traceback.format_exc()}")
                store[k] = e
                fault = machine_fault(root, began)
                if not fault or attempt == 3:
                    break
                ctx.log(f"M18: a game process could not read a base-game file ({fault}); starting again, "
                        f"attempt {attempt + 1} of 3")
    value = store[k]
    if isinstance(value, Exception):
        raise AssertionError(f"the M18 part failed: {value!r}")
    return value


def utc(text):
    return datetime.fromisoformat(text.replace("Z", "+00:00")).timestamp()


def health_ok(name, h):
    assert h, f"{name} has no health"
    assert {"flag", "since", "frame_rate", "answer_s", "heartbeat"} <= set(h), f"{name}'s health keys {sorted(h)}"
    fr, a = h["frame_rate"], h["answer_s"]
    assert isinstance(fr, (int, float)) and fr > 0, f"{name}'s frame_rate {fr!r}"
    assert isinstance(a, (int, float)) and a >= 0, f"{name}'s answer_s {a!r}"
    assert {"utc", "tick", "time", "frame"} <= set(h["heartbeat"] or {}), f"{name}'s heartbeat {h['heartbeat']}"


def m18check(name):
    return check(name, "start", repeat=True)


@m18check("US-093 HLT-01 INS-04 status --json lists server, p1 and p2 of a dedicated session with two clients each with health.frame_rate, health.answer_s and the clocks of its last heartbeat, and status prints the health line of cli.md under each")
def us093_status(ctx):
    v = once(ctx)
    ms = machines(v["status"])
    assert sorted(ms) == sorted(NAMES), f"status lists {sorted(ms)}"
    for n in NAMES:
        health_ok(n, ms[n]["health"])
    ctx.log("M18 US-093 1: " + "; ".join(f"{n} {ms[n]['health']['frame_rate']} fps, {ms[n]['health']['answer_s']} s"
                                         for n in NAMES))
    lines = [ln.strip() for ln in v["text"].splitlines() if "health:" in ln]
    good = [ln for ln in lines if HEALTH.search(ln)]
    assert len(good) == 3, f"status prints {len(good)} health lines of the documented form, expected 3: {v['text']!r}"


@m18check("US-093 AGT-02 INS-04 the status tool gives the same health fields, frame_rate and answer_s for every machine, valid against its outputSchema")
def us093_tool(ctx):
    v = once(ctx)
    sc = structured(v["tool"])
    assert v["tool_schema"], "the status tool declares no outputSchema"
    errs = schema.validate(sc, v["tool_schema"])
    assert not errs, f"the status tool's structuredContent is not valid against its outputSchema: {errs[:3]}"
    ms, cli = machines(sc), machines(v["status"])
    assert sorted(ms) == sorted(cli), f"the tool lists {sorted(ms)}, the command {sorted(cli)}"
    for n in NAMES:
        health_ok(n, ms[n]["health"])
        assert sorted(ms[n]["health"]) == sorted(cli[n]["health"]), \
            f"{n}'s health keys differ: tool {sorted(ms[n]['health'])}, command {sorted(cli[n]['health'])}"


@m18check("US-094 HLT-01 DM-05 p2 starved of CPU, suspended 480 ms of every 500 ms by its id, during a check that would pass ends it invalid unhealthy naming p2 and its frame rate, while status shows p2 flagged frame-rate with a low frame rate, also as text")
def us094_invalid(ctx):
    v = once(ctx)
    e = execution(v["run"]["result"], STARVE)
    reason = e.get("reason") or {}
    k, msg = (e["outcome"], reason.get("kind")), reason.get("message") or ""
    ctx.log(f"M18 US-094 1: {STARVE} ended {k}: {msg}")
    assert k == ("invalid", "unhealthy"), f"{STARVE} ended {k}: {msg}"
    assert "p2" in msg or reason.get("machine") == "p2", f"the reason names no p2: {reason}"
    assert re.search(r"frame.rate|frames a second", msg, re.I), f"the message does not name the frame rate: {msg!r}"
    flagged = [(t, h) for t, h, _ in v["seen"] if h and h.get("flag")]
    assert flagged, "status never showed p2 flagged"
    flags = {h["flag"] for _, h in flagged}
    assert flags == {"frame-rate"}, f"the flags status showed for p2: {flags}"
    low = min(h["frame_rate"] for _, h in flagged if h.get("frame_rate") is not None)
    since = utc(flagged[0][1]["since"])
    ctx.log(f"M18 US-094 1: flagged {since - v['starve_from']:.1f} s after the starving began, lowest frame rate {low}")
    assert low < 10, f"p2's lowest frame rate while flagged was {low}"
    assert v["flag_text"] and "flagged frame-rate since" in v["flag_text"], \
        f"status as text while p2 was flagged: {v['flag_text']!r}"


@m18check("US-094 HLT-01 the flag on p2 clears 5 to 12 s after the starving ends, and the next check passes on p2")
def us094_clears(ctx):
    v = once(ctx)
    seen = v["seen"]
    last = max((t for t, h, _ in seen if h and h.get("flag")), default=None)
    assert last is not None, "status never showed p2 flagged"
    clear = [t for t, h, _ in seen if t > last and h and h.get("flag") is None]
    assert clear, "status never showed p2's flag cleared"
    took = clear[0] - v["starve_to"]
    ctx.log(f"M18 US-094 2: the flag was first seen clear {took:.1f} s after the starving ended")
    assert 4.5 <= took <= 12, f"the flag cleared {took:.1f} s after the starving ended; HLT-01 gives a whole 5 s"
    e = execution(v["run"]["result"], AFTER)
    assert e["outcome"] == "pass", f"{AFTER} ended {e['outcome']}: {e.get('reason')}"
