"""The Mac half of M4's exit criterion 1: US-046's third criterion on macOS, with no game and no runner.

Usage: python3 acceptance/run_mac.py [--install <dir>] [--repeat <n>] [--work <dir>]

It writes a project with the shakedown.toml of docs/project-file.md, two checks and the .mcp.json
docs/agent-tools.md gives, starts `shakedown mcp` from that .mcp.json with the install folder on PATH,
speaks JSON-RPC to it on stdio and judges status, list_checks and guide. It never starts Claude Code,
the game or Steam. See acceptance/README.md.
"""

import argparse
import json
import platform
import re
import subprocess
import sys
import tempfile
import time
import traceback
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
sys.path.insert(0, str(HERE))

from harness.mcp import Mcp, structured  # noqa: E402
from harness.schema import validate  # noqa: E402

NAME = ("US-046 AGT-05 NFR-PT-02 on the Mac with no game shakedown mcp starts from .mcp.json and answers status "
        "runner not-running, list_checks and guide, and starts nothing else")
GAME = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3"
CHECKS = {
    "mac/a": "// @check US-046 mac check a\n// @tags acc-mac\n\n[\"a\", {true}] call skd_fnc_step;\n",
    "mac/b": "// @check US-046 mac check b\n// @tags acc-mac other\n// @setups sp\n// @timeout 30\n\n[\"b\", {true}] call skd_fnc_step;\n",
}


def make_project(work):
    """The example of docs/project-file.md, whose game folder does not exist on a Mac, two checks, and the .mcp.json."""
    project = work / "project"
    for cid, body in CHECKS.items():
        f = project / "checks" / f"{cid}.sqf"
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_text(body, encoding="utf-8")
    (project / "shakedown.toml").write_text(
        f"game = '{GAME}'\nchecks = \"checks\"\n\n"
        "[mods.cba]\npath = 'C:\\Program Files (x86)\\Steam\\steamapps\\workshop\\content\\107410\\450814997'\n"
        "patch = \"cba_main\"\n\n"
        "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"cba\"]\n\n"
        "[setups.dedicated]\nmode = \"dedicated\"\nclients = 2\nheadless = 1\nmods = [\"cba\"]\n", encoding="utf-8")
    (project / ".mcp.json").write_text(json.dumps(
        {"mcpServers": {"shakedown": {"command": "shakedown", "args": ["mcp"], "timeout": 70000}}}, indent=2),
        encoding="utf-8")
    return project


def processes():
    """{pid: (ppid, command)} of every process of this user."""
    out = subprocess.run(["ps", "-x", "-o", "pid=,ppid=,comm="], capture_output=True, text=True, timeout=30).stdout
    procs = {}
    for line in out.splitlines():
        parts = line.split(None, 2)
        if len(parts) == 3:
            procs[int(parts[0])] = (int(parts[1]), parts[2])
    return procs


def named(procs, *words):
    return {pid for pid, (_, comm) in procs.items() if any(Path(comm).name.lower().startswith(w) for w in words)}


def descendants(procs, root):
    found, todo = set(), [root]
    while todo:
        p = todo.pop()
        kids = {pid for pid, (ppid, _) in procs.items() if ppid == p}
        todo += list(kids - found)
        found |= kids
    return found


def check(project, install, log):
    assert platform.system() == "Darwin", f"this check is the Mac half; it runs on {platform.system()}"
    assert not Path(GAME).exists(), "the project's game folder exists"
    before = processes()
    game = named(before, "arma3", "steam")
    assert not game, f"game or Steam processes {sorted(game)} run on this machine"
    others = named(before, "shakedown")
    assert not others, f"shakedown processes {sorted(others)} run before the check; it needs no runner running"

    m = Mcp(project, install, log)
    try:
        m.initialize()
        assert m.info.get("serverInfo", {}).get("name") == "shakedown", f"the server names itself {m.info.get('serverInfo')}"
        tools = {t["name"]: t for t in m.tools()}
        for name in ("status", "list_checks", "guide"):
            assert name in tools, f"tools/list lacks {name}; it lists {sorted(tools)}"

        def answer(name, args=None):
            result, seconds = m.call(name, args or {}, ceiling=30)
            out = structured(result)
            problems = validate(out, tools[name]["outputSchema"])
            assert not problems, f"{name} does not match its outputSchema: {problems[:5]}"
            return out, seconds

        st, st_s = answer("status")
        assert st["runner"] == "not-running" and st["runner_pid"] is None and st["session"] is None \
            and st["run"] is None, f"status gave {st}"
        lc, lc_s = answer("list_checks")
        ids = [c["id"] for c in lc["checks"]]
        assert sorted(ids) == sorted(CHECKS), f"list_checks gave {ids}, the project has {sorted(CHECKS)}"
        bad = [c for c in lc["checks"] if c["error"] is not None]
        assert not bad, f"list_checks reports header errors: {bad}"
        b = next(c for c in lc["checks"] if c["id"] == "mac/b")
        assert b["setups"] == ["sp"] and b["timeout_s"] == 30 and b["tags"] == ["acc-mac", "other"], f"mac/b listed as {b}"
        tagged, _ = answer("list_checks", {"tags": ["other"]})
        assert [c["id"] for c in tagged["checks"]] == ["mac/b"], f"list_checks tags other gave {tagged}"
        gd, gd_s = answer("guide")
        doc = (REPO / "docs" / "agent-tools.md").read_text(encoding="utf-8")
        block = re.search(r"## The guide.*?```text\n(.*?)```", doc, re.S)
        norm = lambda s: re.sub(r"\s+", " ", s).strip()  # noqa: E731
        assert block and norm(gd["text"]) == norm(block.group(1)), "guide differs from the text docs/agent-tools.md gives"
        assert len(gd["text"]) < 8000, f"guide is {len(gd['text'])} characters"

        during = processes()
        assert m.proc.poll() is None, "the server exited during the calls"
        spawned = descendants(during, m.pid)
        assert not spawned, f"the server started processes {[(p, during[p][1]) for p in spawned]}"
        others = named(during, "shakedown") - {m.pid}
        assert not others, f"shakedown processes {sorted(others)} besides the server run after the calls"
    finally:
        m.close()
    after = named(processes(), "shakedown")
    assert not after, f"shakedown processes {sorted(after)} run after the server closed"
    return (f"status runner {st['runner']}, session null, run null in {st_s:.2f} s; list_checks {len(ids)} checks"
            f" in {lc_s:.2f} s; guide {len(gd['text'])} characters, the documented text, in {gd_s:.2f} s;"
            f" server pid {m.pid} exited {m.proc.returncode}, nothing else started")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--install", default=str(REPO / "target" / "release"), help="the folder holding the shakedown binary")
    ap.add_argument("--repeat", type=int, default=1)
    ap.add_argument("--work")
    a = ap.parse_args()
    install = Path(a.install).resolve()
    if not (install / "shakedown").is_file():
        print(f"no shakedown binary in {install}")
        return 3
    work = Path(a.work) if a.work else Path(tempfile.mkdtemp(prefix="shakedown-mac-"))
    work.mkdir(parents=True, exist_ok=True)
    lines = []

    def log(text):
        lines.append(f"{time.strftime('%H:%M:%S')} {text}")

    project = make_project(work)
    results = []
    for n in range(1, a.repeat + 1):
        try:
            detail = check(project, install, log)
            results.append({"check": NAME, "repeat": n, "pass": True, "detail": detail})
            print(f"PASS {NAME} #{n}\n     {detail}")
        except Exception as e:  # noqa: BLE001  a harness error fails the check it happened in
            results.append({"check": NAME, "repeat": n, "pass": False, "detail": f"{e}",
                            "trace": traceback.format_exc()})
            print(f"FAIL {NAME} #{n}\n     {e}")
    (work / "harness.log").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (work / "report.json").write_text(json.dumps(results, indent=2), encoding="utf-8")
    passed = sum(r["pass"] for r in results)
    print(f"{passed} of {len(results)} passed; {work / 'report.json'}")
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    sys.exit(main())
