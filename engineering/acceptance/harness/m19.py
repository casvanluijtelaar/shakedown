r"""The checks of M19 The first public release: US-095 to US-098, DST-01 to DST-04, NFR-LG-04 and NFR-PT-01.

They judge a release zip built locally, `shakedown-windows-x64.zip` and its `.sha256` in the dist folder
(`--dist`, default: the `SHAKEDOWN_DIST` variable, else `target\dist\` of the repository), as `docs/install.md` says
`tools/package.py` writes them, with the `tools/install.ps1` a release carries beside them. No release on GitHub and
no winget package is driven: those are the owner's steps.

The `offline` checks read the zip, the root `README.md`, `LICENSE`, `docs/install.md` and the workspace's package
metadata from `cargo metadata`, and run `tools/package.py` with a wrong tag and `cargo check` of the extension and the
protocol crate for 64-bit Linux.

The `start` checks share one part, which runs once. Every command it starts runs in a "clean shell": the `PATH` a new
shell gets from the registry (the machine's, then the user's), with every folder that holds Rust, HEMTT or Python left
out. The user's registry `PATH` is saved before each install and put back exactly, with its value kind, after it, with
the change broadcast as Windows does; each install goes into a folder of the work folder, given by
`SHAKEDOWN_INSTALL_DIR`. In order:

1. `install.ps1` run as a file from a release folder (`SHAKEDOWN_DOWNLOAD`) into `file\Shakedown`; `shakedown version`
   in a new shell. Then two files of the install changed and the script run again (US-096). Then a release folder whose
   zip has one byte changed, run as a file over that install (US-097). Then `run --setup sp` of the README's quickstart
   check from that install. The user `PATH` is put back.
2. The one-line route, `irm http://127.0.0.1:<port>/install.ps1 | iex`, from an HTTP server inside the harness that
   serves the release folder, into `iex\Shakedown`; `shakedown version` in a new shell; `run --setup sp`. Put back.
3. A release folder whose zip is cut off at half, run as a file with no install into `cut\Shakedown` (US-097 2).
4. The zip unpacked by hand into `hand\Shakedown` with Python's `zipfile`, that folder put on the new shell's `PATH`;
   `shakedown version`; `run --setup sp`.

The US-098 checks share a part of their own: `install.ps1` run as a file into `run\Shakedown`, a session started from
that install and `shakedown mcp` started from it, the one-line route run over it (US-098 1), and once both are stopped,
the same command again (US-098 2). The user `PATH` is put back.

After each `run --setup sp` and the US-098 session, the part waits up to 75 s for the runner to exit by itself and stops
it by its id after that. A game start lost to `Error 20 reading file` starts again, at most three times in all.
"""

import ctypes
import functools
import hashlib
import http.server
import json
import os
import re
import shutil
import struct
import subprocess
import sys
import threading
import time
import traceback
import winreg
import zipfile
from pathlib import Path

from . import driver
from .core import check
from .m08 import game_folder
from .m13 import mcp_json
from .m17 import machine_fault
from .mcp import Mcp

ACC = Path(__file__).resolve().parent.parent
ZIP, SHA = "shakedown-windows-x64.zip", "shakedown-windows-x64.zip.sha256"
TOOLS = ("cargo.exe", "rustc.exe", "rustup.exe", "hemtt.exe", "python.exe", "python3.exe", "pythonw.exe", "py.exe")
POWERSHELL = Path(os.environ.get("SystemRoot", r"C:\Windows")) / "System32" / "WindowsPowerShell" / "v1.0" / "powershell.exe"
ROOT_FILES = {"shakedown.exe", "@shakedown/addons/shakedown_main.pbo", "@shakedown/shakedown_x64.dll", "LICENSE",
              "THIRD-PARTY-LICENSES.txt"}


def keep(ctx):
    if not hasattr(ctx, "m19_keep"):
        ctx.m19_keep = {}
    return ctx.m19_keep


def dist(ctx):
    d = Path(getattr(ctx, "dist", None) or ctx.repo / "target" / "dist")
    assert (d / ZIP).is_file() and (d / SHA).is_file(), \
        f"no {ZIP} with its .sha256 in {d}; run `python tools/package.py --tag v<version>` or give --dist"
    return d


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def metadata(ctx, *extra):
    out = subprocess.run(["cargo", "metadata", "--format-version", "1", *extra], cwd=ctx.repo, capture_output=True,
                         text=True, encoding="utf-8", errors="replace", timeout=300)
    assert out.returncode == 0, f"cargo metadata exit {out.returncode}: {out.stderr[-500:]}"
    return json.loads(out.stdout)


def version(ctx):
    store = keep(ctx)
    if "version" not in store:
        pkgs = {p["name"]: p for p in metadata(ctx, "--no-deps")["packages"]}
        store["version"] = pkgs["shakedown"]["version"]
    return store["version"]


def zip_names(path):
    with zipfile.ZipFile(path) as z:
        return [n.replace("\\", "/") for n in z.namelist()]


# --- the registry PATH and the clean shell -------------------------------------------------------------------

def user_path():
    with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment") as k:
        try:
            return winreg.QueryValueEx(k, "Path")
        except FileNotFoundError:
            return None


def machine_path():
    with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment") as k:
        return winreg.QueryValueEx(k, "Path")[0]


def broadcast():
    """Tells running programs the environment changed, as Windows' own Environment Variables dialog does."""
    result = ctypes.c_size_t()
    ctypes.windll.user32.SendMessageTimeoutW(0xFFFF, 0x001A, 0, "Environment", 0x0002, 5000, ctypes.byref(result))


class UserPath:
    """Saves the user's registry PATH, with its value kind, and puts it back exactly on exit."""

    def __init__(self, ctx):
        self.ctx = ctx

    def __enter__(self):
        self.saved = user_path()
        return self

    def __exit__(self, *exc):
        if user_path() != self.saved:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment", 0, winreg.KEY_SET_VALUE) as k:
                if self.saved is None:
                    winreg.DeleteValue(k, "Path")
                else:
                    winreg.SetValueEx(k, "Path", 0, self.saved[1], self.saved[0])
            broadcast()
        now = user_path()
        self.ctx.log(f"M19: the user PATH put back; it matches what was saved: {now == self.saved}")
        if now != self.saved:
            raise AssertionError("the user PATH could not be put back as it was")


def expand(text):
    return os.path.expandvars(text) if text else ""


def has_tool(folder):
    try:
        return any((Path(folder) / t).is_file() for t in TOOLS)
    except OSError:
        return False


def clean(entries):
    return [e for e in entries if e.strip() and not has_tool(expand(e))]


def new_shell_env(extra=()):
    """The environment of a new shell: PATH from the registry, machine then user, without Rust, HEMTT or Python."""
    up = user_path()
    entries = machine_path().split(";") + (up[0].split(";") if up else []) + list(extra)
    env = {k: v for k, v in os.environ.items() if not k.upper().startswith(("PYTHON", "CARGO", "RUSTUP", "SHAKEDOWN"))}
    env["PATH"] = ";".join(expand(e) for e in clean(entries))
    return env


def which(env, name):
    return shutil.which(name, path=env["PATH"])


def user_entries(folder):
    up = user_path()
    want = os.path.normcase(os.path.normpath(str(folder)))
    return [e for e in (up[0].split(";") if up else [])
            if e.strip() and os.path.normcase(os.path.normpath(expand(e).rstrip("\\/"))) == want]


def run_cmd(argv, env, cwd, ceiling, log):
    started = time.monotonic()
    p = subprocess.run([str(a) for a in argv], env=env, cwd=cwd, capture_output=True, text=True, encoding="utf-8",
                       errors="replace", timeout=ceiling)
    log(f"M19: `{' '.join(str(a) for a in argv)}` exit {p.returncode} in {time.monotonic() - started:.1f} s")
    return {"code": p.returncode, "out": p.stdout, "err": p.stderr, "text": p.stdout + "\n" + p.stderr}


def tree(folder):
    """{relative path: sha256} of every file under folder."""
    folder = Path(folder)
    if not folder.is_dir():
        return {}
    return {p.relative_to(folder).as_posix(): sha256(p) for p in sorted(folder.rglob("*")) if p.is_file()}


def zip_tree(path):
    out = {}
    with zipfile.ZipFile(path) as z:
        for i in z.infolist():
            if not i.is_dir():
                out[i.filename.replace("\\", "/")] = hashlib.sha256(z.read(i)).hexdigest()
    return out


def shakedown_procs(folder):
    """{pid: command line} of every shakedown.exe whose executable is in folder."""
    out = subprocess.run([str(POWERSHELL), "-NoProfile", "-Command",
                          "@(Get-CimInstance Win32_Process -Filter \"Name='shakedown.exe'\" | "
                          "Select-Object ProcessId,ExecutablePath,CommandLine) | ConvertTo-Json -Compress"],
                         capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=120).stdout.strip()
    rows = json.loads(out) if out else []
    rows = [rows] if isinstance(rows, dict) else rows
    want = os.path.normcase(os.path.normpath(str(folder)))
    return {r["ProcessId"]: (r["CommandLine"] or "") for r in rows
            if r.get("ExecutablePath") and os.path.normcase(os.path.dirname(r["ExecutablePath"])) == want}


class Server:
    """An HTTP server inside the harness serving one folder on 127.0.0.1, stopped on exit."""

    def __init__(self, folder):
        handler = functools.partial(Quiet, directory=str(folder))
        self.httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        self.url = f"http://127.0.0.1:{self.httpd.server_address[1]}"
        self.thread = threading.Thread(target=self.httpd.serve_forever, daemon=True)

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *exc):
        self.httpd.shutdown()
        self.httpd.server_close()
        self.thread.join(timeout=30)


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass


# --- the part ------------------------------------------------------------------------------------------------

def release_folder(ctx, name, damage=None):
    """A folder as a GitHub release holds it: the zip, its .sha256 and install.ps1. damage changes the zip's bytes."""
    src = dist(ctx)
    folder = ctx.work / "m19" / "release" / name
    if folder.exists():
        shutil.rmtree(folder)
    folder.mkdir(parents=True)
    shutil.copyfile(src / SHA, folder / SHA)
    shutil.copyfile(ctx.repo / "tools" / "install.ps1", folder / "install.ps1")
    data = (src / ZIP).read_bytes()
    if damage == "corrupt":
        i = len(data) // 2
        data = data[:i] + bytes([data[i] ^ 0xFF]) + data[i + 1:]
    elif damage == "cut":
        data = data[:len(data) // 2]
    (folder / ZIP).write_bytes(data)
    return folder


def install_env(release, target):
    env = new_shell_env()
    env["SHAKEDOWN_DOWNLOAD"] = str(release)
    env["SHAKEDOWN_INSTALL_DIR"] = str(target)
    return env


def as_file(ctx, release, target):
    return run_cmd([POWERSHELL, "-NoProfile", "-File", release / "install.ps1"], install_env(release, target),
                   ctx.work, 300, ctx.log)


def one_line(ctx, server, target):
    env = install_env(server.url, target)
    return run_cmd([POWERSHELL, "-NoProfile", "-Command", f"irm {server.url}/install.ps1 | iex"], env, ctx.work, 300,
                   ctx.log)


def shell_version(ctx, extra=()):
    env = new_shell_env(extra)
    found = which(env, "shakedown")
    out = run_cmd(["cmd", "/c", "shakedown", "version"], env, ctx.work, 60, ctx.log)
    out["found"] = found
    return out


def project(ctx):
    store = keep(ctx)
    if "project" not in store:
        root = ctx.work / "m19" / "proj"
        if root.exists():
            shutil.rmtree(root)
        (root / "tests").mkdir(parents=True)
        shutil.copyfile(ACC / "m19" / "smoke.sqf", root / "tests" / "smoke.sqf")
        mcp_json(root)
        game = game_folder(ctx.project)
        (root / "shakedown.toml").write_text(
            f"game = '{game}'\ntests = \"tests\"\n\n[setups.sp]\nmode = \"singleplayer\"\nmods = []\n", encoding="utf-8")
        store["project"] = root
    return store["project"]


def steam_running():
    return bool(driver.pids("steam.exe"))


def wait_runner(ctx, before=None):
    """Waits up to 75 s for the runner runner.json names to exit by itself, then stops it by its id."""
    rp = driver.runner_pid()
    if rp and rp != before and driver.alive(rp):
        took = driver.wait_gone(rp, 75)
        if took is None:
            ctx.log(f"M19: the runner {rp} still runs 75 s after its last session; stopping it by id")
            driver.kill(rp)
        else:
            ctx.log(f"M19: the runner {rp} exited by itself {took:.0f} s later")


def game_run(ctx, folder, extra=()):
    """`shakedown run --setup sp --json` from the shakedown the new shell finds, in the M19 project."""
    root = project(ctx)
    env = new_shell_env(extra)
    exe = which(env, "shakedown")
    out = {"exe": exe}
    for attempt in range(1, 4):
        assert steam_running(), "no steam process runs; a game start would be refused"
        assert not driver.game_pids(), f"game processes run before the M19 run: {sorted(driver.game_pids())}"
        began = time.time()
        before = set(driver.game_pids())
        try:
            r = run_cmd(["cmd", "/c", "shakedown", "run", "--setup", "sp", "--json"], env, root, 600, ctx.log)
        finally:
            for pid in driver.game_pids() - before:
                if driver.wait_gone(pid, 30) is None:
                    driver.kill(pid)
        out["run"] = r
        try:
            out["json"] = json.loads(r["out"])
            out["result"] = json.loads(Path(out["json"]["result_file"]).read_text(encoding="utf-8"))
        except (ValueError, KeyError, OSError, TypeError):
            out["json"] = out["result"] = None
        wait_runner(ctx)
        fault = machine_fault(root, began)
        if r["code"] == 0 or not fault or attempt == 3:
            break
        ctx.log(f"M19: a game process could not read a base-game file ({fault}); starting again, attempt {attempt + 1}")
    return out


def us098(ctx, server, target):
    """A session and `shakedown mcp` run from target while the one-line route runs over it; then both stopped."""
    root = project(ctx)
    env = new_shell_env()
    exe = which(env, "shakedown")
    out = {}
    pids = set()
    mcp = None
    try:
        for attempt in range(1, 4):
            assert steam_running(), "no steam process runs; a game start would be refused"
            began = time.time()
            start = run_cmd(["cmd", "/c", "shakedown", "session", "start", "--setup", "sp", "--json"], env, root, 400,
                            ctx.log)
            try:
                pids |= {m["pid"] for m in json.loads(start["out"]).get("machines", []) if m.get("pid")}
            except (ValueError, AttributeError):
                pass
            if start["code"] == 0 or not machine_fault(root, began) or attempt == 3:
                break
            run_cmd(["cmd", "/c", "shakedown", "session", "stop"], env, root, 180, ctx.log)
        out["start"] = start
        assert start["code"] == 0, f"session start from {target} exit {start['code']}: {start['text'][:600]}"
        mcp = Mcp(root, target, ctx.log)
        mcp.initialize()
        out["procs"] = shakedown_procs(target)
        out["runner"] = driver.runner_pid()
        out["mcp_pid"] = mcp.pid
        out["before"] = tree(target)
        out["refused"] = one_line(ctx, server, target)
        out["after"] = tree(target)
        out["procs_after"] = shakedown_procs(target)
        st = run_cmd(["cmd", "/c", "shakedown", "status", "--json"], env, root, 60, ctx.log)
        out["status"] = json.loads(st["out"]) if st["code"] == 0 else st
        run_cmd(["cmd", "/c", "shakedown", "session", "stop", "--json"], env, root, 180, ctx.log)
        mcp.close()
        mcp = None
        wait_runner(ctx)
        for _ in range(30):
            if not shakedown_procs(target):
                break
            time.sleep(1)
        out["left"] = shakedown_procs(target)
        out["upgrade"] = one_line(ctx, server, target)
        out["upgraded"] = tree(target)
        out["version"] = shell_version(ctx)
    finally:
        if mcp is not None:
            mcp.close()
        try:
            run_cmd(["cmd", "/c", "shakedown", "session", "stop"], env, root, 180, ctx.log) if exe else None
        except Exception as e:  # noqa: BLE001
            ctx.log(f"M19: session stop after US-098: {e!r}")
        for pid in pids:
            if driver.alive(pid) and driver.wait_gone(pid, 30) is None:
                driver.kill(pid)
        wait_runner(ctx)
    return out


def part(ctx):
    v = {}
    base = ctx.work / "m19"
    for name in ("file", "iex", "cut", "hand"):
        if (base / name).exists():
            shutil.rmtree(base / name)
    env = new_shell_env()
    v["clean"] = {t: which(env, t) for t in ("cargo", "rustc", "hemtt", "python", "python3", "py")}
    v["path_before"] = user_path()
    good = release_folder(ctx, "good")
    v["zip"] = zip_tree(good / ZIP)

    # 1. as a file, again, a corrupt zip over it, a run, US-098
    a = base / "file" / "Shakedown"
    with UserPath(ctx):
        v["file"] = as_file(ctx, good, a)
        v["file_entries"] = len(user_entries(a))
        v["file_tree"] = tree(a)
        v["file_version"] = shell_version(ctx)
        changed = a / "LICENSE"
        if changed.is_file():
            changed.write_text(changed.read_text(encoding="utf-8") + "\nchanged by the acceptance check\n",
                               encoding="utf-8")
        (a / "THIRD-PARTY-LICENSES.txt").unlink(missing_ok=True)
        v["again"] = as_file(ctx, good, a)
        v["again_entries"] = len(user_entries(a))
        v["again_tree"] = tree(a)
        v["again_version"] = shell_version(ctx)
        bad = release_folder(ctx, "corrupt", "corrupt")
        v["corrupt_hash"] = sha256(bad / ZIP)
        v["corrupt_before"] = tree(a)
        path_before = user_path()
        v["corrupt"] = as_file(ctx, bad, a)
        v["corrupt_after"] = tree(a)
        v["corrupt_path_same"] = user_path() == path_before
        v["corrupt_version"] = shell_version(ctx)
        try:
            v["file_run"] = game_run(ctx, a)
        except Exception as e:  # noqa: BLE001
            ctx.log(f"M19 file run: {traceback.format_exc()}")
            v["file_run"] = e
    v["path_after_file"] = user_path()

    # 2. the one-line route
    b = base / "iex" / "Shakedown"
    with UserPath(ctx):
        with Server(good) as server:
            v["iex"] = one_line(ctx, server, b)
        v["iex_entries"] = len(user_entries(b))
        v["iex_tree"] = tree(b)
        v["iex_version"] = shell_version(ctx)
        try:
            v["iex_run"] = game_run(ctx, b)
        except Exception as e:  # noqa: BLE001
            ctx.log(f"M19 iex run: {traceback.format_exc()}")
            v["iex_run"] = e

    # 3. a cut-off zip with nothing installed
    c = base / "cut" / "Shakedown"
    cut = release_folder(ctx, "cut", "cut")
    v["cut_hash"] = sha256(cut / ZIP)
    with UserPath(ctx):
        path_before = user_path()
        v["cut"] = as_file(ctx, cut, c)
        v["cut_tree"] = tree(c)
        v["cut_exists"] = c.exists()
        v["cut_path_same"] = user_path() == path_before

    # 4. by hand
    d = base / "hand" / "Shakedown"
    d.mkdir(parents=True)
    with zipfile.ZipFile(good / ZIP) as z:
        z.extractall(d)
    v["hand_version"] = shell_version(ctx, [str(d)])
    try:
        v["hand_run"] = game_run(ctx, d, [str(d)])
    except Exception as e:  # noqa: BLE001
        ctx.log(f"M19 hand run: {traceback.format_exc()}")
        v["hand_run"] = e

    v["path_end"] = user_path()
    v["folders"] = {"file": a, "iex": b, "cut": c, "hand": d}
    return v


def part098(ctx):
    """US-098 in a part of its own: install.ps1 run as a file into `run\\Shakedown`, then `us098` over it."""
    target = ctx.work / "m19" / "run" / "Shakedown"
    if target.parent.exists():
        shutil.rmtree(target.parent)
    good = release_folder(ctx, "good")
    v = {"zip": zip_tree(good / ZIP), "folders": {"run": target}, "path_before": user_path()}
    with UserPath(ctx):
        r = as_file(ctx, good, target)
        assert r["code"] == 0, f"install.ps1 into {target} exit {r['code']}: {r['text'][-600:]}"
        with Server(good) as server:
            v["us098"] = us098(ctx, server, target)
    v["path_end"] = user_path()
    return v


def once(ctx, key="part", fn=None):
    store = keep(ctx)
    if key not in store:
        try:
            store[key] = (fn or part)(ctx)
        except Exception as e:  # noqa: BLE001
            ctx.log(f"M19 {key}: {traceback.format_exc()}")
            store[key] = e
    value = store[key]
    if isinstance(value, Exception):
        raise AssertionError(f"the M19 part failed: {value!r}")
    return value


# --- judging -------------------------------------------------------------------------------------------------

def same(a, b):
    return os.path.normcase(os.path.normpath(str(a))) == os.path.normcase(os.path.normpath(str(b)))


def installed(ctx, v, key, folder):
    r = v[key]
    assert r["code"] == 0, f"install.ps1 exit {r['code']}: {r['text'][-800:]}"
    t = v[f"{key}_tree"]
    assert t == v["zip"], (f"the install folder {folder} does not hold the zip's files: missing "
                           f"{sorted(set(v['zip']) - set(t))}, extra {sorted(set(t) - set(v['zip']))}, changed "
                           f"{sorted(k for k in t if k in v['zip'] and t[k] != v['zip'][k])}")


def printed_version(ctx, r, folder):
    want = version(ctx)
    assert r["found"] and same(Path(r["found"]).parent, folder), \
        f"a new shell finds shakedown at {r['found']}, not in {folder}"
    assert r["code"] == 0 and re.search(rf"(?<![\d.]){re.escape(want)}(?![\d.])", r["out"]), \
        f"`shakedown version` in a new shell exit {r['code']}, printed {r['text'].strip()[:300]!r}; expected {want}"


def passed(ctx, v, key, folder):
    r = v[key]
    if isinstance(r, Exception):
        raise AssertionError(f"the run failed: {r!r}")
    assert r["exe"] and same(Path(r["exe"]).parent, folder), f"the new shell's shakedown is {r['exe']}, not in {folder}"
    run = r["run"]
    assert run["code"] == 0, f"`shakedown run --setup sp` exit {run['code']}: {run['text'][-800:]}"
    res = r["result"]
    assert res, f"no result file: {run['out'][:400]}"
    outcomes = [(c["check_id"], c["outcome"]) for c in res["checks"]]
    assert outcomes == [("smoke", "pass")], f"the run's outcomes: {outcomes}"


def refused(r, hashes):
    text = r["text"]
    low = text.lower()
    for h in hashes:
        assert h.lower() in low, f"the refusal does not name the hash {h}: {text.strip()[-600:]!r}"
    assert "nothing" in low, f"the refusal does not say nothing was installed: {text.strip()[-600:]!r}"


def squash(text):
    """The text with every space, line break and quote left out: PowerShell wraps an error record at the width of
    its host, also when its output is redirected, so a long command runs over several lines."""
    return re.sub(r"[\s\"']", "", text)


def sha_value(ctx):
    return (dist(ctx) / SHA).read_text(encoding="utf-8").split()[0]


def m19check(name, phase="start"):
    return check(name, phase)


@m19check("US-095 DST-01 NFR-LG-04 the release zip holds at its root shakedown.exe, @shakedown with addons\\shakedown_main.pbo and shakedown_x64.dll, LICENSE and THIRD-PARTY-LICENSES.txt, and nothing else, no fixture", "offline")
def zip_contents(ctx):
    names = [n for n in zip_names(dist(ctx) / ZIP) if not n.endswith("/")]
    ctx.log(f"M19: the zip holds {names}")
    assert set(names) == ROOT_FILES, \
        f"the zip holds {sorted(names)}; missing {sorted(ROOT_FILES - set(names))}, extra {sorted(set(names) - ROOT_FILES)}"
    assert len(names) == len(set(names)), f"the zip holds a name twice: {names}"


@m19check("US-095 DST-01 the zip's .sha256 is one line of its SHA-256 in 64 lowercase hex digits, two spaces and the zip's name", "offline")
def zip_sha(ctx):
    raw = (dist(ctx) / SHA).read_bytes().decode("utf-8")
    lines = raw.splitlines()
    assert len(lines) == 1, f"the .sha256 holds {len(lines)} lines: {raw!r}"
    m = re.fullmatch(r"([0-9a-f]{64})  (\S+)", lines[0])
    assert m, f"the .sha256 line is not 64 lowercase hex digits, two spaces and a name: {lines[0]!r}"
    assert m.group(2) == ZIP, f"the .sha256 names {m.group(2)!r}"
    assert m.group(1) == sha256(dist(ctx) / ZIP), "the .sha256 differs from the zip's SHA-256"


@m19check("NFR-LG-04 LICENSE holds the GPL version 2 text with the grant of later versions, the zip's LICENSE is the repository's, and every crate of the workspace declares GPL-2.0-or-later", "offline")
def licence(ctx):
    text = (ctx.repo / "LICENSE").read_text(encoding="utf-8")
    for want in ("GNU GENERAL PUBLIC LICENSE", "Version 2, June 1991", "either version 2 of the License, or",
                 "(at your option) any later version", "END OF TERMS AND CONDITIONS"):
        assert want in text, f"LICENSE does not hold {want!r}"
    with zipfile.ZipFile(dist(ctx) / ZIP) as z:
        inzip = z.read("LICENSE").decode("utf-8")
    assert inzip.replace("\r\n", "\n") == text.replace("\r\n", "\n"), "the zip's LICENSE differs from the repository's"
    meta = metadata(ctx, "--no-deps")
    wrong = {p["name"]: p.get("license") for p in meta["packages"] if p.get("license") != "GPL-2.0-or-later"}
    ctx.log(f"M19: the workspace's crates: {sorted(p['name'] for p in meta['packages'])}")
    assert not wrong, f"crates that do not declare GPL-2.0-or-later: {wrong}"


@m19check("NFR-LG-04 THIRD-PARTY-LICENSES.txt names every package compiled into shakedown.exe or shakedown_x64.dll, rmcp with the Apache-2.0 licence text among them", "offline")
def third_party(ctx):
    with zipfile.ZipFile(dist(ctx) / ZIP) as z:
        text = z.read("THIRD-PARTY-LICENSES.txt").decode("utf-8", "replace")
    meta = metadata(ctx, "--filter-platform", "x86_64-pc-windows-msvc")
    pkgs = {p["id"]: p for p in meta["packages"]}
    members = set(meta["workspace_members"])
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    roots = [i for i in members if pkgs[i]["name"] in ("shakedown", "shakedown-ext")]
    seen, todo = set(), list(roots)
    while todo:
        node = nodes[todo.pop()]
        for dep in node["deps"]:
            if not any(k["kind"] is None for k in dep["dep_kinds"]):
                continue
            p = pkgs[dep["pkg"]]
            if any("proc-macro" in t["kind"] for t in p["targets"]) or dep["pkg"] in seen:
                continue
            seen.add(dep["pkg"])
            todo.append(dep["pkg"])
    names = sorted({pkgs[i]["name"] for i in seen if i not in members})
    missing = [n for n in names if not re.search(rf"(?<![\w-]){re.escape(n)}(?![\w-])", text)]
    ctx.log(f"M19: {len(names)} packages compiled in; THIRD-PARTY-LICENSES.txt is {len(text)} characters")
    assert not missing, f"THIRD-PARTY-LICENSES.txt does not name {len(missing)} of {len(names)} packages: {missing[:20]}"
    assert "rmcp" in names, f"rmcp is not among the packages compiled in: {names}"
    assert "Apache License" in text and "Version 2.0" in text, "THIRD-PARTY-LICENSES.txt holds no Apache-2.0 text"


def pe_machine(data):
    off = struct.unpack_from("<I", data, 0x3C)[0]
    assert data[off:off + 4] == b"PE\0\0", "not a PE file"
    return struct.unpack_from("<H", data, off + 4)[0]


@m19check("NFR-PT-01 shakedown.exe and shakedown_x64.dll of the zip are 64-bit Windows binaries, and the extension and the protocol crate check for 64-bit Linux", "offline")
def portable(ctx):
    with zipfile.ZipFile(dist(ctx) / ZIP) as z:
        for name in ("shakedown.exe", "@shakedown/shakedown_x64.dll"):
            assert pe_machine(z.read(name)) == 0x8664, f"{name} is not an x64 binary"
    r = subprocess.run(["cargo", "check", "--release", "--target", "x86_64-unknown-linux-gnu", "-p", "shakedown-ext",
                        "-p", "shakedown-protocol"], cwd=ctx.repo, capture_output=True, text=True, encoding="utf-8",
                       errors="replace", timeout=900)
    ctx.log(f"M19: cargo check for x86_64-unknown-linux-gnu exit {r.returncode}")
    assert r.returncode == 0, f"cargo check for 64-bit Linux exit {r.returncode}: {r.stderr[-800:]}"


@m19check("DST-01 tools/package.py given a tag that differs from the workspace version stops with an error and writes no zip", "offline")
def wrong_tag(ctx):
    out = ctx.repo / "target" / "dist"
    before = {p.name: p.stat().st_mtime for p in out.iterdir()} if out.is_dir() else {}
    r = subprocess.run([sys.executable, str(ctx.repo / "tools" / "package.py"), "--tag", "v9.9.9"], cwd=ctx.repo,
                       capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=900)
    after = {p.name: p.stat().st_mtime for p in out.iterdir()} if out.is_dir() else {}
    ctx.log(f"M19: package.py --tag v9.9.9 exit {r.returncode}: {(r.stdout + r.stderr).strip()[-300:]}")
    assert r.returncode != 0, f"package.py --tag v9.9.9 exit 0 with the workspace at {version(ctx)}"
    assert before == after, f"package.py --tag v9.9.9 changed target\\dist: before {before}, after {after}"
    text = r.stdout + r.stderr
    assert "9.9.9" in text and version(ctx) in text, f"the error names neither the tag nor the workspace version: {text[-400:]!r}"


@m19check("US-098 docs/install.md says to stop the runner and the AI agent sessions before winget upgrade, and that winget then cannot replace the running shakedown.exe and the upgrade fails", "offline")
def docs_upgrade(ctx):
    text = (ctx.repo / "docs" / "install.md").read_text(encoding="utf-8")
    sec = text[text.index("## Upgrade"):text.index("## Remove")]
    assert "winget upgrade" in sec, "the Upgrade section names no winget upgrade"
    assert "session stop" in sec and "shakedown mcp" in sec, "the Upgrade section does not say to stop the runner and the agents"
    assert re.search(r"winget cannot replace|upgrade fails", sec), "the Upgrade section does not say what winget does then"


@m19check("NFR-LG-04 DST-01 the root README says what Shakedown is, installs it by winget, the one-line script and the zip, names the licence GPL-2.0-or-later, and says the game's EULA binds its use", "offline")
def readme(ctx):
    text = (ctx.repo / "README.md").read_text(encoding="utf-8")
    for want in ("winget install casvanluijtelaar.Shakedown", "install.ps1 | iex", ZIP, "version 2 or later", "EULA",
                 "shakedown version"):
        assert want in text, f"README.md does not hold {want!r}"


@m19check("US-095 DST-03 in a shell with no Rust, HEMTT or Python on PATH, install.ps1 run as a file from a release folder installs the zip's files into SHAKEDOWN_INSTALL_DIR, adds that folder to the user PATH once, and shakedown version in a new shell prints the release's version")
def file_route(ctx):
    v = once(ctx)
    assert not any(v["clean"].values()), f"the clean shell still finds {v['clean']}"
    installed(ctx, v, "file", v["folders"]["file"])
    assert v["file_entries"] == 1, f"the user PATH names the install folder {v['file_entries']} times"
    printed_version(ctx, v["file_version"], v["folders"]["file"])


@m19check("US-095 DST-03 the one-line route, irm of install.ps1 from a local HTTP server piped to iex with SHAKEDOWN_DOWNLOAD at that server, installs the zip's files, adds the folder to the user PATH once, and shakedown version in a new shell prints the release's version")
def iex_route(ctx):
    v = once(ctx)
    installed(ctx, v, "iex", v["folders"]["iex"])
    assert v["iex_entries"] == 1, f"the user PATH names the install folder {v['iex_entries']} times"
    printed_version(ctx, v["iex_version"], v["folders"]["iex"])


@m19check("US-095 DST-04 the zip unpacked by hand into a folder put on PATH gives a shakedown that prints the release's version in a new shell")
def hand_route(ctx):
    v = once(ctx)
    printed_version(ctx, v["hand_version"], v["folders"]["hand"])


@m19check("US-095 DST-04 after the install by install.ps1 as a file, shakedown run --setup sp in a project with the README's quickstart check gives pass")
def file_run(ctx):
    v = once(ctx)
    passed(ctx, v, "file_run", v["folders"]["file"])


@m19check("US-095 DST-04 after the install by the one-line route, shakedown run --setup sp in a project with the README's quickstart check gives pass")
def iex_run(ctx):
    v = once(ctx)
    passed(ctx, v, "iex_run", v["folders"]["iex"])


@m19check("US-095 DST-04 after the install by hand, shakedown run --setup sp in a project with the README's quickstart check gives pass")
def hand_run(ctx):
    v = once(ctx)
    passed(ctx, v, "hand_run", v["folders"]["hand"])


@m19check("US-096 DST-03 install.ps1 run again over an install puts back the release's files, shakedown version prints the release's version, and the user PATH names the install folder once")
def again(ctx):
    v = once(ctx)
    installed(ctx, v, "again", v["folders"]["file"])
    assert v["again_entries"] == 1, f"the user PATH names the install folder {v['again_entries']} times after the second run"
    printed_version(ctx, v["again_version"], v["folders"]["file"])


@m19check("US-097 DST-03 a zip with one byte changed is refused by install.ps1 run as a file with exit 1 and an error naming both hashes and saying nothing was installed; every file of the install and the user PATH are unchanged, and shakedown version still prints the installed version")
def corrupt(ctx):
    v = once(ctx)
    r = v["corrupt"]
    assert r["code"] == 1, f"install.ps1 of a corrupt zip exit {r['code']}: {r['text'][-600:]}"
    refused(r, [v["corrupt_hash"], sha_value(ctx)])
    assert v["corrupt_after"] == v["corrupt_before"], "a file of the install changed"
    assert v["corrupt_path_same"], "the user PATH changed"
    printed_version(ctx, v["corrupt_version"], v["folders"]["file"])


@m19check("US-097 DST-03 a zip cut off at half, with nothing installed, is refused by install.ps1 run as a file with exit 1 and an error naming both hashes and saying nothing was installed; the install folder holds no file of the release and the user PATH is unchanged")
def cut(ctx):
    v = once(ctx)
    r = v["cut"]
    assert r["code"] == 1, f"install.ps1 of a cut-off zip exit {r['code']}: {r['text'][-600:]}"
    refused(r, [v["cut_hash"], sha_value(ctx)])
    left = [k for k in v["cut_tree"] if k in v["zip"]]
    assert not left, f"the install folder holds files of the release: {left}"
    assert v["cut_path_same"], "the user PATH changed"


@m19check("US-098 DST-03 with a session's runner and a shakedown mcp running from the install folder, the one-line route refuses, names each shakedown.exe of that folder by its id and command and says to stop them first; no file of the install changes and the session still answers status")
def running(ctx):
    v = once(ctx, "us098", part098)
    u = v["us098"]
    if isinstance(u, Exception):
        raise AssertionError(f"the US-098 part failed: {u!r}")
    procs = u["procs"]
    assert u["runner"] in procs and u["mcp_pid"] in procs, \
        f"the shakedown.exe processes of the folder {procs} do not hold the runner {u['runner']} and the mcp {u['mcp_pid']}"
    r = u["refused"]
    text = r["text"]
    ctx.log(f"M19 US-098: the refusal: {text.strip()[-700:]}")
    flat = squash(text)
    for pid, cmd in procs.items():
        assert re.search(rf"(?<!\d){pid}" + re.escape(squash(cmd)), flat, re.I), \
            f"the refusal does not name {pid} with its command {cmd!r}"
    others = {int(m.group(1)) for m in re.finditer(r"^\s{2,}(\d+)\s{2}\S", text, re.M)} - set(procs)
    assert not others, f"the refusal names processes that are not shakedown.exe of the folder: {sorted(others)}"
    assert re.search(r"\bstop\b", text, re.I), "the refusal does not say to stop them"
    assert u["after"] == u["before"], "a file of the install changed"
    st = u["status"]
    assert isinstance(st, dict) and (st.get("session") or {}).get("state") == "ready", \
        f"status after the refusal: {str(st)[:400]}"


@m19check("US-098 DST-03 once the session, its runner and shakedown mcp are stopped, the same one-line command upgrades the install and shakedown version prints the release's version")
def upgrades(ctx):
    v = once(ctx, "us098", part098)
    u = v["us098"]
    if isinstance(u, Exception):
        raise AssertionError(f"the US-098 part failed: {u!r}")
    assert not u["left"], f"shakedown.exe processes of the folder still ran: {u['left']}"
    r = u["upgrade"]
    assert r["code"] == 0, f"the second one-line run exit {r['code']}: {r['text'][-600:]}"
    assert u["upgraded"] == v["zip"], "the install folder does not hold the zip's files after the upgrade"
    printed_version(ctx, u["version"], v["folders"]["run"])


@m19check("US-095 US-096 US-097 DST-03 the user PATH is as it was before the M19 installs, put back by the harness after each")
def path_back(ctx):
    v = once(ctx)
    assert v["path_end"] == v["path_before"], "the user PATH differs from the one saved before the installs"
