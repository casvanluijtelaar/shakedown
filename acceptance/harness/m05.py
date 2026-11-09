"""The acceptance checks of M5 A session that outlives its client: US-007, US-035, US-052, US-053 and
US-060, and exit criterion 2 (NFR-RL-03 5 of 5, the searches of NFR-SC-01 and NFR-SC-03).

The start checks bring up what they need themselves and end the runner they started by its process id:
the four ways a host ends its MCP server (SES-08), the five kills of a runner during a run on a dedicated
session, and a stand-in that runner.json names. The ded batch drives the running dedicated session: a
run from another console, the sockets, the audit, connections without the right token, and a suspended
runner. The claude check starts headless Claude Code sessions and runs only when a --check value selects it.

A token is never logged or kept in a file: a message names a hit only as "a runner token" or "a session token".
"""

import json
import os
import re
import secrets
import socket
import subprocess
import sys
import time
import traceback
from pathlib import Path

from . import driver, win
from .core import batch, check
from .mcp import Mcp, structured, text_of, tool_error

LOCAL = Path(os.environ.get("LOCALAPPDATA", "")) / "Shakedown"
RUNNER_LOG = LOCAL / "runner.log"
RECORD = LOCAL / "record.json"
TOKEN = re.compile(r"skd_[rs]_[0-9a-f]{64}")
SESSION_TOKEN = re.compile(r"skd_s_[0-9a-f]{64}")
LONG = "us035/US-035-long"
ON_P1 = "us052/US-052-on-p1"
CREATE_SUSPENDED = 0x4
CREATE_NO_WINDOW = 0x08000000
MARK = "skdacc_m05_intruder"


# ================================================================ helpers

def log_mark():
    return RUNNER_LOG.stat().st_size if RUNNER_LOG.exists() else 0


def log_since(mark):
    with RUNNER_LOG.open("rb") as f:
        f.seek(mark)
        return f.read().decode("utf-8", "replace")


def no_token(text, where):
    found = TOKEN.findall(text or "")
    assert not found, f"{where} holds {len(found)} token(s) of the form skd_[rs]_<64 hex>"


def game_folder(ctx):
    for line in (ctx.project / "shakedown.toml").read_text(encoding="utf-8").splitlines():
        if line.startswith("game = "):
            return Path(line.split("=", 1)[1].strip().strip("'\""))
    raise AssertionError("the generated shakedown.toml has no game line")


def mission_copies(ctx, sid):
    folder = game_folder(ctx) / "MPMissions"
    return [p for p in folder.glob(f"shakedown_{sid}.*")] if folder.is_dir() else []


def session_json(ctx, sid):
    p = ctx.project / ".shakedown" / "sessions" / sid / "session.json"
    return json.loads(p.read_text(encoding="utf-8")) if p.exists() else None


def wait_no_runner(ctx, ceiling=75):
    """Waits up to `ceiling` s for a runner that runner.json names to exit by itself."""
    pid = driver.runner_pid()
    if pid and driver.alive(pid):
        ctx.log(f"waiting up to {ceiling} s for the runner {pid} to exit by itself")
        assert driver.wait_gone(pid, ceiling) is not None, f"the runner {pid} still runs after {ceiling} s"
    left = win.runner_processes()
    assert not left, f"{len(left)} shakedown runner process(es) run ({sorted(left)}); this check needs none"


def end_runner(ctx, pid):
    """Ends a runner this check started, by its id, and waits up to 10 s for it to be gone."""
    if pid and driver.alive(pid):
        ctx.log(f"ending the runner {pid} this check started, by its id")
        win.terminate(pid)
        driver.wait_gone(pid, 10)


def stop_session_and_runner(ctx, before):
    """Stops the session, any game process started since `before`, and the runner, each by id."""
    pid = driver.runner_pid()
    if pid and driver.alive(pid):
        ctx.sh("session", "stop", "--json", ceiling=180)
    for g in driver.game_pids() - before:
        if driver.wait_gone(g, 30) is None:
            ctx.log(f"game process {g} still runs; stopping it by id")
            driver.kill(g)
    end_runner(ctx, pid)


def free_mb():
    out = subprocess.run(["powershell", "-NoProfile", "-Command",
                          "(Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory"],
                         capture_output=True, text=True, timeout=60).stdout.strip()
    return int(out) // 1024 if out.isdigit() else None


def message(answer):
    return answer.stderr.strip() or answer.stdout.strip()


def keep(ctx):
    if not hasattr(ctx, "m05_keep"):
        ctx.m05_keep = {}
    return ctx.m05_keep


def once(ctx, key, fn):
    """Runs fn once per suite run and keeps its value or its error under key; checks share it."""
    store = keep(ctx)
    if key not in store:
        try:
            store[key] = fn(ctx)
        except Exception as e:
            ctx.log(f"step {key}: {traceback.format_exc()}")
            store[key] = e
    value = store[key]
    if isinstance(value, Exception):
        raise AssertionError(f"the step {key} failed: {value!r}")
    return value


def tool_texts(ctx):
    return keep(ctx).setdefault("tool-output", [])


# ================================================================ US-007 SES-08: four ways a host ends its server

def outlive(ctx, how):
    """Starts sp through `shakedown mcp`, ends the server the way `how` says, and asks a new server."""
    wait_no_runner(ctx)
    before = driver.game_pids()
    mark = log_mark()
    job = None
    out = {"how": how}
    try:
        if how in ("job-breakaway", "job-no-breakaway"):
            job = win.Job(breakaway=how == "job-breakaway")
            m = Mcp(ctx.project, ctx.install, ctx.log, creationflags=CREATE_SUSPENDED)
            job.assign(int(m.proc._handle))
            win.resume_handle(int(m.proc._handle))
        else:
            m = Mcp(ctx.project, ctx.install, ctx.log)
        m.initialize()
        r, seconds = m.call("session_start", {"setup": "sp"}, ceiling=75)
        tool_texts(ctx).append(text_of(r))
        s = structured(r)
        out["session"], out["start_s"] = s, seconds
        ctx.log(f"{how}: session_start sp answered {s['state']} in {seconds:.1f} s")
        out["runner"] = driver.runner_pid()
        if how == "stdin":
            m.close()
        elif how == "tree":
            tree = win.descendants(m.pid)
            out["tree"] = tree
            for pid in [m.pid] + tree:
                win.terminate(pid)
        else:
            job.close()
        for _ in range(100):  # 10 s
            if m.proc.poll() is not None:
                break
            time.sleep(0.1)
        out["server_ended"] = m.proc.poll() is not None
        if not out["server_ended"]:
            ctx.log(f"{how}: `shakedown mcp` {m.pid} still runs; stopping it by id")
            win.terminate(m.pid)
        out["runner_alive"] = bool(out["runner"]) and driver.alive(out["runner"])
        out["games_alive"] = [p for p in (mm["pid"] for mm in s["machines"]) if driver.alive(p)]
        with Mcp(ctx.project, ctx.install, ctx.log) as m2:
            r2, _ = m2.call("status")
            tool_texts(ctx).append(text_of(r2))
            out["status"] = structured(r2)
        out["log"] = log_since(mark)
    finally:
        if job:
            job.close()
        stop_session_and_runner(ctx, before)
    return out


def judge_outlive(out):
    s = out["session"]
    assert s["state"] == "ready", f"session_start sp gave {s['state']} after {out['start_s']:.1f} s: {s.get('reason')}"
    assert out["server_ended"], "the host's way of ending `shakedown mcp` did not end it within 10 s"
    assert out["runner_alive"], f"the runner {out['runner']} did not outlive its client ended by {out['how']}"
    assert len(out["games_alive"]) == len(s["machines"]), \
        f"{len(s['machines']) - len(out['games_alive'])} game process(es) of the session ended with the client"
    st = out["status"]
    assert st["runner"] == "running", f"a new server's status gave runner {st['runner']}"
    got = st["session"] or {}
    assert got.get("id") == s["id"] and got.get("state") == "ready", \
        f"a new server's status gave session {got.get('id')} {got.get('state')}, expected {s['id']} ready"
    assert re.search(r"started the runner by (breakaway|wmi)", out["log"]), \
        "runner.log has no line naming the way the runner was started (breakaway or wmi)"


@check("US-007 SES-06 SES-08 INT-CC-02 session_start through shakedown mcp, then its stdin closed: the runner and the game run on, and a new server's status gives the same session ready", "start")
def us007_stdin(ctx):
    judge_outlive(once(ctx, "outlive-stdin", lambda c: outlive(c, "stdin")))


@check("US-007 SES-08 INT-WIN-02 session_start through shakedown mcp in a kill-on-close job that allows breakaway, then the job closed: a new server's status gives the same session ready", "start")
def us007_job_breakaway(ctx):
    judge_outlive(once(ctx, "outlive-job-breakaway", lambda c: outlive(c, "job-breakaway")))


@check("US-007 SES-08 INT-WIN-02 session_start through shakedown mcp in a kill-on-close job that forbids breakaway, then the job closed: the runner started by wmi, and a new server's status gives the same session ready", "start")
def us007_job_no_breakaway(ctx):
    out = once(ctx, "outlive-job-no-breakaway", lambda c: outlive(c, "job-no-breakaway"))
    judge_outlive(out)
    assert re.search(r"started the runner by wmi", out["log"]), \
        "runner.log names no start by wmi though the client's job forbids breakaway"


@check("US-007 SES-08 session_start through shakedown mcp, then each process of its tree ended by id: a new server's status gives the same session ready", "start")
def us007_tree(ctx):
    judge_outlive(once(ctx, "outlive-tree", lambda c: outlive(c, "tree")))


# ================================================================ US-035 NFR-RL-03: five kills of the runner

def kill_tries(ctx):
    """Five tries: start ded, start a run, kill the runner by id with taskkill /F, watch the game end.
    A sixth start of ded judges the fifth try's recovery and is stopped normally."""
    wait_no_runner(ctx)
    before = driver.game_pids()
    tries = []
    ctx.log(f"free memory before the US-035 dedicated sessions: {free_mb()} MB")
    try:
        for n in range(1, 7):
            mark = log_mark()
            prev = tries[-1] if tries else None
            ans = ctx.sh("session", "start", "--setup", "ded", "--json", ceiling=400)
            s = ans.json() if ans.stdout.strip() else {}
            t = {"n": n, "start": ans, "session": s, "runner": driver.runner_pid()}
            if prev:
                prev["next_start"] = (ans.code, s.get("state"), t["runner"])
                prev["copies_after"] = [str(p) for p in mission_copies(ctx, prev["session"]["id"])]
                prev["session_json"] = session_json(ctx, prev["session"]["id"])
                prev["recovery_log"] = log_since(mark)
            assert ans.code == 0 and s.get("state") == "ready", f"try {n}: {ans}; {message(ans)[:400]}"
            if n == 6:
                break
            tries.append(t)
            sid = s["id"]
            t["copies_during"] = [str(p) for p in mission_copies(ctx, sid)]
            t["record"] = RECORD.read_text(encoding="utf-8") if RECORD.exists() else ""
            games = {m["pid"] for m in s["machines"]}
            t["games"] = games
            bg = ctx.sh.start("run", LONG, "--json")
            for _ in range(60):  # 30 s
                st = ctx.sh("status", "--json", ceiling=30).json()
                if st.get("run"):
                    break
                time.sleep(0.5)
            t["run_seen"] = bool(st.get("run"))
            t0 = time.monotonic()
            win.terminate(t["runner"])
            gone = None
            for _ in range(200):  # 20 s, the criterion is 10 s
                if not (driver.game_pids() & games):
                    gone = time.monotonic() - t0
                    break
                time.sleep(0.1)
            t["gone_s"] = gone
            left = driver.game_pids() & games
            for g in left:
                ctx.log(f"try {n}: game process {g} still runs 20 s after the kill; stopping it by id")
                driver.kill(g)
            try:
                bg.communicate(timeout=60)
                t["run_exit"] = bg.returncode
            except subprocess.TimeoutExpired:
                ctx.log(f"try {n}: the run command {bg.pid} did not exit 60 s after the kill; stopping it by id")
                win.terminate(bg.pid)
                t["run_exit"] = None
            runners_before = set(win.runner_processes())
            st_ans = ctx.sh("status", "--json", ceiling=30)
            t["status"] = st_ans.json()
            t["status_code"] = st_ans.code
            t["runners_after_status"] = set(win.runner_processes()) - runners_before
            ctx.log(f"try {n}: the game ended {gone if gone is None else round(gone, 2)} s after the kill")
    finally:
        stop_session_and_runner(ctx, before)
        ctx.log(f"free memory after the US-035 dedicated sessions: {free_mb()} MB")
    return tries


def tries_of(ctx):
    tries = once(ctx, "kill-tries", kill_tries)
    assert len(tries) == 5, f"{len(tries)} tries ran, expected 5"
    return tries


@check("US-035 NFR-RL-03 STP-01 INT-WIN-01 after taskkill /F of the runner during a run on a dedicated session no game process of the session runs within 10 s, 5 of 5", "start")
def us035_kill(ctx):
    tries = tries_of(ctx)
    for t in tries:
        assert t["run_seen"], f"try {t['n']}: status showed no run before the kill"
        assert t["gone_s"] is not None and t["gone_s"] <= 10, \
            f"try {t['n']}: game processes of the session ran {t['gone_s']} s after the kill (None: past 20 s)"
    ctx.log("US-035 kill to no game process: " + ", ".join(f"{t['gone_s']:.2f} s" for t in tries))


@check("US-035 INS-02 after the runner died status exits 0 with runner not-running and no session, and starts no runner", "start")
def us035_status(ctx):
    for t in tries_of(ctx):
        st = t["status"]
        assert t["status_code"] == 0, f"try {t['n']}: status exited {t['status_code']}"
        assert st["runner"] == "not-running" and st["session"] is None, \
            f"try {t['n']}: status gave runner {st['runner']}, session {(st['session'] or {}).get('id')}"
        assert not t["runners_after_status"], f"try {t['n']}: status started a runner"


@check("US-035 P-12 STP-03 STP-04 NFR-RL-03 the next session start of ded starts a new runner and is ready, no mission copy of the dead session is left, its session.json says stopped runner-died, and runner.log names the recovery", "start")
def us035_recovery(ctx):
    for t in tries_of(ctx):
        sid = t["session"]["id"]
        code, state, runner = t["next_start"]
        assert code == 0 and state == "ready", f"try {t['n']}: the next start exited {code} with {state}"
        assert runner and runner != t["runner"], f"try {t['n']}: the next start has runner {runner}, the dead one was {t['runner']}"
        assert t["copies_during"], f"try {t['n']}: no mission copy shakedown_{sid}.* was in MPMissions during the session"
        assert not t["copies_after"], f"try {t['n']}: the dead session's mission copy is left: {t['copies_after']}"
        sj = t["session_json"]
        assert sj and sj.get("state") == "stopped" and sj.get("reason") == "runner-died", \
            f"try {t['n']}: the dead session's session.json is {sj}"
        assert f"recovery: {sid}" in t["recovery_log"] and "runner-died" in t["recovery_log"], \
            f"try {t['n']}: runner.log has no recovery line for {sid}"


@check("STP-03 US-035 during a dedicated session record.json lists the session, each game process id and the mission copy", "start")
def stp03_record(ctx):
    for t in tries_of(ctx):
        rec, sid = t["record"], t["session"]["id"]
        assert sid in rec, f"try {t['n']}: record.json does not name the session {sid}"
        missing = [p for p in t["games"] if not re.search(rf"(?<!\d){p}(?!\d)", rec)]
        assert not missing, f"try {t['n']}: record.json does not list the game process(es) {missing}"
        assert f"shakedown_{sid}" in rec, f"try {t['n']}: record.json does not list the mission copy shakedown_{sid}"


# ================================================================ US-060 4: a stand-in of another version

STANDIN = r'''
import json, socket, sys, threading, time
srv = socket.socket(); srv.bind(("127.0.0.1", 0)); srv.listen(8)
open(sys.argv[1], "w").write(str(srv.getsockname()[1]))
def serve(c):
    try:
        c.settimeout(10)
        line = c.makefile("rb").readline()
        hello = json.loads(line)
        with open(sys.argv[2], "a") as f:
            f.write(json.dumps({"hello": hello.get("hello"), "version": hello.get("version")}) + "\n")
        c.sendall(b'{"hello": "runner", "version": "0.0.1"}\n')
        time.sleep(5)
    except Exception:
        pass
    finally:
        c.close()
srv.settimeout(1)
end = time.monotonic() + 600
while time.monotonic() < end:
    try:
        c, _ = srv.accept()
    except socket.timeout:
        continue
    threading.Thread(target=serve, args=(c,), daemon=True).start()
'''


def standin(ctx):
    wait_no_runner(ctx)
    folder = ctx.work / "m05-standin"
    folder.mkdir(exist_ok=True)
    script, port_file, hellos = folder / "standin.py", folder / "port", folder / "hellos.jsonl"
    script.write_text(STANDIN, encoding="utf-8")
    for p in (port_file, hellos):
        p.unlink(missing_ok=True)
    proc = subprocess.Popen([sys.executable, "-I", str(script), str(port_file), str(hellos)], cwd=folder)
    out = {"pid": proc.pid}
    try:
        for _ in range(100):
            if port_file.exists() and port_file.read_text().strip():
                break
            time.sleep(0.1)
        port = int(port_file.read_text())
        started = win.created_at(proc.pid).strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3] + "Z"
        LOCAL.mkdir(parents=True, exist_ok=True)
        driver.RUNNER_JSON.write_text(json.dumps({"port": port, "token": "skd_r_" + secrets.token_hex(32),
                                                  "pid": proc.pid, "started_at": started}), encoding="utf-8")
        out["version"] = ctx.sh("version").stdout.strip().split()[-1]
        runners = set(win.runner_processes())
        out["status"] = ctx.sh("status", "--json", ceiling=30)
        out["run"] = ctx.sh("run", "sel/RUN-01-a", ceiling=30)
        with Mcp(ctx.project, ctx.install, ctx.log) as m:
            out["mcp"] = {name: m.call(name, args)[0] for name, args in
                          (("run_start", {"ids": ["sel/RUN-01-a"]}), ("exec", {"machine": "p1", "code": "1"}),
                           ("status", {}))}
        for r in out["mcp"].values():
            tool_texts(ctx).append(text_of(r))
        out["new_runners"] = set(win.runner_processes()) - runners
        out["hellos"] = hellos.read_text().splitlines() if hellos.exists() else []
        out["runner_json_pid"] = driver.runner_pid()
    finally:
        proc.kill()
        proc.wait(timeout=30)
        if driver.runner_pid() == proc.pid:
            driver.RUNNER_JSON.unlink(missing_ok=True)
    return out


@check("US-060 P-01 INS-02 AGT-02 a stand-in that runner.json names and that answers another version is refused: status says unreachable, run exits 4 and the tools return runner-unreachable naming both versions, and no runner starts", "start")
def us060_standin(ctx):
    out = once(ctx, "standin", standin)
    pid, mine = out["pid"], out["version"]
    assert out["hellos"], "the stand-in got no client hello"
    st = out["status"]
    assert st.code == 0 and st.seconds < 10, f"{st}"
    j = st.json()
    assert j["runner"] == "unreachable" and j["runner_pid"] == pid and j["session"] is None, \
        f"status gave runner {j['runner']}, runner_pid {j['runner_pid']} (stand-in {pid}), session {j['session']}"
    run = out["run"]
    text = message(run)
    assert run.code == 4 and run.seconds < 10, f"{run}: {text[:300]}"
    for part in (str(pid), "0.0.1", mine):
        assert part in text, f"run's message does not name {part}: {text!r}"
    for name in ("run_start", "exec"):
        t = tool_error(out["mcp"][name], "runner-unreachable")
        for part in (str(pid), "0.0.1", mine):
            assert part in t, f"{name}'s error does not name {part}: {t!r}"
    sj = structured(out["mcp"]["status"])
    assert sj["runner"] == "unreachable" and sj["runner_pid"] == pid, f"the status tool gave {sj['runner']} {sj['runner_pid']}"
    assert not out["new_runners"], f"{len(out['new_runners'])} runner(s) started beside the stand-in"
    assert out["runner_json_pid"] == pid, "runner.json no longer names the stand-in"


# ================================================================ the ded session

M5_DED = []


def m5check(name, phase="session", setup="ded"):
    M5_DED.append(name)
    return check(name, phase, needs=(), setup=setup)


def step(ctx, key, fn):
    try:
        ctx.m05[key] = fn()
    except Exception as e:
        ctx.log(f"batch step {key}: {traceback.format_exc()}")
        ctx.m05[key] = e


def got(ctx, key):
    store = getattr(ctx, "m05", {})
    assert key in store, f"the batch step {key} did not run this repeat; see harness.log"
    if isinstance(store[key], Exception):
        raise AssertionError(f"the batch step {key} failed: {store[key]!r}")
    return store[key]


def run_elsewhere(ctx):
    """`shakedown run` from another process with a console of its own, as from another terminal."""
    before = set(ctx.sh.run_folders())
    proc = subprocess.run([ctx.sh.exe, "run", ON_P1, "--json"], cwd=ctx.project, capture_output=True, text=True,
                          encoding="utf-8", errors="replace", timeout=300, creationflags=CREATE_NO_WINDOW)
    out = json.loads(proc.stdout)
    result = json.loads(Path(out["result_file"]).read_text(encoding="utf-8"))
    ctx.runs.append((None, out, result, sorted(set(ctx.sh.run_folders()) - before)))
    return proc.returncode, out, result


def netstat(ctx, pids):
    rows = []
    for proto in ("TCP", "TCPv6", "UDP", "UDPv6"):
        out = subprocess.run(["netstat", "-ano", "-p", proto], capture_output=True, text=True, timeout=60).stdout
        for line in out.splitlines():
            f = line.split()
            if f and f[0] in ("TCP", "UDP") and f[-1].isdigit() and int(f[-1]) in pids:
                rows.append((f[0], f[1], f[2] if len(f) > 3 else "*:*", f[3] if f[0] == "TCP" else "", int(f[-1])))
    return rows


def hello_attempt(port, lines):
    """Sends the lines on a new connection; returns (closed by the runner, bytes it answered, seconds)."""
    t0 = time.monotonic()
    s = socket.create_connection(("127.0.0.1", port), timeout=10)
    try:
        s.sendall("".join(json.dumps(x) + "\n" for x in lines).encode("utf-8"))
        data = b""
        s.settimeout(1)
        for _ in range(10):  # 10 s
            try:
                chunk = s.recv(4096)
            except socket.timeout:
                continue
            except OSError:
                return True, data, time.monotonic() - t0
            if not chunk:
                return True, data, time.monotonic() - t0
            data += chunk
        return False, data, time.monotonic() - t0
    finally:
        s.close()


def intrusions(ctx):
    rj = json.loads(driver.RUNNER_JSON.read_text(encoding="utf-8"))
    port, sid = rj["port"], ctx.session["id"]
    p1 = next(m["pid"] for m in ctx.session["machines"] if m["name"] == "p1")
    found = SESSION_TOKEN.findall(win.environment(p1))
    assert found, "no session token in p1's game process environment"
    session_token = found[0]
    version = ctx.sh("version").stdout.strip().split()[-1]
    order = {"request": "exec", "machine": "all", "code": f"{MARK} = 1", "kind": "exec", "target": "p1"}
    mark = log_mark()
    tries = {
        "client bad-token": [{"hello": "client", "version": version, "token": "skd_r_" + "0" * 64}, order],
        "not-hello": [order],
        "extension bad-token": [{"hello": "extension", "session": sid, "machine": "p1", "token": "skd_s_" + "0" * 64}, order],
        "extension unknown-process": [{"hello": "extension", "session": sid, "machine": "p1", "token": session_token}, order],
    }
    out = {name: hello_attempt(port, lines) for name, lines in tries.items()}
    del session_token
    for _ in range(20):
        text = log_since(mark)
        if text.count("refused ") >= 4:
            break
        time.sleep(0.25)
    ans = ctx.sh("exec", "--machine", "all", "--code", f'isNil "{MARK}"', "--json")
    st = ctx.sh("status", "--json").json()
    return {"tries": out, "log": log_since(mark), "pid": os.getpid(), "isnil": ans.json(), "status": st}


def suspended(ctx):
    st = ctx.sh("status", "--json").json()
    pid = st["runner_pid"]
    starts = {m["name"]: m["start"] for m in st["session"]["machines"]}
    out = {"pid": pid, "sid": st["session"]["id"], "starts": starts}
    runners = set(win.runner_processes())
    win.suspend(pid)
    try:
        out["status"] = ctx.sh("status", "--json", ceiling=30)
        out["run"] = ctx.sh("run", ON_P1, ceiling=30)
        with Mcp(ctx.project, ctx.install, ctx.log) as m:
            out["mcp"] = {name: m.call(name, args)[0] for name, args in
                          (("run_start", {"ids": [ON_P1]}), ("exec", {"machine": "p1", "code": "1"}), ("status", {}))}
        out["new_runners"] = set(win.runner_processes()) - runners
    finally:
        win.resume(pid)
    for r in out["mcp"].values():
        tool_texts(ctx).append(text_of(r))
    after = None
    for _ in range(60):  # 30 s
        after = ctx.sh("status", "--json", ceiling=30).json()
        if after["runner"] == "running" and after["session"] and after["session"]["state"] == "ready":
            break
        time.sleep(0.5)
    out["after"] = after
    return out


@batch("ded")
def ded_batch(ctx, checks):
    ctx.m05 = {}
    if not any(c.name in M5_DED for c in checks):
        return
    step(ctx, "elsewhere", lambda: run_elsewhere(ctx))
    rp = driver.runner_pid()
    games = {m["pid"] for m in ctx.session["machines"]}
    step(ctx, "netstat", lambda: (rp, games, json.loads(driver.RUNNER_JSON.read_text(encoding="utf-8"))["port"],
                                  netstat(ctx, games | {rp})))
    step(ctx, "exec-cli", lambda: ctx.sh("exec", "--machine", "server", "--code", "6 * 7 + 0 * 52051", "--json"))

    def by_mcp():
        with Mcp(ctx.project, ctx.install, ctx.log) as m:
            r = m.call("exec", {"machine": "hc1", "code": "6 * 8 + 0 * 52053"})[0]
            s = m.call("status")[0]
        tool_texts(ctx).extend([text_of(r), text_of(s)])
        return structured(r)
    step(ctx, "exec-mcp", by_mcp)
    step(ctx, "cmdlines", lambda: [cmd for _, _, cmd in win.processes().values()])
    step(ctx, "intrusions", lambda: intrusions(ctx))
    step(ctx, "audit", lambda: [json.loads(line) for line in
                                (ctx.project / ".shakedown" / "sessions" / ctx.session["id"] / "audit.jsonl")
                                .read_text(encoding="utf-8").splitlines() if line.strip()])
    step(ctx, "suspended", lambda: suspended(ctx))


@m5check("US-007 SES-06 after session start returned, shakedown run from another console runs against that session and its result file names the same session id")
def us007_cli(ctx):
    code, out, result = got(ctx, "elsewhere")
    assert code == 0, f"the run exited {code}"
    assert result["session"]["id"] == ctx.sessions["ded"]["id"], \
        f"the result file names session {result['session']['id']}, the started session is {ctx.sessions['ded']['id']}"


@m5check("US-052 SEC-01 NFR-SC-01 during a dedicated session netstat shows every socket of the runner and every connection of a game process to it on 127.0.0.1")
def us052_netstat(ctx):
    rp, games, port, rows = got(ctx, "netstat")
    mine = [r for r in rows if r[4] == rp]
    assert any(r[3] == "LISTENING" and r[1].endswith(f":{port}") for r in mine), \
        f"netstat shows no listening socket of the runner {rp} on its port {port}"
    bad = [r for r in mine if not r[1].startswith("127.0.0.1:") or (r[0] == "TCP" and r[3] != "LISTENING"
                                                                      and not r[2].startswith("127.0.0.1:"))]
    assert not bad, f"sockets of the runner off 127.0.0.1: {bad}"
    to_runner = [r for r in rows if r[4] in games and r[2].endswith(f":{port}")]
    assert len({r[4] for r in to_runner}) == len(games), \
        f"{len({r[4] for r in to_runner})} of {len(games)} game processes have a connection to the runner's port"
    bad = [r for r in to_runner if not (r[1].startswith("127.0.0.1:") and r[2].startswith("127.0.0.1:"))]
    assert not bad, f"game process connections to the runner off 127.0.0.1: {bad}"


@m5check("US-052 SEC-04 every exec from the command line, from MCP and from a check is in the session's audit.jsonl with its time, source, target and code")
def us052_audit(ctx):
    got(ctx, "elsewhere"), got(ctx, "exec-cli"), got(ctx, "exec-mcp")
    lines = got(ctx, "audit")
    for line in lines:
        missing = [k for k in ("time", "source", "target", "code") if k not in line]
        assert not missing, f"an audit line lacks {missing}: keys {sorted(line)}"

    def find(source, target, needle):
        return [l for l in lines if l["source"] == source and l["target"] == target and needle in l["code"]]
    assert find("cli", "server", "52051"), "audit.jsonl has no line source cli, target server for `shakedown exec`"
    assert find("mcp", "hc1", "52053"), "audit.jsonl has no line source mcp, target hc1 for the exec tool"
    assert find(ON_P1, "p1", "52052"), f"audit.jsonl has no line source {ON_P1}, target p1 for the code sent with skd_fnc_on"
    body = [l for l in lines if l["source"] == ON_P1 and "skd_fnc_on" in l["code"]]
    assert body, f"audit.jsonl has no line of the check {ON_P1}'s body"


@m5check("US-052 SEC-02 NFR-SC-03 SEC-06 no runner or session token is on any process's command line during the session, or in the tool and command output of the round")
def us052_no_token_live(ctx):
    for cmd in got(ctx, "cmdlines"):
        no_token(cmd, "a process's command line")
    for text in tool_texts(ctx):
        no_token(text, "a tool output")
    for _, out, _, _ in ctx.runs:
        no_token(json.dumps(out), "a run --json output")


@m5check("US-053 SEC-01 NFR-SC-02 a client hello with a wrong runner token, a first message that is not a hello and an extension hello with a wrong session token are closed without a reply, and runner.log records each")
def us053_bad_token(ctx):
    it = got(ctx, "intrusions")
    for name in ("client bad-token", "not-hello", "extension bad-token"):
        closed, data, seconds = it["tries"][name]
        assert closed and not data, f"{name}: the connection was {'closed' if closed else 'still open after 10 s'}" \
                                    f" with {len(data)} bytes of answer"
    pid = it["pid"]
    for line in (f"refused client connection from pid {pid}: bad-token",
                 f"refused client connection from pid {pid}: not-hello",
                 f"refused extension connection from pid {pid}: bad-token"):
        assert line in it["log"], f"runner.log has no line `{line}`"


@m5check("US-053 SEC-03 NFR-SC-02 an extension hello with the right session token from a process the runner did not start is closed without a reply, runner.log records unknown-process, and no order reached a machine")
def us053_unknown_process(ctx):
    it = got(ctx, "intrusions")
    closed, data, _ = it["tries"]["extension unknown-process"]
    assert closed and not data, f"the connection was {'closed' if closed else 'open after 10 s'} with {len(data)} bytes"
    line = f"refused extension connection from pid {it['pid']}: unknown-process"
    assert line in it["log"], f"runner.log has no line `{line}`"
    no_token(it["log"], "runner.log")
    res = it["isnil"]["results"]
    assert len(res) == len(ctx.session["machines"]), f"exec on all answered for {sorted(res)}"
    ran = [m for m, r in res.items() if r.get("value") is not True]
    assert not ran, f"the intruders' code ran on {ran}: isNil {MARK} is not true there"
    assert not [l for l in got(ctx, "audit") if f"{MARK} = 1" in l["code"]], "audit.jsonl has a line with the intruders' code"
    st = it["status"]["session"]
    assert st["state"] == "ready" and all(m["state"] == "ready" for m in st["machines"]), \
        f"after the intrusions the session is {st['state']}, machines {[(m['name'], m['state']) for m in st['machines']]}"


@m5check("US-060 P-01 INS-02 INS-04 with the runner suspended status --json exits 0 within 10 s with runner unreachable, its runner_pid and no session, and run exits 4 within 10 s naming the process and that ending it ends the session")
def us060_suspended_cli(ctx):
    out = got(ctx, "suspended")
    st = out["status"]
    assert st.code == 0 and st.seconds <= 10, f"{st}"
    j = st.json()
    assert j["runner"] == "unreachable" and j["runner_pid"] == out["pid"] and j["session"] is None, \
        f"status gave runner {j['runner']}, runner_pid {j['runner_pid']} (runner {out['pid']}), session {j['session'] and j['session']['id']}"
    run = out["run"]
    text = message(run)
    assert run.code == 4 and run.seconds <= 10, f"{run}: {text[:300]}"
    assert str(out["pid"]) in text, f"run's message does not name the runner's process id: {text!r}"
    assert "end" in text.lower() and "session" in text.lower(), f"run's message does not say ending the process ends its session: {text!r}"


@m5check("US-060 P-01 AGT-02 with the runner suspended no second runner starts, run_start and exec return runner-unreachable, and the status tool returns runner unreachable")
def us060_suspended_mcp(ctx):
    out = got(ctx, "suspended")
    assert not out["new_runners"], f"{len(out['new_runners'])} runner(s) started beside the suspended one"
    for name in ("run_start", "exec"):
        t = tool_error(out["mcp"][name], "runner-unreachable")
        assert str(out["pid"]) in t, f"{name}'s error does not name the runner's process id: {t!r}"
    sj = structured(out["mcp"]["status"])
    assert sj["runner"] == "unreachable" and sj["runner_pid"] == out["pid"], f"the status tool gave {sj['runner']} {sj['runner_pid']}"


@m5check("US-060 SES-06 once the runner is resumed status shows the same session ready with every machine at the start number it had")
def us060_resumed(ctx):
    out = got(ctx, "suspended")
    after = out["after"]
    assert after["runner"] == "running", f"status after the resume gave runner {after['runner']}"
    s = after["session"]
    assert s and s["id"] == out["sid"] and s["state"] == "ready", f"status after the resume gave {s and (s['id'], s['state'])}"
    starts = {m["name"]: m["start"] for m in s["machines"]}
    assert starts == out["starts"], f"start numbers before {out['starts']}, after {starts}"


# ================================================================ end: the token search after the sessions

@check("US-052 NFR-SC-03 SEC-06 after the sessions no result file, run log, session file, runner.log, record.json or harness.log holds a runner or session token", "end")
def us052_no_token_after(ctx):
    files = [p for p in (ctx.project / ".shakedown").rglob("*") if p.is_file()]
    files += [RUNNER_LOG, RECORD, ctx.work / "harness.log"]
    hits = []
    for p in files:
        if p.exists() and p.stat().st_size < 200_000_000:
            if TOKEN.search(p.read_bytes().decode("utf-8", "replace")):
                hits.append(str(p))
    assert not hits, f"{len(hits)} file(s) hold a token of the form skd_[rs]_<64 hex>: {hits[:10]}"
    ctx.log(f"NFR-SC-03: searched {len(files)} files, no token")


# ================================================================ claude: US-007 under Claude Code itself

def claude_session(ctx, project, prompt, tools, ceiling):
    exe = subprocess.run(["where", "claude"], capture_output=True, text=True, timeout=30).stdout.split()
    assert exe, "claude is not on PATH"
    env = dict(os.environ)
    env["PATH"] = str(ctx.install) + os.pathsep + env.get("PATH", "")
    argv = [exe[0], "-p", prompt, "--mcp-config", str(project / ".mcp.json"), "--strict-mcp-config",
            "--allowedTools", ",".join(tools), "--output-format", "json", "--max-turns", "12"]
    proc = subprocess.Popen(argv, cwd=project, env=env, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, text=True, encoding="utf-8", errors="replace")
    ctx.log(f"claude -p started, pid {proc.pid}")
    try:
        stdout, stderr = proc.communicate(timeout=ceiling)
    except subprocess.TimeoutExpired:
        ctx.log(f"claude {proc.pid} ran past {ceiling} s; ending its tree by id")
        subprocess.run(["taskkill", "/PID", str(proc.pid), "/T", "/F"], capture_output=True, timeout=30)
        stdout, stderr = proc.communicate(timeout=30)
    return proc.pid, proc.returncode, stdout, stderr


def mcp_servers_of(project):
    return {pid for pid, (_, name, cmd) in win.processes().items()
            if name.lower() == "shakedown.exe" and cmd.lower().rstrip().endswith(" mcp")}


@check("US-007 SES-06 SES-08 INT-CC-02 a headless Claude Code session in e2e/grappling-hook starts the dedicated session through MCP and ends; a new one's status tool gives the same session ready", "claude")
def us007_claude(ctx):
    project = ctx.repo / "e2e" / "grappling-hook"
    wait_no_runner(ctx)
    before = driver.game_pids()
    servers_before = mcp_servers_of(project)
    mark = log_mark()
    ctx.log(f"free memory before the Claude Code dedicated session: {free_mb()} MB")
    try:
        _, code1, out1, err1 = claude_session(
            ctx, project,
            "Call the shakedown MCP tool session_start with setup \"dedicated\". If it answers state starting, call the "
            "status tool every few seconds until the session state is ready or failed. Then reply with only the session "
            "id and its state, as `<id> <state>`, and nothing else.",
            ["mcp__shakedown__session_start", "mcp__shakedown__status"], 400)
        ctx.log(f"first claude exited {code1}")
        st1 = ctx.sh("status", "--json").json()
        sid = (st1.get("session") or {}).get("id")
        left = mcp_servers_of(project) - servers_before
        _, code2, out2, err2 = claude_session(
            ctx, project,
            "Call the shakedown MCP tool status once. Reply with only the session id and its state from its answer, as "
            "`<id> <state>`, and nothing else.",
            ["mcp__shakedown__status"], 180)
        ctx.log(f"second claude exited {code2}")
        st2 = ctx.sh("status", "--json").json()
        left2 = mcp_servers_of(project) - servers_before
        log = log_since(mark)
    finally:
        stop_session_and_runner(ctx, before)
        for pid in mcp_servers_of(project) - servers_before:
            ctx.log(f"`shakedown mcp` {pid} outlived its Claude Code session; stopping it by id")
            win.terminate(pid)
        ctx.log(f"free memory after the Claude Code dedicated session: {free_mb()} MB")
    r1 = json.loads(out1).get("result", "") if out1.strip().startswith("{") else out1
    r2 = json.loads(out2).get("result", "") if out2.strip().startswith("{") else out2
    ctx.log(f"first claude replied {r1.strip()[:200]!r}; second replied {r2.strip()[:200]!r}")
    assert code1 == 0, f"the first Claude Code session exited {code1}: {err1[-300:]}"
    assert sid and st1["session"]["state"] == "ready", f"after the first Claude Code session status gave {st1.get('session')}"
    assert not left, f"{len(left)} `shakedown mcp` process(es) outlived the first Claude Code session"
    assert st1["runner"] == "running", f"after the first Claude Code session the runner is {st1['runner']}"
    assert code2 == 0, f"the second Claude Code session exited {code2}: {err2[-300:]}"
    assert sid in r2 and "ready" in r2, f"the second Claude Code session's status gave {r2.strip()[:200]!r}, expected {sid} ready"
    assert st2["session"] and st2["session"]["id"] == sid, f"after the second session status gave {st2.get('session')}"
    assert not left2, f"{len(left2)} `shakedown mcp` process(es) outlived the second Claude Code session"
    assert re.search(r"started the runner by (breakaway|wmi)", log), "runner.log names no way the runner was started"


# ================================================================ SEC-02 INT-WIN-03: who can read runner.json

ACL_PS = r'''
$me = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$a = Get-Acl -LiteralPath '{path}'
@($a.GetAccessRules($true, $true, [System.Security.Principal.SecurityIdentifier]) | ForEach-Object {
  [pscustomobject]@{ me = ($_.IdentityReference.Value -eq $me); sid = $(if ($_.IdentityReference.Value -eq $me) { "the current user" } else { $_.IdentityReference.Value });
                     allow = ($_.AccessControlType -eq "Allow"); read = ((([int]$_.FileSystemRights) -band 1) -ne 0); rights = "$($_.FileSystemRights)" }
}) | ConvertTo-Json -Compress
'''
SYSTEM_SID = "S-1-5-18"


def runner_json_acl(ctx):
    pid = driver.runner_pid()
    if not (pid and driver.alive(pid)):
        # Any command but status, list, guide, version and mcp starts a runner (P-01); with no session it then refuses.
        ctx.sh("exec", "--machine", "p1", "--code", "1")
        pid = driver.runner_pid()
        ctx.runner_pid = pid  # the suite waits for it to exit by itself
    assert pid and driver.alive(pid), "no runner runs and runner.json names none after a command that starts one"
    out = subprocess.run(["powershell", "-NoProfile", "-Command", ACL_PS.replace("{path}", str(driver.RUNNER_JSON).replace("'", "''"))],
                         capture_output=True, text=True, timeout=60).stdout.strip()
    rules = json.loads(out) if out else []
    return rules if isinstance(rules, list) else [rules]


@check("US-052 SEC-02 INT-WIN-03 while a runner runs, runner.json grants read to the current user and to no other account or group", "start")
def us052_runner_json_acl(ctx):
    rules = runner_json_acl(ctx)
    ctx.log("runner.json ACL: " + "; ".join(f"{r['sid']} {'allow' if r['allow'] else 'deny'} {r['rights']}" for r in rules))
    assert any(r["me"] and r["allow"] and r["read"] for r in rules), "runner.json grants the current user no read"
    # SYSTEM is the operating system itself, not an account a person or program logs on as.
    others = [r for r in rules if r["allow"] and r["read"] and not r["me"] and r["sid"] != SYSTEM_SID]
    assert not others, "runner.json is readable by " + ", ".join(f"{r['sid']} ({r['rights']})" for r in others) + \
                       " besides the current user"
