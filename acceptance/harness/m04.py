"""The acceptance checks of M4 Agent tools: US-001, US-003, US-005, US-006, US-013, US-014, US-023,
US-031, US-032, US-042, US-046, US-047, US-048, US-049, US-050, US-051, US-061 and US-062, and exit
criterion 3 (NFR-PF-03).

The tools are driven over their protocol: harness/mcp.py starts the server the generated project's
.mcp.json declares, as Claude Code does, and speaks JSON-RPC to it on stdio. Each batch does its
calls and keeps what came back in ctx.m04; the checks judge that.
"""

import json
import math
import re
import shutil
import time
import traceback
from pathlib import Path

from . import driver, schema
from .core import batch, check, execution
from .mcp import Mcp, structured, text_of, tool_error

ON_ARGS = "us013/US-013-on-args"
ON_EACH = "us013/US-013-on-each"
P3 = "us014/US-014-p3"
COMPILE = "us014/US-014-compile"
WAIT = "us023/US-023-wait"
TRY = "us023/US-023-try"
LOGS = "us042/US-042-logs"
OTHER = "us042/US-042-other"
THROW = "us061/US-061-throw"
ERR = "us062/US-062-error"
STRING = "us062/US-062-string"
TRY_ERR = "us062/US-062-try-error"
TRY_STRING = "us062/US-062-try-string"
FAILS = "us048/US-048-fails"
SLOW = "us050/US-050-slow"
SEL_A, SEL_B, SEL_C = "sel/RUN-01-a", "sel/RUN-01-b", "sel/RUN-01-c"

DED_RUN = [ON_ARGS, ON_EACH, P3, COMPILE, WAIT, TRY, LOGS, OTHER, THROW, ERR, STRING, TRY_ERR, TRY_STRING]
SETUP_NAMES = ["sp", "cba", "ded", "ded30", "slow"]
M4_TOOLS = ["status", "session_start", "session_stop", "list_checks", "run_start", "run_status", "run_result",
            "reset", "exec", "wait_until", "expect", "inspect", "logs", "errors", "guide"]
READ_ONLY = ["status", "list_checks", "run_status", "run_result", "inspect", "logs", "errors", "guide"]
STEAM_ID = re.compile(r"(?<!\d)7656119\d{10}(?!\d)")

# Keys whose values are clocks or durations: two answers to the same request at two moments differ there.
VOLATILE = {"duration_s", "waited_s", "last_answer_at", "utc", "tick", "time", "frame", "fps", "heartbeat", "since"}


def mask(value):
    if isinstance(value, dict):
        return {k: ("<volatile>" if k in VOLATILE else mask(v)) for k, v in value.items()}
    if isinstance(value, list):
        return [mask(v) for v in value]
    return value


def first_difference(a, b, path="$"):
    """The path of the first place two JSON values differ, or None."""
    if type(a) is not type(b):
        return f"{path}: {str(a)[:120]} vs {str(b)[:120]}"
    if isinstance(a, dict):
        if set(a) != set(b):
            return f"{path}: keys {sorted(set(a) ^ set(b))} are in only one"
        for k in a:
            d = first_difference(a[k], b[k], f"{path}.{k}")
            if d:
                return d
        return None
    if isinstance(a, list):
        if len(a) != len(b):
            return f"{path}: {len(a)} items vs {len(b)}"
        for i, (x, y) in enumerate(zip(a, b)):
            d = first_difference(x, y, f"{path}[{i}]")
            if d:
                return d
        return None
    return None if a == b else f"{path}: {str(a)[:120]} vs {str(b)[:120]}"


def same_json(cli, tool, what):
    d = first_difference(mask(cli), mask(tool))
    assert d is None, f"{what}: the command's --json and the tool's structured content differ at {d}"


def whole(sc):
    """The structured content, or, for a result over 20,000 characters, what its result_path file holds (AGT-07)."""
    if set(sc) == {"result_path"}:
        return json.loads(Path(sc["result_path"]).read_text(encoding="utf-8"))
    return sc


def p95(values):
    s = sorted(values)
    return s[max(0, math.ceil(0.95 * len(s)) - 1)]


def message(answer):
    """What a refused command said: its stderr, else its stdout."""
    return (answer.stderr.strip() or answer.stdout.strip())


class Calls:
    """One MCP server for a batch, keeping every call it made for the AGT-02 check."""

    def __init__(self, ctx):
        self.ctx = ctx
        self.m = Mcp(ctx.project, ctx.install, ctx.log)
        self.m.initialize()
        self.tools = {t["name"]: t for t in self.m.tools()}
        ctx.m04.setdefault("calls", [])

    def __call__(self, name, args=None, token=None, ceiling=70):
        result, seconds = self.m.call(name, args or {}, token, ceiling)
        self.ctx.m04["calls"].append((name, args or {}, result, self.tools.get(name, {}).get("outputSchema")))
        return result, seconds

    def ok(self, name, args=None, token=None, ceiling=70):
        result, seconds = self(name, args, token, ceiling)
        return structured(result), seconds

    def close(self):
        self.m.close()


def step(ctx, key, fn):
    """Runs one part of a batch; keeps its value, or the error it raised, under key."""
    try:
        ctx.m04[key] = fn()
    except Exception as e:  # the check that reads key reports it
        ctx.log(f"batch step {key}: {traceback.format_exc()}")
        ctx.m04[key] = e


def got(ctx, key, store=None):
    store = ctx.m04 if store is None else store
    assert key in store, f"the batch step {key} did not run this repeat; see harness.log"
    value = store[key]
    if isinstance(value, Exception):
        raise AssertionError(f"the batch step {key} failed: {value!r}")
    return value


def no_runner_running():
    pid = driver.runner_pid()
    assert not (pid and driver.alive(pid)), \
        f"a runner ({pid}) runs; this criterion needs none. Wait for it to exit (60 s after its last session)."


def shakedown_pids():
    return driver.pids("shakedown.exe")


# ================================================================ offline: no runner, no game

@check("US-046 INS-02 with no runner shakedown status exits 0 saying no runner runs, and starts none", "offline")
def us046_no_runner(ctx):
    no_runner_running()
    before = shakedown_pids()
    text = ctx.sh("status")
    assert text.code == 0, f"{text}, expected exit 0: {message(text)}"
    said = (text.stdout + text.stderr).lower()
    assert "runner" in said and re.search(r"\bno\b|\bnot\b", said), f"status does not say no runner runs: {said[:300]!r}"
    js = ctx.sh("status", "--json")
    assert js.code == 0, f"{js}"
    out = js.json()
    assert out["runner"] == "not-running" and out["runner_pid"] is None and out["session"] is None and out["run"] is None, \
        f"status --json with no runner gave {out}"
    new = shakedown_pids() - before
    assert not new, f"shakedown.exe processes {sorted(new)} run after status"
    assert driver.runner_pid() is None, "runner.json exists after status with no runner"


@check("US-048 AGT-01 AGT-02 INT-CC-01 the server .mcp.json declares lists every tool of M4 with an inputSchema, an outputSchema and its hints", "offline")
def us048_tools(ctx):
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        assert m.info.get("serverInfo", {}).get("name") == "shakedown", f"the server names itself {m.info.get('serverInfo')}"
        assert "tools" in m.info.get("capabilities", {}), f"the server declares no tools capability: {m.info}"
        tools = {t["name"]: t for t in m.tools()}
    missing = [n for n in M4_TOOLS if n not in tools]
    assert not missing, f"tools/list lacks {missing}; it lists {sorted(tools)}"
    for name, t in tools.items():
        for key in ("inputSchema", "outputSchema"):
            assert isinstance(t.get(key), dict) and t[key].get("type") == "object", f"{name} has no object {key}: {t.get(key)}"
    wrong = [n for n in READ_ONLY if (tools[n].get("annotations") or {}).get("readOnlyHint") is not True]
    assert not wrong, f"these tools are not marked readOnlyHint true: {wrong}"
    assert (tools["exec"].get("annotations") or {}).get("destructiveHint") is True, \
        f"exec is not marked destructiveHint true: {tools['exec'].get('annotations')}"


@check("US-046 AGT-05 shakedown mcp answers status runner not-running, list_checks and guide with no runner, and starts none (on Windows)", "offline")
def us046_mcp_no_runner(ctx):
    no_runner_running()
    before = shakedown_pids()
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        server = m.pid
        st = structured(m.call("status")[0])
        assert st["runner"] == "not-running" and st["runner_pid"] is None and st["session"] is None, f"status gave {st}"
        checks = whole(structured(m.call("list_checks")[0]))["checks"]
        assert len(checks) == len(ctx.project_ids()), f"list_checks gave {len(checks)} checks, the project has {len(ctx.project_ids())}"
        guide = structured(m.call("guide")[0])["text"]
        assert len(guide) > 500, f"guide gave {len(guide)} characters"
        new = shakedown_pids() - before - {server}
    assert not new, f"shakedown.exe processes {sorted(new)} besides the MCP server ran during the calls"
    assert driver.runner_pid() is None, "runner.json exists after the server's status, list_checks and guide"


@check("US-048 NFR-US-03 status and list_checks give the same JSON as shakedown status --json and shakedown list --json", "offline")
def us048_same_offline(ctx):
    no_runner_running()
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        pairs = [(("status", "--json"), "status", {}),
                 (("list", "--json"), "list_checks", {}),
                 (("list", "--tag", "acc-sel-x", "--json"), "list_checks", {"tags": ["acc-sel-x"]}),
                 (("list", SEL_B, SEL_A, "--json"), "list_checks", {"ids": [SEL_B, SEL_A]})]
        for argv, tool, args in pairs:
            cli = ctx.sh(*argv)
            assert cli.code == 0, f"{cli}: {message(cli)}"
            same_json(cli.json(), whole(structured(m.call(tool, args)[0])), f"{tool} {args}")


@check("US-031 RUN-02 list_checks with tags acc-sel-x selects the checks shakedown list --tag acc-sel-x prints", "offline")
def us031_list_checks(ctx):
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        tool = [c["id"] for c in structured(m.call("list_checks", {"tags": ["acc-sel-x"]})[0])["checks"]]
    cli = [c["id"] for c in ctx.sh("list", "--tag", "acc-sel-x", "--json").json()["checks"]]
    assert tool == cli == [SEL_A, SEL_C], f"list_checks selected {tool}, list --tag {cli}, expected {[SEL_A, SEL_C]}"


@check("US-032 AGT-02 list_checks with tags US-999 returns isError unknown-check naming US-999 and the tags that exist", "offline")
def us032_list_checks(ctx):
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        text = tool_error(m.call("list_checks", {"tags": ["US-999"]})[0], "unknown-check")
    assert "US-999" in text and "acc-sel-x" in text and "US-061" in text, f"the error does not name US-999 and the tags: {text[:300]}"


@check("US-048 AGT-06 guide returns under 8,000 characters on starting a session, running checks, reading a result and writing a check, as shakedown guide does", "offline")
def us048_guide(ctx):
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        text = structured(m.call("guide")[0])["text"]
    assert len(text) < 8000, f"guide is {len(text)} characters"
    for part in ("session_start", "run_start", "run_status", "run_result", "@check", "skd_fnc_"):
        assert part in text, f"guide does not mention {part}"
    cli = ctx.sh("guide", "--json")
    assert cli.code == 0 and cli.json()["text"] == text, f"{cli}: shakedown guide --json differs from the guide tool"


@check("AGT-06 the guide tool returns the text docs/agent-tools.md gives under The guide", "offline")
def agt06_guide_doc(ctx):
    doc = (ctx.repo / "docs" / "agent-tools.md").read_text(encoding="utf-8")
    block = re.search(r"## The guide.*?```text\n(.*?)```", doc, re.S)
    assert block, "docs/agent-tools.md has no text block under The guide"
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        text = structured(m.call("guide")[0])["text"]
    norm = lambda s: re.sub(r"\s+", " ", s).strip()
    assert norm(text) == norm(block.group(1)), \
        f"the guide differs from the documented text first at character {next((i for i, (a, b) in enumerate(zip(norm(text), norm(block.group(1)))) if a != b), None)}"


# ================================================================ start: no session; each check stops what it starts

def no_session(ctx):
    """Clears a failed start that an earlier start check left in status, so that no session runs."""
    st = ctx.sh("status", "--json").json()
    if st.get("session"):
        assert st["session"]["state"] in ("failed", "stopped"), f"a session runs before the check: {st['session']['state']}"
        ctx.sh("session", "stop", "--json", ceiling=180)


@check("US-047 SES-01 with no session exec, inspect, reset and run without --setup exit 4 naming shakedown session start --setup and the setups", "start")
def us047_cli(ctx):
    no_session(ctx)
    before = driver.game_pids()
    for argv in (("exec", "--machine", "p1", "--code", "1"), ("inspect", "--machine", "p1"), ("reset",), ("run", SEL_A)):
        a = ctx.sh(*argv)
        said = message(a)
        assert a.code == 4, f"{a}, expected exit 4: {said[:300]}"
        assert re.search(r"no session", said, re.I), f"`{argv[0]}` does not say no session runs: {said[:300]}"
        assert "shakedown session start --setup" in said, f"`{argv[0]}` does not name shakedown session start --setup: {said[:300]}"
        missing = [s for s in SETUP_NAMES if not re.search(rf"\b{s}\b", said)]
        assert not missing, f"`{argv[0]}` does not name the setups {missing}: {said[:300]}"
    assert not driver.game_pids() - before, "a game process started"


@check("US-047 AGT-02 with no session the tools exec, inspect, reset and run_start without setup return isError no-session with the same next step", "start")
def us047_mcp(ctx):
    no_session(ctx)
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        for tool, args in (("exec", {"machine": "p1", "code": "1"}), ("inspect", {"machine": "p1"}), ("reset", {}),
                           ("run_start", {"ids": [SEL_A]})):
            text = tool_error(m.call(tool, args)[0], "no-session")
            assert re.search(r"session_start|shakedown session start --setup", text), f"{tool} does not name the start: {text}"
            missing = [s for s in SETUP_NAMES if not re.search(rf"\b{s}\b", text)]
            assert not missing, f"{tool} does not name the setups {missing}: {text}"


def side_project(ctx, name, toml):
    """A project beside the generated one, holding one check, the toml text and the .mcp.json."""
    root = ctx.work / name
    if root.exists():
        shutil.rmtree(root)
    (root / "checks" / "sel").mkdir(parents=True)
    shutil.copy(ctx.project / "checks" / "sel" / "RUN-01-a.sqf", root / "checks" / "sel" / "RUN-01-a.sqf")
    (root / "shakedown.toml").write_text(toml, encoding="utf-8")
    shutil.copy(ctx.project / ".mcp.json", root / ".mcp.json")
    return root


def game_line(ctx):
    return (ctx.project / "shakedown.toml").read_text(encoding="utf-8").splitlines()[0]


SP_SETUP = "[setups.sp]\nmode = \"singleplayer\"\nmods = []\n"


def bad_projects(ctx):
    """(name, toml, key, line or None) for each malformed project file of US-006."""
    game = game_line(ctx)
    return [
        ("unknown key", f"{game}\ncolour = \"red\"\n\n{SP_SETUP}", "colour", 2),
        ("missing game", f"checks = \"checks\"\n\n{SP_SETUP}", "game", None),
        ("clients two", f"{game}\n\n[setups.ded]\nmode = \"dedicated\"\nclients = \"two\"\nmods = []\n", "clients", 5),
    ]


@check("US-006 CFG-01 a project file with an unknown key, no game or clients = \"two\" makes session start, run and list exit 3 naming the key and its line, and the tools return config-error", "start")
def us006_bad_file(ctx):
    before = driver.game_pids()
    for i, (what, toml, key, line) in enumerate(bad_projects(ctx)):
        root = side_project(ctx, f"us006-{i}", toml)
        sh = driver.Shakedown(ctx.sh.exe, root, ctx.log)
        setup = "ded" if key == "clients" else "sp"
        for argv in (("session", "start", "--setup", setup), ("run", SEL_A), ("list",)):
            a = sh(*argv, ceiling=120)
            said = message(a)
            assert a.code == 3, f"{what}: {a}, expected exit 3: {said[:300]}"
            assert re.search(rf"\b{key}\b", said), f"{what}: `{argv[0]}` does not name the key {key}: {said[:300]}"
            assert "line" in said and (line is None or re.search(rf"\b{line}\b", said)), \
                f"{what}: `{argv[0]}` does not name the line {line}: {said[:300]}"
        with Mcp(root, ctx.install, ctx.log) as m:
            for tool, args in (("session_start", {"setup": setup}), ("run_start", {"ids": [SEL_A]}), ("list_checks", {})):
                text = tool_error(m.call(tool, args)[0], "config-error")
                assert re.search(rf"\b{key}\b", text), f"{what}: {tool} does not name the key {key}: {text}"
    assert not driver.game_pids() - before, "a game process started"


@check("US-006 session start --setup nosuch exits 3 naming the setups the file declares, and session_start returns unknown-setup", "start")
def us006_unknown_setup(ctx):
    no_session(ctx)
    before = driver.game_pids()
    a = ctx.sh("session", "start", "--setup", "nosuch", ceiling=120)
    said = message(a)
    assert a.code == 3, f"{a}, expected exit 3: {said[:300]}"
    missing = [s for s in SETUP_NAMES if not re.search(rf"\b{s}\b", said)]
    assert "nosuch" in said and not missing, f"the message does not name nosuch and the setups {missing}: {said[:300]}"
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        text = tool_error(m.call("session_start", {"setup": "nosuch"})[0], "unknown-setup")
    missing = [s for s in SETUP_NAMES if not re.search(rf"\b{s}\b", text)]
    assert not missing, f"session_start does not name the setups {missing}: {text}"
    assert not driver.game_pids() - before, "a game process started"


@check("US-003 SES-03 a setup whose mod folder does not exist is refused with exit 4 naming the mod and the folder, session_start returns precondition-failed with the same text, and status shows no session", "start")
def us003_missing_mod(ctx):
    no_session(ctx)
    before = driver.game_pids()
    folder = ctx.work / "us003-no-such-mod"
    toml = f"{game_line(ctx)}\n\n[mods.gone]\npath = '{folder}'\n\n[setups.sp]\nmode = \"singleplayer\"\nmods = [\"gone\"]\n"
    root = side_project(ctx, "us003", toml)
    sh = driver.Shakedown(ctx.sh.exe, root, ctx.log)
    a = sh("session", "start", "--setup", "sp", ceiling=120)
    said = message(a)
    assert a.code == 4, f"{a}, expected exit 4: {said[:300]}"
    assert re.search(r"\bgone\b", said) and str(folder) in said, f"the message does not name the mod gone and {folder}: {said[:400]}"
    st = sh("status", "--json").json()
    assert st["session"] is None, f"status after the refusal shows a session: {st['session']}"
    with Mcp(root, ctx.install, ctx.log) as m:
        text = tool_error(m.call("session_start", {"setup": "sp"})[0], "precondition-failed")
        st2 = structured(m.call("status")[0])
    assert text[len("precondition-failed: "):].strip() == said.strip(), \
        f"session_start's text is not the command's message: {text!r} vs {said!r}"
    assert st2["session"] is None, f"status after the tool's refusal shows a session: {st2['session']}"
    assert not driver.game_pids() - before, "a game process started"


@check("US-003 SES-03 INT-STM-01 with no steam process session start exits 4 saying Steam is not running and must be started and logged in, session_start returns precondition-failed with the same text, and status shows no session", "nosteam")
def us003_no_steam(ctx):
    steam = driver.pids("steam.exe")
    assert not steam, (f"{len(steam)} steam process(es) run; this check needs none. Stop Steam first; "
                       "a full run leaves this check out")
    no_session(ctx)
    before = driver.game_pids()
    a = ctx.sh("session", "start", "--setup", "sp", ceiling=120)
    ctx.runner_pid = driver.runner_pid()  # the suite waits for it to exit, whether the check passes or not
    said = message(a)
    assert a.code == 4, f"{a}, expected exit 4: {said[:300]}"
    # "Steam is not running" said either way: "not running", or a running Steam client named as missing.
    not_running = (re.search(r"\bSteam\b.*\bnot running\b", said, re.I | re.S)
                   or re.search(r"\bmissing\b.*\brunning Steam\b", said, re.I | re.S))
    assert not_running, f"the message does not say Steam is not running: {said[:400]}"
    assert re.search(r"\bstart Steam\b|\bSteam must be started\b", said, re.I) and re.search(r"\blog(ged)? ?in\b", said, re.I), \
        f"the message does not say Steam must be started and logged in: {said[:400]}"
    st = ctx.sh("status", "--json").json()
    assert st["session"] is None, f"status after the refusal shows a session: {st['session']}"
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        text = tool_error(m.call("session_start", {"setup": "sp"})[0], "precondition-failed")
        st2 = structured(m.call("status")[0])
    assert text[len("precondition-failed: "):].strip() == said.strip(), \
        f"session_start's text is not the command's message: {text!r} vs {said!r}"
    assert st2["session"] is None, f"status after the tool's refusal shows a session: {st2['session']}"
    started = driver.game_pids() - before
    assert not started, f"{len(started)} game process(es) started"


def free_mb():
    import subprocess
    out = subprocess.run(["powershell", "-NoProfile", "-Command", "(Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory"],
                         capture_output=True, text=True, timeout=60).stdout.strip()
    return int(out) // 1024 if out.isdigit() else None


def stop_started(ctx, before):
    """Stops by id every game process started since `before` that outlives 30 s."""
    for pid in driver.game_pids() - before:
        if driver.wait_gone(pid, 30) is None:
            ctx.log(f"game process {pid} still runs; stopping it by id")
            driver.kill(pid)


@check("US-001 SES-01 session_start with setup sp returns a ready session, and session_stop stops it", "start")
def us001_mcp_start(ctx):
    no_session(ctx)
    before = driver.game_pids()
    try:
        with Mcp(ctx.project, ctx.install, ctx.log) as m:
            r, seconds = m.call("session_start", {"setup": "sp"})
            s = structured(r)
            ctx.m04_keep["mcp-sp"] = s
            ctx.log(f"session_start sp answered {s['state']} in {seconds:.1f} s")
            stopped = structured(m.call("session_stop")[0])
    finally:
        stop_started(ctx, before)
    assert s["state"] == "ready", f"session_start sp gave state {s['state']} after {seconds:.1f} s: {s.get('reason')}"
    assert stopped["state"] == "stopped", f"session_stop gave state {stopped['state']}"


@check("US-050 AGT-03 INS-02 session_start of a start over 50 s returns state starting within 60 s with progress every 5 s, and status later shows it ready", "start")
def us050_slow_start(ctx):
    no_session(ctx)
    before = driver.game_pids()
    ctx.log(f"free memory before the slow dedicated session: {free_mb()} MB")
    seen, polls = None, []
    try:
        with Mcp(ctx.project, ctx.install, ctx.log) as m:
            rid, t0 = m.start_call("session_start", {"setup": "slow"}, token="slow-start")
            at, msg = m.wait(rid, 75)
            seconds = at - t0
            result = msg.get("result", {})
            s = structured(result)
            marks = [t for t, _ in m.progress("slow-start")]
            ctx.log(f"session_start slow answered {s['state']} in {seconds:.1f} s with {len(marks)} progress notifications")
            if s["state"] == "starting":
                for _ in range(150):  # every 2 s, at most 300 s
                    st = structured(m.call("status")[0])
                    seen = (st.get("session") or {}).get("state")
                    polls.append(seen)
                    if seen in ("ready", "failed"):
                        break
                    time.sleep(2)
            stopped = structured(m.call("session_stop", ceiling=200)[0])
    finally:
        stop_started(ctx, before)
        ctx.log(f"free memory after the slow dedicated session: {free_mb()} MB")
    assert seconds < 60, f"session_start answered after {seconds:.1f} s"
    assert s["state"] == "starting", \
        f"session_start answered {s['state']} after {seconds:.1f} s; the criterion needs a start over 50 s (slow setup on Altis)"
    assert seconds >= 49, f"session_start answered starting after only {seconds:.1f} s, before the 50 s it waits"
    assert seen == "ready", f"status showed the session {seen} after {len(polls)} polls, expected ready"
    gaps = [b - a for a, b in zip([t0] + marks, marks + [at])]
    assert marks and max(gaps) <= 5.5, \
        f"session_start sent {len(marks)} progress notifications; the longest gap was {max(gaps):.1f} s"
    assert stopped["state"] == "stopped", f"session_stop gave {stopped['state']}"


# ================================================================ the ded session

@batch("ded")
def ded_batch(ctx, checks):
    if not any(c.name in M4_NAMES for c in checks):
        return
    needed = {i for c in checks for i in c.needs}
    ids = [i for i in DED_RUN if i in needed]
    if ids:
        step(ctx, "ded-run", lambda: ctx.run(ids))
    calls = Calls(ctx)
    try:
        step(ctx, "exec-server", lambda: (ctx.sh("exec", "--machine", "server", "--code", "sleep 1; 1 + 1", "--json"),
                                          calls.ok("exec", {"machine": "server", "code": "sleep 1; 1 + 1"})))
        if any(c.name.startswith("NFR-PF-03") for c in checks):
            for machine in ("server", "p1", "hc1"):
                step(ctx, f"trips-{machine}", lambda machine=machine: trips(calls, machine))
        step(ctx, "exec-p3", lambda: (ctx.sh("exec", "--machine", "p3", "--code", "1"),
                                      calls("exec", {"machine": "p3", "code": "1"})[0]))
        step(ctx, "exec-compile", lambda: (ctx.sh("exec", "--machine", "p1", "--code", "1 +"),
                                           calls("exec", {"machine": "p1", "code": "1 +"})[0]))
        step(ctx, "wait-false", lambda: (ctx.sh("wait", "--machine", "server", "--condition", "false", "--ceiling", "2", "--json"),
                                         calls.ok("wait_until", {"machine": "server", "condition": "false", "ceiling": 2})))
        step(ctx, "wait-error", lambda: (ctx.sh("wait", "--machine", "p1", "--condition", "_skdfx_undefined > 1", "--json"),
                                         calls.ok("wait_until", {"machine": "p1", "condition": "_skdfx_undefined > 1"})))
        step(ctx, "exec-throw", lambda: (ctx.sh("exec", "--machine", "p1", "--code", "[] call skdfx_fnc_throw; 7", "--json"),
                                         calls("exec", {"machine": "p1", "code": "[] call skdfx_fnc_throw; 7"})[0]))
        step(ctx, "errors-p1", lambda: (ctx.sh("errors", "--machine", "p1", "--json"), ctx.sh("errors", "--machine", "p1")))
        if LOGS in ids and not isinstance(ctx.m04.get("ded-run"), Exception):
            step(ctx, "logs", lambda: logs_of(ctx, calls))
    finally:
        calls.close()


def trips(calls, machine):
    """100 exec calls of 1 + 1: (wall seconds of each call, each answer)."""
    out = []
    for _ in range(100):
        s, seconds = calls.ok("exec", {"machine": machine, "code": "1 + 1"})
        out.append((seconds, s))
    calls.ctx.m04["calls"] = calls.ctx.m04["calls"][:-100]  # one sample is enough for the AGT-02 check
    return out


def logs_of(ctx, calls):
    run_id = ctx.m04["ded-run"][1]["run_id"]
    out = {}
    for name, extra, args in (("all", [], {}), ("grep", ["--grep", "line 2"], {"grep": "line 2"}),
                              ("last", ["--last", "1"], {"last": 1})):
        cli = ctx.sh("logs", "--machine", "p2", "--check", LOGS, "--run", run_id, *extra, "--json")
        tool = calls.ok("logs", {"machine": "p2", "check_id": LOGS, "run_id": run_id, **args})[0]
        out[name] = (cli, tool)
    ctx.m04_keep.setdefault("logs-answers", []).extend(out.values())
    return out


M4_NAMES = set()


def m4check(name, phase, needs=(), setup="sp"):
    M4_NAMES.add(name)
    return check(name, phase, needs, setup)


def ded_ex(ctx, cid):
    return execution(got(ctx, "ded-run")[2], cid)


def a_message(ex):
    return (ex.get("reason") or {}).get("message", "")


@m4check("US-013 CTL-02 DM-13 DM-14 skd_fnc_on to p2 with args [42] returns 42 first, and p1's unit passed to p2 arrives as the same object", "session", [ON_ARGS], "ded")
def us013_on(ctx):
    ex = ded_ex(ctx, ON_ARGS)
    assert ex["outcome"] == "pass", f"{ON_ARGS} ended {ex['outcome']}: {ex['reason']}"


@m4check("US-013 CTL-01 skd_fnc_onEach on all returns a HashMap with one key per machine of the session", "session", [ON_EACH], "ded")
def us013_each(ctx):
    ex = ded_ex(ctx, ON_EACH)
    assert ex["outcome"] == "pass", f"{ON_EACH} ended {ex['outcome']}: {ex['reason']}"


@m4check("US-013 CTL-01 CTL-03 exec on server of sleep 1; 1 + 1 gives done, 2 and at least 1 s, and the exec tool returns the same JSON", "session", (), "ded")
def us013_exec(ctx):
    cli, (tool, _) = got(ctx, "exec-server")
    assert cli.code == 0, f"{cli}: {message(cli)}"
    out = cli.json()
    r = out["results"]["server"]
    assert r["state"] == "done" and r["value"] == 2 and r["duration_s"] >= 1, f"exec gave {r}"
    assert tool["results"]["server"]["duration_s"] >= 1, f"the tool gave {tool['results']['server']}"
    same_json(out, tool, "exec server")


def trip_check(ctx, key, kind):
    answers = got(ctx, key)
    bad = [s for _, s in answers if list(s["results"].values())[0]["value"] != 2 or list(s["results"].values())[0]["state"] != "done"]
    assert len(answers) == 100 and not bad, f"{len(answers)} trips to {kind}; {len(bad)} did not answer done 2: {bad[:2]}"
    wall = p95([t for t, _ in answers])
    inside = p95([list(s["results"].values())[0]["duration_s"] for _, s in answers])
    ctx.log(f"NFR-PF-03 {kind}: 95th percentile {wall * 1000:.0f} ms per exec call, {inside * 1000:.0f} ms duration_s")
    assert wall < 1, f"the 95th percentile of 100 exec calls to {kind} is {wall:.3f} s"


@m4check("NFR-PF-03 US-013 100 exec calls of 1 + 1 to the dedicated server answer 2 with a 95th percentile under 1 s", "session", (), "ded")
def nfr_pf_03_server(ctx):
    trip_check(ctx, "trips-server", "the dedicated server")


@m4check("NFR-PF-03 US-013 100 exec calls of 1 + 1 to a client answer 2 with a 95th percentile under 1 s", "session", (), "ded")
def nfr_pf_03_client(ctx):
    trip_check(ctx, "trips-p1", "client p1")


@m4check("NFR-PF-03 US-013 100 exec calls of 1 + 1 to a headless client answer 2 with a 95th percentile under 1 s", "session", (), "ded")
def nfr_pf_03_hc(ctx):
    trip_check(ctx, "trips-hc1", "headless client hc1")


@m4check("US-014 CTL-04 DM-04 skd_fnc_on to p3 in a session with two clients ends fail check-error naming p3 and the machines", "session", [P3], "ded")
def us014_p3(ctx):
    ex = ded_ex(ctx, P3)
    msg = a_message(ex)
    assert ex["outcome"] == "fail" and ex["reason"]["kind"] == "check-error", f"{P3} ended {ex['outcome']} / {ex['reason']}"
    missing = [n for n in ("p3", "server", "p1", "p2", "hc1") if not re.search(rf"\b{n}\b", msg)]
    assert not missing, f"the message does not name {missing}: {msg}"
    assert ["never runs"] not in [s["path"] for s in ex["steps"]], "a step after the failure ran"


@m4check("US-014 CTL-04 DM-04 code that does not compile on p1 ends fail check-error with the engine's compile message", "session", [COMPILE], "ded")
def us014_compile(ctx):
    ex = ded_ex(ctx, COMPILE)
    msg = a_message(ex)
    assert ex["outcome"] == "fail" and ex["reason"]["kind"] == "check-error", f"{COMPILE} ended {ex['outcome']} / {ex['reason']}"
    assert "Invalid number in expression" in msg, f"the message holds no engine compile message: {msg}"


@m4check("US-014 CTL-04 AGT-02 exec --machine p3 exits 3 naming the machines, the exec tool returns unknown-machine, and code that does not compile exits 3 and returns compile-error with the engine's message", "session", (), "ded")
def us014_exec(ctx):
    cli, tool = got(ctx, "exec-p3")
    said = message(cli)
    assert cli.code == 3, f"{cli}, expected exit 3: {said}"
    for text in (said, tool_error(tool, "unknown-machine")):
        missing = [n for n in ("p3", "server", "p1", "p2", "hc1") if not re.search(rf"\b{n}\b", text)]
        assert not missing, f"the message does not name {missing}: {text}"
    cli, tool = got(ctx, "exec-compile")
    assert cli.code == 3 and "Invalid number in expression" in message(cli), f"{cli}: {message(cli)}"
    text = tool_error(tool, "compile-error")
    assert "Invalid number in expression" in text, f"compile-error does not give the engine's message: {text}"


@m4check("US-023 OBS-01 OBS-02 a waitUntil on p1 for a flag the server sets 3 s later returns true, its step under 5 s", "session", [WAIT], "ded")
def us023_wait(ctx):
    ex = ded_ex(ctx, WAIT)
    assert ex["outcome"] == "pass", f"{WAIT} ended {ex['outcome']}: {ex['reason']}"
    st = [s for s in ex["steps"] if s["path"] == ["wait for the flag"]]
    assert len(st) == 1, f"the result has {len(st)} steps wait for the flag"
    d = st[0]["duration_s"]
    assert 2.5 <= d < 5, f"the wait's step took {d} s, expected about 3 s and under 5 s"


@m4check("US-023 OBS-07 tryWaitUntil with a ceiling of 1 s on a condition that never holds returns false and the check passes", "session", [TRY], "ded")
def us023_try(ctx):
    ex = ded_ex(ctx, TRY)
    assert ex["outcome"] == "pass", f"{TRY} ended {ex['outcome']}: {ex['reason']}"
    assert ["after the wait"] in [s["path"] for s in ex["steps"]], "the check did not go on after the wait"


@m4check("US-023 OBS-01 wait --machine server --condition false --ceiling 2 exits 0 after about 2 s with met false, and wait_until returns the same", "session", (), "ded")
def us023_cli(ctx):
    cli, (tool, seconds) = got(ctx, "wait-false")
    assert cli.code == 0, f"{cli}: {message(cli)}"
    out = cli.json()
    assert out["met"] is False and 1.8 <= out["waited_s"] <= 3 and 1.8 <= cli.seconds < 6, \
        f"wait gave {out} in {cli.seconds:.1f} s"
    assert tool["met"] is False and 1.8 <= tool["waited_s"] <= 3 and seconds < 6, f"wait_until gave {tool} in {seconds:.1f} s"
    same_json(out, tool, "wait server false")


@m4check("US-042 LOG-03 LOG-04 logs --machine p2 --check --run prints only p2's lines of that check, --grep and --last narrow them, and the logs tool returns the same", "session", [LOGS, OTHER], "ded")
def us042_logs(ctx):
    got_logs = got(ctx, "logs")
    mine = [f"acc-m04 US-042 line {n}" for n in (1, 2, 3)]
    for name, (cli, tool) in got_logs.items():
        assert cli.code == 0, f"logs {name}: {cli}: {message(cli)}"
        same_json(cli.json(), tool, f"logs {name}")
    lines = got_logs["all"][0].json()["lines"]
    for text in mine:
        assert sum(text in l for l in lines) == 1, f"p2's lines of the check hold {text!r} {sum(text in l for l in lines)} times"
    for foreign in ("a line of p1", "a line of the next check"):
        assert not any(foreign in l for l in lines), f"p2's lines of the check hold {foreign!r}"
    assert any("[shakedown]" in l and " start " in l for l in lines) or len(lines) < 50, \
        f"the check's lines are {len(lines)}, not cut at its markers"
    grep = got_logs["grep"][0].json()["lines"]
    assert grep and all("line 2" in l for l in grep) and any(mine[1] in l for l in grep), f"--grep line 2 gave {grep}"
    last = got_logs["last"][0].json()["lines"]
    assert len(last) == 1 and last[0] == lines[-1], f"--last 1 gave {last}, the last line is {lines[-1:]}"


@m4check("US-061 CTL-05 ERR-03 code on p1 that raises a script error returns 7 and the body runs on; the check ends fail script-error naming p1, _skdfx_undefined and fn_throw.sqf in step throw on p1", "session", [THROW], "ded")
def us061_check(ctx):
    ex = ded_ex(ctx, THROW)
    msg = a_message(ex)
    assert ex["outcome"] == "fail" and ex["reason"]["kind"] == "script-error", f"{THROW} ended {ex['outcome']} / {ex['reason']}"
    assert ex["reason"].get("machine") == "p1", f"reason.machine is {ex['reason'].get('machine')!r}"
    assert "_skdfx_undefined" in msg and "fn_throw.sqf" in msg, f"the message is {msg!r}"
    assert ex["evidence"]["failed_step"] == ["throw on p1"], f"evidence.failed_step is {ex['evidence']['failed_step']}"
    assert ["after the error"] in [s["path"] for s in ex["steps"]], "the body did not run on after the error"
    blob = json.dumps(ex["evidence"])
    assert "reached the end" in blob, "the evidence of the body's end is missing"


@m4check("US-061 CTL-05 ERR-05 exec on p1 of skdfx_fnc_throw; 7 exits 0 with done, 7 and the error with machine, text, file and line; the tool returns the same, not isError", "session", (), "ded")
def us061_exec(ctx):
    cli, tool = got(ctx, "exec-throw")
    assert cli.code == 0, f"{cli}: {message(cli)}"
    out = cli.json()
    r = out["results"]["p1"]
    assert r["state"] == "done" and r["value"] == 7, f"exec gave {r}"
    errs = [e for e in out["errors"] if e["machine"] == "p1" and "_skdfx_undefined" in e["text"]]
    assert len(errs) == 1, f"exec lists {len(errs)} errors of p1 holding _skdfx_undefined: {out['errors']}"
    e = errs[0]
    assert e["file"].endswith("fn_throw.sqf") and isinstance(e["line"], int) and e["line"] > 0 and e["check_id"] is None, f"the error is {e}"
    same_json(out, structured(tool), "exec throw")


@m4check("US-061 ERR-05 errors --machine p1 afterwards lists that error with its time, as raised outside any check", "session", (), "ded")
def us061_errors(ctx):
    js, text = got(ctx, "errors-p1")
    assert js.code == 0 and text.code == 0, f"{js}; {text}"
    errs = [e for e in js.json()["errors"] if e["machine"] == "p1" and "_skdfx_undefined" in e["text"]
            and e["file"].endswith("fn_throw.sqf") and e["check_id"] is None]
    assert errs and all(e["at"]["utc"] for e in errs), f"errors --machine p1 --json lists no error of exec's throw outside a check"
    assert re.search(r"_skdfx_undefined.*fn_throw\.sqf.*outside any check", text.stdout), \
        f"errors --machine p1 does not list the error outside any check: {text.stdout[-600:]}"


@m4check("US-062 OBS-01 a waitUntil whose condition raises a script error ends fail script-error within 2 s naming p1 and the error, listed once", "session", [ERR], "ded")
def us062_error(ctx):
    ex = ded_ex(ctx, ERR)
    msg = a_message(ex)
    assert ex["outcome"] == "fail" and ex["reason"]["kind"] == "script-error", f"{ERR} ended {ex['outcome']} / {ex['reason']}"
    assert ex["reason"].get("machine") == "p1" and "_skdfx_undefined" in msg, f"the reason is {ex['reason']}"
    assert decided_in(ex) < 2, f"the check was decided {decided_in(ex):.2f} s after its start"
    errs = [e for e in ex["errors"] if "_skdfx_undefined" in e["text"]]
    assert len(errs) == 1, f"the result lists the error {len(errs)} times: {ex['errors']}"


def decided_in(ex):
    """Seconds from the execution's start to the moment its reason was decided. The execution itself goes on
    after that, through the wait for script errors still in flight (result-file.md)."""
    from datetime import datetime
    t = lambda v: datetime.fromisoformat(v.replace("Z", "+00:00"))
    return (t(ex["reason"]["at"]) - t(ex["started_at"])).total_seconds()


def string_ended(ex, cid):
    msg = a_message(ex)
    assert ex["outcome"] == "fail" and ex["reason"]["kind"] == "check-error", f"{cid} ended {ex['outcome']} / {ex['reason']}"
    assert "a string, not a Boolean" in msg and "String" in msg and "Boolean" in msg, f"the message is {msg!r}"
    assert decided_in(ex) < 1, f"the check was decided {decided_in(ex):.2f} s after its start, not at once"


@m4check("US-062 OBS-01 DM-04 a waitUntil whose condition returns a String ends fail check-error at once naming the description, String and Boolean", "session", [STRING], "ded")
def us062_string(ctx):
    string_ended(ded_ex(ctx, STRING), STRING)


@m4check("US-062 OBS-01 tryWaitUntil with either broken condition ends the check the same way instead of returning false", "session", [TRY_ERR, TRY_STRING], "ded")
def us062_try(ctx):
    ex = ded_ex(ctx, TRY_ERR)
    assert ex["outcome"] == "fail" and ex["reason"]["kind"] == "script-error" and ex["reason"].get("machine") == "p1", \
        f"{TRY_ERR} ended {ex['outcome']} / {ex['reason']}"
    assert decided_in(ex) < 2, f"{TRY_ERR} was decided {decided_in(ex):.2f} s after its start"
    string_ended(ded_ex(ctx, TRY_STRING), TRY_STRING)


@m4check("US-062 OBS-01 wait --machine p1 on a condition that raises a script error exits 0 within 2 s with met false and the error, and wait_until returns the same", "session", (), "ded")
def us062_cli(ctx):
    cli, (tool, seconds) = got(ctx, "wait-error")
    assert cli.code == 0 and cli.seconds < 2, f"{cli}: {message(cli)}"
    out = cli.json()
    assert out["met"] is False and out["error"] and "_skdfx_undefined" in out["error"], f"wait gave {out}"
    assert seconds < 2, f"wait_until answered in {seconds:.1f} s"
    same_json(out, tool, "wait p1 on an error")


# ================================================================ the sp session

@batch("sp")
def sp_batch(ctx, checks):
    if not any(c.name in M4_NAMES for c in checks):
        return
    calls = Calls(ctx)
    sh = ctx.sh
    try:
        step(ctx, "status-ready", lambda: sh("status", "--json").json())
        step(ctx, "two-runs", lambda: (ctx.run([SEL_A, SEL_B, SEL_C]), ctx.run([SEL_C, SEL_B, SEL_A])))
        step(ctx, "second-start", lambda: (sh("status", "--json").json(), sh("session", "start", "--setup", "sp", ceiling=120),
                                           calls("session_start", {"setup": "sp"})[0], sh("status", "--json").json()))
        step(ctx, "by-tag", lambda: follow(calls, {"tags": ["acc-sel-x"]}))
        step(ctx, "nothing", lambda: nothing(ctx, calls))
        step(ctx, "same-status", lambda: (sh("status", "--json").json(), calls.ok("status")[0]))
        step(ctx, "same-inspect", lambda: (sh("inspect", "--machine", "p1", "--json").json(), calls.ok("inspect", {"machine": "p1"})[0]))
        step(ctx, "same-exec", lambda: (sh("exec", "--machine", "p1", "--code", "1 + 1", "--json").json(),
                                        calls.ok("exec", {"machine": "p1", "code": "1 + 1"})[0]))
        step(ctx, "same-result", lambda: (sh("run", FAILS, SEL_A, "--json", ceiling=300), follow(calls, {"ids": [FAILS, SEL_A]})))
        step(ctx, "long-logs", lambda: calls.ok("logs", {"machine": "p1", "last": 5000})[0])
        step(ctx, "long-exec", lambda: calls.ok("exec", {"machine": "p1", "code":
                                                         "private _a = []; for '_i' from 1 to 3000 do {_a pushBack 'xxxxxxxx'}; _a joinString ''"})[0])
        step(ctx, "p9", lambda: calls("exec", {"machine": "p9", "code": "1"})[0])
        step(ctx, "ceiling-55", lambda: calls("exec", {"machine": "p1", "code": "1", "ceiling": 55})[0])
        step(ctx, "long-run", lambda: long_run(ctx, calls))
        step(ctx, "cancel-exec", lambda: cancel_exec(ctx, calls))
        step(ctx, "cancel-wait", lambda: cancel_wait(ctx, calls))
        if any(c.name.startswith("NFR-PF-03") for c in checks):
            step(ctx, "trips-sp", lambda: trips(calls, "p1"))
    finally:
        calls.close()


def register(ctx, run_result):
    """Adds a run started through run_start to the suite's runs, as ctx.run does for `shakedown run`, so the
    checks over every run (SES-05, NFR-RL-05) see it. There is no command answer and no new-folder list."""
    result = json.loads(Path(run_result["result_file"]).read_text(encoding="utf-8"))
    ctx.runs.append((None, run_result, result, []))
    ctx.session_runs.append((None, run_result, result, []))


def follow(calls, selection):
    """run_start, run_status with wait_s 50 until finished (at most 20 calls), run_result."""
    start, seconds = calls.ok("run_start", selection)
    statuses = []
    for _ in range(20):
        st, s = calls.ok("run_status", {"run_id": start["run_id"], "wait_s": 50})
        statuses.append((st, s))
        if st["state"] in ("finished", "stopped"):
            break
    result = calls.ok("run_result", {"run_id": start["run_id"]})[0]
    register(calls.ctx, result)
    return start, seconds, statuses, result


def nothing(ctx, calls):
    before = ctx.sh.run_folders()
    cli = ctx.sh("run", "--tag", "US-999", ceiling=120)
    tool = calls("run_start", {"tags": ["US-999"]})[0]
    after = ctx.sh.run_folders()
    return cli, tool, before, after, ctx.sh("status", "--json").json()


def long_run(ctx, calls):
    """A run of a fast check, a check of about 55 s and a fast check, followed step by step."""
    out = {}
    start, out["start_s"] = calls.ok("run_start", {"ids": [SEL_A, SLOW, SEL_B]})
    out["start"] = start
    rid = start["run_id"]
    out["early_result"] = calls("run_result", {"run_id": rid})[0]
    out["status1"] = calls.ok("run_status", {"run_id": rid, "wait_s": 50})
    out["cli_status"] = ctx.sh("status", "--json").json()
    out["tool_status"] = calls.ok("status")[0]
    rq, t0 = calls.m.start_call("run_status", {"run_id": rid, "wait_s": 50}, token="long-run")
    at, msg = calls.m.wait(rq, 70)
    out["status2"] = (structured(msg["result"]), at - t0)
    out["progress2"] = (t0, [t for t, _ in calls.m.progress("long-run")], at)
    out["status3"] = calls.ok("run_status", {"run_id": rid, "wait_s": 50})
    rest = []
    for _ in range(10):
        st = calls.ok("run_status", {"run_id": rid, "wait_s": 50})
        rest.append(st)
        if st[0]["state"] in ("finished", "stopped"):
            break
    out["rest"] = rest
    out["result"] = calls("run_result", {"run_id": rid})[0]
    register(ctx, structured(out["result"]))
    return out


EXEC_NAME = "skd_exec"


def exec_scripts(ctx):
    js = ctx.sh("inspect", "--machine", "p1", "--json").json()
    return [s for s in js.get("scripts") or [] if EXEC_NAME in (s.get("name") or "")]


def cancel_exec(ctx, calls):
    """An exec of code that waits forever, cancelled after 5 s; when its script leaves inspect's scripts."""
    out = {"seen": []}
    rid, t0 = calls.m.start_call("exec", {"machine": "p1", "code": "waitUntil {false}; 1", "ceiling": 50})
    for _ in range(50):  # every 0.2 s until 5 s have passed
        found = exec_scripts(ctx)
        out["seen"].append(len(found))
        if time.monotonic() - t0 >= 5:
            break
        time.sleep(0.2)
    calls.m.cancel(rid)
    cancelled = time.monotonic()
    out["gone_after"] = None
    for _ in range(40):  # every 0.25 s, at most 10 s
        if not exec_scripts(ctx):
            out["gone_after"] = time.monotonic() - cancelled
            break
        time.sleep(0.25)
    out["state"] = ctx.sh("status", "--json").json()["session"]["state"]
    out["next"] = calls("status")
    return out


def cancel_wait(ctx, calls):
    """A wait_until whose condition counts its evaluations, cancelled after 3 s; whether the count still grows."""
    out = {}
    count = "missionNamespace getVariable ['acc_m04_polls', 0]"
    rid, t0 = calls.m.start_call("wait_until", {"machine": "p1", "ceiling": 50,
                                                "condition": f"acc_m04_polls = ({count}) + 1; false"})
    for _ in range(30):  # every 0.2 s until 3 s have passed
        out["before"] = ctx.sh("exec", "--machine", "p1", "--code", count, "--json").json()["results"]["p1"]["value"]
        if time.monotonic() - t0 >= 3:
            break
        time.sleep(0.2)
    calls.m.cancel(rid)
    cancelled = time.monotonic()
    out["stopped_after"] = None
    for _ in range(5):  # at most five windows of 2 s
        n = ctx.sh("exec", "--machine", "p1", "--code", count, "--json").json()["results"]["p1"]["value"]
        w = ctx.sh("wait", "--machine", "p1", "--condition", f"({count}) > (_this select 0)", "--args", json.dumps([n]),
                   "--ceiling", "2", "--json").json()
        if w["met"] is False and not w.get("error"):
            out["stopped_after"] = time.monotonic() - cancelled
            break
    out["next"] = calls("status")
    return out


@m4check("US-001 SES-01 session start --setup sp exits 0 ready, and status shows the session ready with one machine p1 ready", "session")
def us001_ready(ctx):
    assert ctx.start_answer.code == 0 and ctx.session["state"] == "ready", f"{ctx.start_answer}"
    st = got(ctx, "status-ready")
    s = st["session"]
    assert s and s["state"] == "ready" and s["id"] == ctx.session["id"], f"status shows {s and s['state']}"
    assert [(m["name"], m["state"]) for m in s["machines"]] == [("p1", "ready")], f"status shows the machines {s['machines']}"


@m4check("US-001 SES-05 two runs of three checks against the session write result files with the same session id and p1 start 1", "session", [SEL_A, SEL_B, SEL_C])
def us001_two_runs(ctx):
    (a1, o1, r1), (a2, o2, r2) = got(ctx, "two-runs")
    assert r1["run"]["id"] != r2["run"]["id"], "the two runs have one id"
    for r in (r1, r2):
        assert len(r["checks"]) == 3, f"a run holds {len(r['checks'])} executions"
        assert r["session"]["id"] == ctx.session["id"], f"a run's session.id is {r['session']['id']}, the session is {ctx.session['id']}"
        p1 = [m for m in r["session"]["machines"] if m["name"] == "p1"]
        assert len(p1) == 1 and p1[0]["start"] == 1, f"p1 in the result file: {p1}"


@m4check("US-001 NFR-US-03 the session_start tool gives the same session shape as session start --setup sp --json", "session")
def us001_same_shape(ctx):
    tool = got(ctx, "mcp-sp", ctx.m04_keep)
    cli = ctx.sessions["sp"]
    assert set(tool) == set(cli), f"the session keys differ: {sorted(set(tool) ^ set(cli))}"
    for k in ("setup", "mode", "state", "reason"):
        assert tool[k] == cli[k], f"{k} is {tool[k]!r} from the tool and {cli[k]!r} from the command"
    shape = lambda s: [(m["name"], m["kind"], m["start"], m["state"], sorted(m), sorted(m["health"])) for m in s["machines"]]
    assert shape(tool) == shape(cli), f"the machines differ: {shape(tool)} vs {shape(cli)}"


@m4check("US-005 SES-02 a second session start exits 4 naming the session's id, its setup and shakedown session stop; session_start returns session-exists; the session stays ready with the same machines", "session")
def us005(ctx):
    before, cli, tool, after = got(ctx, "second-start")
    said = message(cli)
    sid = ctx.session["id"]
    assert cli.code == 4, f"{cli}, expected exit 4: {said}"
    assert sid in said and re.search(r"\bsp\b", said) and "shakedown session stop" in said, f"the message is {said!r}"
    text = tool_error(tool, "session-exists")
    assert sid in text, f"session-exists does not name {sid}: {text}"
    machines = lambda st: [(m["name"], m["start"], m["pid"]) for m in st["session"]["machines"]]
    assert after["session"]["state"] == "ready" and after["session"]["id"] == sid, f"after the refusal the session is {after['session']['state']}"
    assert machines(after) == machines(before), f"the machines changed: {machines(before)} -> {machines(after)}"


@m4check("US-031 RUN-01 AGT-02 run_start with tags acc-sel-x runs the checks list_checks selects, and run_result reports them", "session", [SEL_A, SEL_C])
def us031_run_start(ctx):
    start, _, statuses, result = got(ctx, "by-tag")
    assert start["total"] == 2, f"run_start gave total {start['total']}"
    assert statuses[-1][0]["state"] == "finished", f"the run ended {statuses[-1][0]['state']}"
    ran = [o["check_id"] for o in result["outcomes"]]
    assert ran == [SEL_A, SEL_C], f"run_result reports {ran}, list --tag acc-sel-x selects {[SEL_A, SEL_C]}"


@m4check("US-032 RUN-01 run --tag US-999 exits 3 naming US-999 and the tags, run_start returns unknown-check, no run folder is made and the session stays ready", "session")
def us032(ctx):
    cli, tool, before, after, st = got(ctx, "nothing")
    said = message(cli)
    assert cli.code == 3, f"{cli}, expected exit 3: {said[:300]}"
    assert "US-999" in said and "acc-sel-x" in said, f"the message does not name US-999 and the tags: {said[:300]}"
    tool_error(tool, "unknown-check")
    assert after == before, f"run folders appeared: {sorted(set(after) - set(before))}"
    assert st["session"]["state"] == "ready", f"the session is {st['session']['state']}"


@m4check("US-048 NFR-US-03 status, inspect and exec give the same JSON as the command with --json", "session")
def us048_same(ctx):
    for key, what in (("same-status", "status"), ("same-inspect", "inspect p1"), ("same-exec", "exec p1 1 + 1")):
        cli, tool = json.loads(json.dumps(got(ctx, key)))  # a copy: the AGT-02 check reads the answers as they came
        if key == "same-inspect":  # the player stands, but a reading at another moment may show another nearby or fps
            for v in (cli, tool):
                v.pop("errors", None)
        same_json(cli, tool, what)


@m4check("US-048 AGT-02 NFR-US-03 a run with a failing check is a successful run_result call whose outcome is fail, the same JSON as run --json", "session", [FAILS, SEL_A])
def us048_run_result(ctx):
    cli, (start, _, statuses, result) = got(ctx, "same-result")
    assert cli.code == 1, f"{cli}, expected exit 1"
    outcomes = {o["check_id"]: o["outcome"] for o in result["outcomes"]}
    assert outcomes == {FAILS: "fail", SEL_A: "pass"}, f"run_result gives {outcomes}"
    out, result = cli.json(), json.loads(json.dumps(result))  # a copy: the AGT-02 check reads the answer as it came
    for v in (out, result):
        v["run_id"] = "<run>"
        v["result_file"] = "<file>"
        for o in v.get("outcomes", []):
            if o.get("evidence_path"):
                o["evidence_path"] = Path(o["evidence_path"]).name
    same_json(out, result, "run_result and run --json")


@m4check("US-048 AGT-07 logs with last 5000 and an exec value over 20,000 characters return a path to a file holding the whole result", "session")
def us048_long(ctx):
    logs = got(ctx, "long-logs")
    if "result_path" in logs:
        held = json.loads(Path(logs["result_path"]).read_text(encoding="utf-8"))
        assert len(held["lines"]) > 0, "the result file holds no lines"
    else:
        assert len(json.dumps(logs)) < 20000, f"logs returned {len(json.dumps(logs))} characters"
        assert logs["truncated"] is True and logs["path"], f"logs with last 5000 gave truncated {logs['truncated']}, path {logs['path']}"
        held = Path(logs["path"]).read_text(encoding="utf-8", errors="replace").splitlines()
        assert len(held) > len(logs["lines"]), f"the file holds {len(held)} lines, the result {len(logs['lines'])}"
    ex = got(ctx, "long-exec")
    assert set(ex) == {"result_path"}, f"an exec value of 24,000 characters returned {str(ex)[:200]}"
    path = Path(ex["result_path"])
    held = json.loads(path.read_text(encoding="utf-8"))
    assert held["results"]["p1"]["value"] == "x" * 24000, "the result file does not hold the value"
    results = ctx.project / ".shakedown" / "results"
    assert path.parent == results, f"the result file is {path}, not in {results} (mcp-tools.md)"


@m4check("US-048 AGT-02 every successful tool call of the round returns structuredContent valid against its outputSchema and the same JSON as one text block", "session")
def us048_schema(ctx):
    calls = ctx.m04.get("calls") or []
    assert calls, "no tool call was made"
    bad = []
    for name, args, result, out in calls:
        if result.get("isError"):
            continue
        try:
            sc = structured(result)
        except AssertionError as e:
            bad.append(f"{name}: {e}")
            continue
        if out is None:
            bad.append(f"{name}: no outputSchema")
            continue
        errs = schema.validate(sc, out)
        if errs:
            bad.append(f"{name} {json.dumps(args)[:80]}: {errs[:3]}")
    assert not bad, f"{len(bad)} of {len(calls)} calls: {bad[:4]}"


@m4check("US-049 AGT-02 NFR-US-01 exec with machine p9 returns unknown-machine: <what is wrong>. <what to do next>. naming the machines", "session")
def us049_p9(ctx):
    text = tool_error(got(ctx, "p9"), "unknown-machine")
    assert "p9" in text and re.search(r"\bp1\b", text), f"the error does not name p9 and the machines: {text}"


@m4check("US-049 AGT-02 run_result on a run that has not ended returns unknown-run saying to call run_status, and exec with ceiling 55 returns bad-arguments saying the most is 50", "session", [SEL_A, SLOW, SEL_B])
def us049_run_ceiling(ctx):
    text = tool_error(got(ctx, "long-run")["early_result"], "unknown-run")
    assert "run_status" in text, f"unknown-run does not say to call run_status: {text}"
    text = tool_error(got(ctx, "ceiling-55"), "bad-arguments")
    assert "50" in text and "55" in text, f"bad-arguments does not say the most is 50: {text}"


@m4check("US-046 INS-02 INS-04 during a run status --json gives each machine's state and last_answer_at and the run's done, total and current_check, and the status tool the same", "session", [SEL_A, SLOW, SEL_B])
def us046_during_run(ctx):
    lr = got(ctx, "long-run")
    cli, tool = lr["cli_status"], lr["tool_status"]
    for st in (cli, tool):
        assert st["session"]["state"] in ("ready", "busy", "running"), f"the session is {st['session']['state']}"
        for m in st["session"]["machines"]:
            assert m["state"] and m["last_answer_at"], f"machine {m['name']}: state {m['state']}, last_answer_at {m['last_answer_at']}"
        run = st["run"]
        assert run and run["id"] == lr["start"]["run_id"] and run["total"] == 3 and run["done"] == 1 and run["current_check"] == SLOW, \
            f"status shows the run {run}"
    same_json(cli, tool, "status during a run")


@m4check("US-050 AGT-03 run_start returns a run_id within 5 s, and run_status with wait_s 50 returns as soon as a check ends with its outcome, never past 60 s, with progress every 5 s", "session", [SEL_A, SLOW, SEL_B])
def us050_follow(ctx):
    lr = got(ctx, "long-run")
    assert lr["start"]["run_id"] and lr["start_s"] < 5, f"run_start answered in {lr['start_s']:.1f} s: {lr['start']}"
    (s1, t1), (s2, t2), (s3, t3) = lr["status1"], lr["status2"], lr["status3"]
    ids = lambda s: [o["check_id"] for o in s["outcomes"]]
    assert ids(s1) == [SEL_A] and t1 < 10, f"the first run_status answered in {t1:.1f} s with {ids(s1)}, expected {SEL_A} at once"
    assert ids(s2) == [SEL_A] and 45 <= t2 < 60, f"the second answered in {t2:.1f} s with {ids(s2)}, expected at about 50 s with no new outcome"
    assert ids(s3) == [SEL_A, SLOW] and t3 < 20, f"the third answered in {t3:.1f} s with {ids(s3)}, expected the slow check's outcome as it ended"
    every = [t for s, t in [lr["status1"], lr["status2"], lr["status3"]] + lr["rest"]]
    assert max(every) < 60, f"a run_status call took {max(every):.1f} s"
    final = lr["rest"][-1][0]
    assert final["state"] == "finished" and ids(final) == [SEL_A, SLOW, SEL_B], f"the run ended {final['state']} with {ids(final)}"
    t0, marks, at = lr["progress2"]
    gaps = [b - a for a, b in zip([t0] + marks, marks + [at])]
    assert marks and max(gaps) <= 5.5, f"run_status of {t2:.0f} s sent {len(marks)} progress notifications; the longest gap was {max(gaps):.1f} s"
    structured(lr["result"])


@m4check("US-051 AGT-04 an exec of code that waits forever, cancelled after 5 s, leaves inspect's scripts of p1 within 10 s, and the session stays ready", "session")
def us051_exec(ctx):
    c = got(ctx, "cancel-exec")
    assert any(c["seen"]), f"inspect never listed the exec's script before the cancel: {c['seen']}"
    assert c["gone_after"] is not None, "the exec's script was still in inspect's scripts 10 s after the cancel"
    assert c["state"] == "ready", f"the session is {c['state']}"
    structured(c["next"][0])


@m4check("US-051 AGT-04 a cancelled wait_until stops polling on its machine, and the next tool call is answered normally", "session")
def us051_wait(ctx):
    c = got(ctx, "cancel-wait")
    assert c.get("before", 0) > 0, f"the condition was never evaluated before the cancel: {c.get('before')}"
    assert c["stopped_after"] is not None, "the condition was still evaluated 10 s after the cancel"
    result, seconds = c["next"]
    structured(result)
    assert seconds < 5, f"the next call took {seconds:.1f} s"


@m4check("NFR-PF-03 US-013 100 exec calls of 1 + 1 to singleplayer p1 answer 2 with a 95th percentile under 1 s", "session")
def nfr_pf_03_sp(ctx):
    trip_check(ctx, "trips-sp", "singleplayer p1")


# ================================================================ after the sessions stopped

@m4check("US-042 LOG-01 SES-09 after session stop each machine start's RPT is still in its own profile folder in the session folder", "end")
def us042_rpts(ctx):
    checked = 0
    for setup, session in ctx.sessions.items():
        for m in session["machines"]:
            profile = Path(m["profile"])
            assert profile.parent.parent == Path(session["folder"]) / "machines" or Path(session["folder"]) in profile.parents, \
                f"{setup} {m['name']}: the profile {profile} is not in the session folder {session['folder']}"
            rpts = list(profile.glob("*.rpt"))
            assert rpts, f"{setup} {m['name']} start {m['start']}: no RPT in {profile} after the session stopped"
            checked += 1
    assert checked, "no session of the run to look at"


@m4check("US-042 LOG-02 NFR-PV-01 no file of any run folder, no logs answer and no stopped session's RPT holds a 17-digit number starting 7656119", "end")
def us042_no_steam_id(ctx):
    root = ctx.project / ".shakedown"
    files = [p for p in (root / "runs").rglob("*") if p.is_file()] if (root / "runs").is_dir() else []
    files += [p for p in (root / "sessions").rglob("*.rpt")] if (root / "sessions").is_dir() else []
    hits = [str(p) for p in files if STEAM_ID.search(p.read_text(encoding="utf-8", errors="replace"))]
    answers = 0
    for cli, tool in ctx.m04_keep.get("logs-answers", []):
        answers += 1
        if STEAM_ID.search(cli.stdout) or STEAM_ID.search(json.dumps(tool)):
            hits.append("a logs answer")
    ctx.log(f"US-042 searched {len(files)} files and {answers} logs answers for a Steam ID: {len(hits)} hold one")
    assert files, "no run folder file and no RPT to search"
    assert not hits, f"{len(hits)} of them hold a Steam ID: {hits[:5]}"
