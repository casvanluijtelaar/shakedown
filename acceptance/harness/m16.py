r"""The checks of M16 A start that explains itself: US-085, US-086, US-087 and DEF-29's acceptance criteria.

Each part starts and stops what it needs itself, once per repeat (`--repeat`), with no session running before it,
in the project `m16` of its own in the work folder. Its `tests\` holds the check of `acceptance/m16/`; its
`shakedown.toml` sets `base_port = 2602` and names the mod `fixture` (`@skd_fixture`, patch `skdfx_main`, with a
`version` only where a part writes one) and the mod `gone`, whose folder does not exist. Its setups are `sp`
(singleplayer), `ded` (a dedicated server with two clients), `hung` (`ded` with `start_timeout = 150`, room for p1 to be ready on a cold machine, so p2 alone is still `launched` at the ceiling) and `gap`
(a dedicated server with one client loading `fixture` and `gone`), each loading `fixture`.

Per repeat:

- `refused`: UDP port 2604 held by a socket of the harness; `session start --setup gap` while the game processes
  are listed every second, `status`, and `session_start` of `gap` through `shakedown mcp` (US-085 1 and 3); then
  in the project `m16bad`, whose mod names `version` without `patch`, `session start --setup sp` (ENV-01, CFG-01).
- `disk`: a project written into an ISO image the harness builds with the system's IMAPI2 file system image and
  mounts with `Mount-DiskImage`, so its drive has no free space; `session start --setup sp` and `session_start`
  from it, then the image is dismounted (US-085 2).
- `version`: in singleplayer, a start with no `version` key, a run of the check that reads the version the result
  file records, and `session stop`; a start with `version = "9.9.9"`, `status`, `session stop`; a start with the
  version recorded and a run of the check; then a dedicated start with `version = "9.9.9"` (US-086, DEF-29 3).
- `hung`: `session start --setup hung` with p2's process suspended by its id as soon as `status` lists it, then
  `status`, `session stop` and `session.json` (US-087, DEF-29 2).
- `exit`: `session start --setup ded` with p2's process ended by its id once its RPT holds 25 lines, then
  `session.json`, `session stop` and `session.json` (DEF-29 1).

The logged-out Steam of US-085 1 is the owner's look: the harness does not log Steam out. A suspended process is
resumed or stopped by its id before the part ends, every game process a part saw is stopped by its id if it
outlives the part, and a part lost to `Error 20 reading file` of a base-game file starts again, at most three times
in all.
"""

import json
import re
import shutil
import socket
import subprocess
import time
import traceback
from pathlib import Path

from . import driver, win
from .core import check
from .m04 import message
from .m08 import game_folder
from .m11 import machines, poll, rpt_lines, status_of
from .m13 import Procs, mcp_json, need_quiet
from .mcp import Mcp, structured, tool_error

ACC = Path(__file__).resolve().parent.parent
CHECK = "US-086-version"
BASE_PORT, HELD_PORT = 2602, 2604
WANT = "9.9.9"
STEAM_ID = re.compile(r"7656119\d{10}")

MKISO = r"""
param([string]$Src, [string]$Out)
$cp = New-Object CodeDom.Compiler.CompilerParameters
$cp.CompilerOptions = "/unsafe"
Add-Type -CompilerParameters $cp -TypeDefinition @"
public class SkdIsoFile {
  public unsafe static void Create(string path, object stream, int blockSize, int totalBlocks) {
    int bytes = 0; byte[] buf = new byte[blockSize]; var ptr = (System.IntPtr)(&bytes);
    var o = System.IO.File.OpenWrite(path); var i = stream as System.Runtime.InteropServices.ComTypes.IStream;
    while (totalBlocks-- > 0) { i.Read(buf, blockSize, ptr); o.Write(buf, 0, bytes); }
    o.Flush(); o.Close();
  }
}
"@
$fsi = New-Object -ComObject IMAPI2FS.MsftFileSystemImage
$fsi.FileSystemsToCreate = 3
$fsi.VolumeName = "SKDFULL"
$fsi.Root.AddTree($Src, $false)
$r = $fsi.CreateResultImage()
[SkdIsoFile]::Create($Out, $r.ImageStream, $r.BlockSize, $r.TotalBlocks)
"""


# ================================================================ the project

def keep(ctx):
    if not hasattr(ctx, "m16_keep"):
        ctx.m16_keep = {}
    return ctx.m16_keep


def toml(ctx, version=None):
    game = game_folder(ctx.project)
    v = f'version = "{version}"\n' if version else ""
    return (f"game = '{game}'\nbase_port = {BASE_PORT}\n\n"
            f"[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n{v}\n"
            f"[mods.gone]\npath = '{ctx.work / 'm16-no-such-mod'}'\n\n"
            "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n\n"
            "[setups.ded]\nmode = \"dedicated\"\nclients = 2\nmods = [\"fixture\"]\n\n"
            "[setups.hung]\nmode = \"dedicated\"\nclients = 2\nstart_timeout = 150\nmods = [\"fixture\"]\n\n"
            "[setups.gap]\nmode = \"dedicated\"\nclients = 1\nmods = [\"fixture\", \"gone\"]\n")


def project(ctx, version=None):
    """The project m16, with its shakedown.toml written for this version of the fixture (None: no key)."""
    store = keep(ctx)
    if "project" not in store:
        root = ctx.work / "m16"
        if root.exists():
            shutil.rmtree(root)
        shutil.copytree(ACC / "m16", root / "tests")
        mcp_json(root)
        store["project"] = (root, driver.Shakedown(ctx.sh.exe, root, ctx.log))
    root, sh = store["project"]
    (root / "shakedown.toml").write_text(toml(ctx, version), encoding="utf-8")
    return root, sh


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
                ctx.log(f"M16 part {k}: {traceback.format_exc()}")
                store[k] = e
                fault = machine_fault(root, began)
                if not fault or attempt == 3:
                    break
                ctx.log(f"M16 part {k}: a game process could not read a base-game file ({fault}); the machine, "
                        f"not the framework. Starting the part again, attempt {attempt + 1} of 3")
    value = store[k]
    if isinstance(value, Exception):
        raise AssertionError(f"the M16 part {key} failed: {value!r}")
    return value


# ================================================================ helpers

def quiet(sh):
    return driver.Shakedown(sh.exe, sh.project, lambda text: None)


def session_json(root, sid):
    p = Path(root) / ".shakedown" / "sessions" / sid / "session.json"
    return json.loads(p.read_text(encoding="utf-8")) if p.exists() else None


def gone_after(pids, ceiling):
    """The pids still alive after polling once a second, at most `ceiling` times."""
    left = set(pids)
    for _ in range(ceiling):
        left = {p for p in left if driver.alive(p)}
        if not left:
            break
        time.sleep(1)
    return left


def tidy(ctx, sh, pids, suspended=()):
    """Resumes what the part suspended, stops a session it left, then every game process it saw, by its id."""
    for pid in suspended:
        if driver.alive(pid):
            try:
                win.resume(pid)
            except Exception as e:  # noqa: BLE001
                ctx.log(f"M16: resuming {pid}: {e!r}")
    try:
        if status_of(sh).get("session"):
            sh("session", "stop", "--json", ceiling=180)
    except Exception as e:  # noqa: BLE001
        ctx.log(f"M16: status or session stop after the part: {e!r}")
    for pid in set(pids):
        if driver.alive(pid) and driver.wait_gone(pid, 30) is None:
            ctx.log(f"M16: game process {pid} still runs; stopping it by id")
            driver.kill(pid)
    ctx.runner_pid = driver.runner_pid() or ctx.runner_pid


def start(sh, setup, pids, ceiling=400):
    """`session start --setup <setup> --json`: (answer, Session JSON or None)."""
    answer = sh("session", "start", "--setup", setup, "--json", ceiling=ceiling)
    try:
        session = json.loads(answer.stdout) if answer.stdout.strip() else None
    except ValueError:
        session = None
    pids |= {m["pid"] for m in (session or {}).get("machines", []) if m.get("pid")}
    return answer, session


def run_check(sh):
    answer = sh("run", CHECK, "--json", ceiling=600)
    out = answer.json()
    result = json.loads(Path(out["result_file"]).read_text(encoding="utf-8"))
    return {"answer": answer, "out": out, "result": result}


def watched_start(ctx, sh, setup, act):
    """Starts `session start --setup <setup> --json` in the background and calls act(p2) once `status` lists p2
    with a process id. Returns what it saw: the Session JSON, the exit code, the time, p2 and what act returned."""
    q = quiet(sh)
    out = {"seen": set()}
    began = time.monotonic()
    bg = sh.start("session", "start", "--setup", setup, "--json")
    try:
        def p2_listed():
            if bg.poll() is not None:
                return {"ended": True}
            try:
                ms = machines(q("status", "--json", ceiling=30).json())
            except Exception:  # noqa: BLE001  a status that failed once is read again
                return None
            out["seen"] |= {m["pid"] for m in ms.values() if m.get("pid")}
            m = ms.get("p2")
            return m if m and m.get("pid") else None
        p2 = poll(p2_listed, 300, 0.2)
        out["p2"] = p2 if p2 and p2.get("pid") else None
        if out["p2"]:
            out["acted"] = act(out["p2"], began)
        stdout, stderr = bg.communicate(timeout=400)
        out["took"] = time.monotonic() - began
        out["code"] = bg.returncode
        out["stdout"], out["stderr"] = stdout, stderr
        try:
            out["session"] = json.loads(stdout)
        except ValueError:
            out["session"] = None
    finally:
        if bg.poll() is None:
            driver.kill(bg.pid)
    out["seen"] |= {m["pid"] for m in (out["session"] or {}).get("machines", []) if m.get("pid")}
    out["left"] = gone_after(out["seen"], 10)
    ctx.log(f"M16 {setup}: exit {out['code']} in {out['took']:.1f} s, p2 {out['p2'] and out['p2'].get('pid')}, "
            f"reason {str((out['session'] or {}).get('reason'))[:1500]}; stderr {out['stderr'][-500:]!r}")
    return out


def iso_drive(ctx, src, iso):
    """Builds an ISO image of src and mounts it; returns its drive letter."""
    script = ctx.work / "m16-mkiso.ps1"
    script.write_text(MKISO, encoding="utf-8")
    if iso.exists():
        iso.unlink()
    subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(script),
                    "-Src", str(src), "-Out", str(iso)], capture_output=True, text=True, timeout=120, check=True)
    p = subprocess.run(["powershell", "-NoProfile", "-Command",
                        f"(Mount-DiskImage -ImagePath '{iso}' -PassThru | Get-Volume).DriveLetter"],
                       capture_output=True, text=True, timeout=120)
    letter = p.stdout.strip()
    assert re.fullmatch(r"[A-Z]", letter), f"the ISO image mounted with no drive letter: {p.stdout!r} {p.stderr[:300]!r}"
    return letter


def unmount(iso):
    subprocess.run(["powershell", "-NoProfile", "-Command", f"Dismount-DiskImage -ImagePath '{iso}' | Out-Null"],
                   capture_output=True, text=True, timeout=120)


# ================================================================ the parts

def refused_part(ctx):
    root, sh = project(ctx)
    need_quiet(sh)
    out = {"before": driver.game_pids()}
    held = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    held.bind(("0.0.0.0", HELD_PORT))
    try:
        procs = Procs()
        try:
            out["answer"] = sh("session", "start", "--setup", "gap", ceiling=120)
            out["status"] = status_of(sh)
            with Mcp(root, ctx.install, ctx.log) as m:
                out["tool"] = m.call("session_start", {"setup": "gap"}, ceiling=70)[0]
                out["tool_status"] = structured(m.call("status")[0])
        finally:
            procs.stop()
        out["procs"] = procs
    finally:
        held.close()
    ctx.log(f"M16 refused: {out['answer']}: {message(out['answer'])}")
    # ENV-01, CFG-01: version without patch.
    bad = ctx.work / "m16bad"
    if bad.exists():
        shutil.rmtree(bad)
    shutil.copytree(ACC / "m16", bad / "tests")
    game = game_folder(ctx.project)
    text = (f"game = '{game}'\n\n[mods.fixture]\npath = '{game / '@skd_fixture'}'\nversion = \"0.0.1\"\n\n"
            "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n")
    (bad / "shakedown.toml").write_text(text, encoding="utf-8")
    out["nopatch_line"] = text.splitlines().index('version = "0.0.1"') + 1
    out["nopatch"] = driver.Shakedown(ctx.sh.exe, bad, ctx.log)("session", "start", "--setup", "sp", ceiling=120)
    out["after"] = driver.game_pids()
    tidy(ctx, sh, (out["after"] - out["before"]) | out["procs"].pids())
    return out


def disk_part(ctx):
    _, sh = project(ctx)
    need_quiet(sh)
    src = ctx.work / "m16disk-src"
    if src.exists():
        shutil.rmtree(src)
    shutil.copytree(ACC / "m16", src / "tests")
    mcp_json(src)
    game = game_folder(ctx.project)
    (src / "shakedown.toml").write_text(
        f"game = '{game}'\n\n[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n\n"
        "[setups.sp]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n", encoding="utf-8")
    iso = ctx.work / "m16disk.iso"
    out = {"before": driver.game_pids()}
    letter = iso_drive(ctx, src, iso)
    out["letter"] = letter
    try:
        root = Path(f"{letter}:\\")
        dsh = driver.Shakedown(ctx.sh.exe, root, ctx.log)
        procs = Procs()
        try:
            out["answer"] = dsh("session", "start", "--setup", "sp", ceiling=120)
            out["status"] = status_of(dsh)
            with Mcp(root, ctx.install, ctx.log) as m:
                out["tool"] = m.call("session_start", {"setup": "sp"}, ceiling=70)[0]
        finally:
            procs.stop()
        out["procs"] = procs
        ctx.log(f"M16 disk: {out['answer']}: {message(out['answer'])}")
    finally:
        unmount(iso)
        tidy(ctx, sh, (driver.game_pids() - out["before"]))
    return out


def version_part(ctx):
    root, sh = project(ctx)
    need_quiet(sh)
    pids, out = set(), {}
    try:
        # No version key: ready, and the result file records the version p1 loaded.
        out["nokey"], s = start(sh, "sp", pids)
        out["nokey_session"] = s
        assert out["nokey"].code == 0 and s and s["state"] == "ready", \
            f"{out['nokey']}: {s and s.get('state')} {str(s and s.get('reason'))[:500]}"
        out["nokey_run"] = run_check(sh)
        mods = {m["name"]: {x["mod"]: x for x in m["mods"]} for m in out["nokey_run"]["result"]["session"]["machines"]}
        out["found"] = mods["p1"]["fixture"]["version"]
        out["nokey_stop"] = sh("session", "stop", "--json", ceiling=180)
        out["nokey_json"] = session_json(root, s["id"])
        # The wrong version in singleplayer.
        project(ctx, WANT)
        before = driver.game_pids()
        out["bad"], out["bad_session"] = start(sh, "sp", pids)
        out["bad_left"] = gone_after((driver.game_pids() - before) | pids, 10)
        out["bad_status"] = status_of(sh)
        sid = (out["bad_session"] or {}).get("id")
        out["bad_json"] = session_json(root, sid) if sid else None
        out["bad_stop"] = sh("session", "stop", "--json", ceiling=180)
        out["bad_status_stopped"] = status_of(sh)
        # The version found: ready again.
        project(ctx, out["found"])
        out["good"], s = start(sh, "sp", pids)
        out["good_session"] = s
        if out["good"].code == 0:
            out["good_run"] = run_check(sh)
        out["good_stop"] = sh("session", "stop", "--json", ceiling=180)
        # The wrong version on a dedicated server with two clients.
        project(ctx, WANT)
        before = driver.game_pids()
        procs = Procs()
        try:
            out["ded"], out["ded_session"] = start(sh, "ded", pids)
            out["ded_left"] = gone_after((driver.game_pids() - before) | pids, 10)
        finally:
            procs.stop()
        pids |= procs.pids()
        out["ded_seen"] = procs.pids()
        sid = (out["ded_session"] or {}).get("id")
        out["ded_json"] = session_json(root, sid) if sid else None
        out["ded_stop"] = sh("session", "stop", "--json", ceiling=180)
        ctx.log(f"M16 version: found {out['found']}; sp {str((out['bad_session'] or {}).get('reason'))[:800]}; "
                f"ded {str((out['ded_session'] or {}).get('reason'))[:800]}")
    finally:
        project(ctx)
        tidy(ctx, sh, pids)
    return out


def hung_part(ctx):
    root, sh = project(ctx)
    need_quiet(sh)
    suspended = []

    def act(p2, began):
        win.suspend(p2["pid"])
        suspended.append(p2["pid"])
        ctx.log(f"M16 hung: suspended p2, pid {p2['pid']}, state {p2['state']}, by its id")
        return {"at": time.monotonic() - began, "state": p2["state"]}

    out = {}
    try:
        out = watched_start(ctx, sh, "hung", act)
        p2 = out["p2"] or {}
        out["rpt_now"] = rpt_lines(p2.get("profile"))
        out["status"] = status_of(sh)
        sid = (out["session"] or {}).get("id")
        out["json_failed"] = session_json(root, sid) if sid else None
        out["stop"] = sh("session", "stop", "--json", ceiling=180)
        out["status_stopped"] = status_of(sh)
        out["json_stopped"] = session_json(root, sid) if sid else None
    finally:
        tidy(ctx, sh, out.get("seen", set()) | set(suspended), suspended)
    return out


def exit_part(ctx):
    root, sh = project(ctx)
    need_quiet(sh)

    def act(p2, began):
        poll(lambda: len(rpt_lines(p2.get("profile"))) >= 25, 60, 0.5)
        n = len(rpt_lines(p2.get("profile")))
        driver.kill(p2["pid"])
        ctx.log(f"M16 exit: ended p2, pid {p2['pid']}, by its id with {n} lines in its RPT")
        return {"at": time.monotonic() - began, "lines": n}

    out = {}
    try:
        out = watched_start(ctx, sh, "ded", act)
        sid = (out["session"] or {}).get("id")
        out["json_failed"] = session_json(root, sid) if sid else None
        out["stop"] = sh("session", "stop", "--json", ceiling=180)
        out["json_stopped"] = session_json(root, sid) if sid else None
    finally:
        tidy(ctx, sh, out.get("seen", set()))
    return out


def part(ctx, key):
    return once(ctx, key, {"refused": refused_part, "disk": disk_part, "version": version_part,
                           "hung": hung_part, "exit": exit_part}[key])


# ================================================================ judging

def no_game_started(procs, before, after, what):
    seen = procs.pids() - before
    assert not seen, f"{what}: game processes started: {procs.describe()}"
    assert not (after - before), f"{what}: game processes run after it: {sorted(after - before)}"


def same_reason(printed, kept, what):
    assert kept is not None, f"{what}: the session folder holds no session.json"
    assert kept.get("state") == "failed", f"{what}: session.json says state {kept.get('state')}, expected failed"
    r = kept.get("reason")
    assert isinstance(r, dict), f"{what}: session.json's reason is {r!r}, expected the reason session start printed"
    for key in ("message", "machine", "rpt_tail"):
        assert r.get(key) == printed.get(key), \
            f"{what}: session.json's reason.{key} {str(r.get(key))[:400]!r} is not the printed {str(printed.get(key))[:400]!r}"
    assert not STEAM_ID.search(json.dumps(kept)), f"{what}: session.json holds a Steam ID"


def failed_with(o, what):
    assert o["code"] == 4, f"{what}: session start exited {o['code']}, expected 4: {o['stderr'][-400:]}"
    s = o["session"]
    assert s and s.get("state") == "failed", f"{what}: the start ended {s and s.get('state')}: {o['stdout'][:400]}"
    return s["reason"] or {}


# ---------------------------------------------------------------- US-085

@check("US-085 SES-03 ENV-01 a dedicated setup with a mod folder missing and one of its five ports held is refused with exit 4 before any game process starts, in one message naming the folder of the mod gone and the held port, and naming no Steam gap while Steam runs logged in", "start", repeat=True)
def us085_gaps(ctx):
    r = part(ctx, "refused")
    a = r["answer"]
    said = message(a)
    assert a.code == 4, f"{a}, expected exit 4: {said[:500]}"
    folder = str(ctx.work / "m16-no-such-mod")
    assert folder in said and re.search(r"\bgone\b", said), f"the message does not name the mod gone and {folder}: {said}"
    assert re.search(rf"\b{HELD_PORT}\b", said), f"the message does not name the held port {HELD_PORT}: {said}"
    assert not re.search(r"\bSteam\b", said), f"the message names a Steam gap while Steam runs logged in: {said}"
    no_game_started(r["procs"], r["before"], r["after"], "the refused start")
    assert r["status"]["session"] is None, f"status after the refusal shows a session: {r['status']['session']}"


@check("US-085 SES-03 ENV-01 AGT-02 session_start of the setup with the missing mod folder and the held port returns precondition-failed with the same text as session start, and status shows no session", "start", repeat=True)
def us085_tool(ctx):
    r = part(ctx, "refused")
    text = tool_error(r["tool"], "precondition-failed")
    said = message(r["answer"])
    assert text[len("precondition-failed: "):].strip() == said.strip(), \
        f"session_start's text is not the command's message: {text!r} vs {said!r}"
    assert r["tool_status"]["session"] is None, f"status after the tool's refusal shows a session: {r['tool_status']['session']}"


@check("US-085 ENV-01 a project on a drive with under 2 GB free is refused with exit 4 before any game process starts, naming the drive and its free space, and session_start returns precondition-failed with the same text", "start", repeat=True)
def us085_disk(ctx):
    d = part(ctx, "disk")
    a = d["answer"]
    said = message(a)
    assert a.code == 4, f"{a}, expected exit 4: {said[:500]}"
    assert re.search(rf"\b{d['letter']}:", said), f"the message does not name the drive {d['letter']}: {said}"
    assert re.search(r"\b2 GB\b", said) and re.search(r"\b0(\.0+)? GB free\b", said), \
        f"the message does not say 2 GB is needed and the drive has 0 GB free: {said}"
    assert not (d["procs"].pids() - d["before"]), f"game processes started: {d['procs'].describe()}"
    assert d["status"]["session"] is None, f"status after the refusal shows a session: {d['status']['session']}"
    text = tool_error(d["tool"], "precondition-failed")
    assert text[len("precondition-failed: "):].strip() == said.strip(), \
        f"session_start's text is not the command's message: {text!r} vs {said!r}"


@check("ENV-01 CFG-01 a mod naming version without patch makes session start exit 3 naming version and its line, and starts no game", "start", repeat=True)
def env01_no_patch(ctx):
    r = part(ctx, "refused")
    a = r["nopatch"]
    said = message(a)
    assert a.code == 3, f"{a}, expected exit 3: {said[:400]}"
    assert re.search(r"\bversion\b", said), f"the message does not name the key version: {said}"
    assert "line" in said and re.search(rf"\b{r['nopatch_line']}\b", said), \
        f"the message does not name the line {r['nopatch_line']}: {said}"
    assert not (r["after"] - r["before"]), f"game processes run after it: {sorted(r['after'] - r['before'])}"


# ---------------------------------------------------------------- US-086

def wrong_version(ctx, key, what):
    v = part(ctx, "version")
    a, s = v[key], v[f"{key}_session"]
    assert a.code == 4, f"{what}: {a}, expected exit 4: {a.stdout[:300]} {a.stderr[-300:]}"
    assert s and s.get("state") == "failed", f"{what}: the start ended {s and s.get('state')}"
    reason = s.get("reason") or {}
    msg = reason.get("message", "")
    assert re.search(r"\bfixture\b", msg), f"{what}: the message does not name the mod fixture: {msg!r}"
    assert v["found"] in msg, f"{what}: the message does not name the version found {v['found']}: {msg!r}"
    assert WANT in msg, f"{what}: the message does not name the version wanted {WANT}: {msg!r}"
    machine = reason.get("machine")
    assert machine and re.search(rf"\b{machine}\b", msg), f"{what}: the message does not name its machine {machine}: {msg!r}"
    assert not v[f"{key}_left"], f"{what}: game processes {sorted(v[f'{key}_left'])} still ran 10 s after the start returned"
    same_reason(reason, v[f"{key}_json"], what)
    return v, reason


@check("US-086 ENV-01 INT-STM-03 with the fixture's version set to 9.9.9 a singleplayer session start exits 4 ended failed, naming the mod fixture, p1, the version the result file records and 9.9.9, no game process runs 10 s after it, and session.json keeps the reason", "start", repeat=True)
def us086_sp(ctx):
    v, reason = wrong_version(ctx, "bad", "singleplayer")
    assert reason.get("machine") == "p1", f"reason.machine {reason.get('machine')}"
    st = v["bad_status"].get("session") or {}
    assert st.get("state") == "failed" and (st.get("reason") or {}).get("message") == reason.get("message"), \
        f"status after the failed start: {st.get('state')} {str(st.get('reason'))[:300]}"
    assert v["bad_stop"].code == 0 and v["bad_status_stopped"].get("session") is None, \
        f"session stop after the failed start: {v['bad_stop']}; status {v['bad_status_stopped'].get('session')}"


@check("US-086 ENV-01 SES-04 with the fixture's version set to 9.9.9 a dedicated session start with two clients exits 4 ended failed, naming the mod fixture, a machine, the version found and 9.9.9, and no process of the start runs 10 s after it", "start", repeat=True)
def us086_ded(ctx):
    v, reason = wrong_version(ctx, "ded", "dedicated")
    assert len(v["ded_seen"]) >= 3, f"the start did not launch its three machines before it failed: {sorted(v['ded_seen'])}"


@check("US-086 ENV-01 with no version key and with the fixture's version set to the one the result file records, a singleplayer session start is ready and the check run in it passes", "start", repeat=True)
def us086_right(ctx):
    v = part(ctx, "version")
    for key in ("nokey", "good"):
        a, s = v[key], v[f"{key}_session"]
        assert a.code == 0 and s and s.get("state") == "ready", \
            f"{key}: {a}: {s and s.get('state')} {str(s and s.get('reason'))[:400]}"
        r = v.get(f"{key}_run")
        assert r, f"{key}: no check ran"
        ex = [c for c in r["result"]["checks"] if c["check_id"] == CHECK]
        assert len(ex) == 1 and ex[0]["outcome"] == "pass", f"{key}: {CHECK} ended {ex and ex[0]['outcome']}"
    mods = {m["name"]: {x["mod"]: x for x in m["mods"]} for m in v["good_run"]["result"]["session"]["machines"]}
    assert mods["p1"]["fixture"]["version"] == v["found"], f"p1 records {mods['p1']['fixture']}"


# ---------------------------------------------------------------- US-087

@check("US-087 SES-04 SES-10 p2 suspended at its launch fails a dedicated start with start_timeout 150 at the ceiling with exit 4, reason.machine p2, the message naming p2 launched, its last RPT lines or that it wrote none, and no process of the start runs 10 s after it", "start", repeat=True)
def us087_hung(ctx):
    h = part(ctx, "hung")
    assert h["p2"], "status never listed p2 with a process id during the start"
    ctx.log(f"US-087: p2 suspended {h['acted']['at']:.1f} s into the start in state {h['acted']['state']}; "
            f"the start returned at {h['took']:.1f} s")
    reason = failed_with(h, "the hung start")
    assert 148 <= h["took"] <= 190, f"the start returned after {h['took']:.0f} s with a start ceiling of 150 s"
    msg = reason.get("message", "")
    assert reason.get("machine") == "p2", f"reason.machine {reason.get('machine')!r}: {msg!r}"
    assert re.search(r"\bp2\b[^;.]*\blaunched\b", msg), f"the message does not name p2 as launched: {msg!r}"
    tail = reason.get("rpt_tail")
    rpt = h["rpt_now"]
    if rpt:
        want = rpt[-20:]
        assert tail and [t.strip() for t in tail] == want[-len(tail):] and len(tail) == min(20, len(rpt)), \
            f"reason.rpt_tail {tail!r} is not the last lines of p2's RPT {want!r}"
    else:
        said = msg + " " + h["stdout"] + " " + h["stderr"]
        assert re.search(r"p2 wrote no RPT line", said) or tail == [], \
            f"p2 wrote no RPT line, and neither the reason nor the output says so: {reason!r}"
    assert not h["left"], f"game processes {sorted(h['left'])} still ran 10 s after the start returned (p2 {h['p2']['pid']})"


@check("US-087 INS-02 SES-10 DEF-29 after the hung start status shows it failed with the same reason, session stop exits 0 and clears it, and the session folder's session.json holds state failed and the reason session start printed, before and after the stop", "start", repeat=True)
def us087_kept(ctx):
    h = part(ctx, "hung")
    reason = failed_with(h, "the hung start")
    st = h["status"].get("session") or {}
    assert st.get("state") == "failed", f"status after the failed start: {st.get('state')}"
    assert (st.get("reason") or {}).get("message") == reason.get("message"), \
        f"status's reason {str(st.get('reason'))[:300]} is not the printed one {reason.get('message')!r}"
    assert h["stop"].code == 0 and h["status_stopped"].get("session") is None, \
        f"session stop: {h['stop']}; status {h['status_stopped'].get('session')}"
    same_reason(reason, h["json_failed"], "session.json after the start")
    same_reason(reason, h["json_stopped"], "session.json after session stop")


# ---------------------------------------------------------------- DEF-29

@check("DEF-29 SES-10 SES-04 a dedicated start that fails because p2's process was ended writes session.json with state failed and the reason session start printed: the message, machine p2 and its rpt_tail, with no Steam ID, before and after session stop", "start", repeat=True)
def def29_exit(ctx):
    e = part(ctx, "exit")
    assert e["p2"], "status never listed p2 with a process id during the start"
    reason = failed_with(e, "the start whose p2 was ended")
    assert reason.get("machine") == "p2", f"reason.machine {reason.get('machine')!r}: {reason.get('message')!r}"
    assert reason.get("rpt_tail"), f"reason.rpt_tail is {reason.get('rpt_tail')!r}"
    same_reason(reason, e["json_failed"], "session.json after the start")
    same_reason(reason, e["json_stopped"], "session.json after session stop")
    assert not e["left"], f"game processes {sorted(e['left'])} still ran 10 s after the start returned"


@check("DEF-29 SES-10 a singleplayer session stopped with session stop while ready writes session.json with state stopped and reason null", "start", repeat=True)
def def29_ready(ctx):
    v = part(ctx, "version")
    assert v["nokey_stop"].code == 0, f"{v['nokey_stop']}"
    j = v["nokey_json"]
    assert j and j.get("state") == "stopped" and "reason" in j and j["reason"] is None, f"session.json {j}"
