"""The checks of M6, reload by restart: US-011, US-012, US-040, US-055, US-056 and US-057.

They share one dedicated session of setup rld: a server, two clients and a headless client loading CBA
and both fixtures, each fixture watched by reload. @skd_fixture is watched in its installed copy, which
a test may add fn_new.sqf to and name config.cpp of (FIX-03). @skd_fixture_cba is watched in a staged
copy of its sources in the project, with a rebuild that copies the staged fnc_version.sqf into the
installed copy, as spec/contract/fixture-mod.md gives it. Per repeat the batch drives, in this order:

1. US-040: a check run, edited and run again; reload naming the check, by command and by tool.
2. US-011: a check that sets a public variable, then a @reset mission check.
3. US-056: a new file and config.cpp named to reload, a game restart.
4. US-012: a @reset mission check whose restart loses p2, then the reload that brings the session back.
5. US-055: the staged version changed to "2" and reloaded, a game restart; a check sees "2".
6. US-057: the staged version put back to "1" and reloaded, with p2 ended during the restart; a run
   refused; the reload tool brings the session back; a check sees "1".

Every repeat ends with the installed copy as it was. After the session stops, the installed copy is
put back once more, whatever happened.
"""

import json
import os
import shutil
import subprocess
import time
from datetime import datetime
from pathlib import Path

from . import driver
from .core import AFTER_SESSION, batch, check, execution
from .mcp import Mcp, structured, tool_error

SETUP = "rld"
EDIT = "m06/US-040-edit"
SET_VAR, RESET = "m06/US-011-set-var", "m06/US-011-mission-reset"
R12, AFTER12 = "m06/US-012-mission-reset", "m06/US-012-after"
V2, V1 = "m06/US-055-version-2", "m06/US-057-version-1"
ALL = ["server", "p1", "p2", "hc1"]
CEILING = 400              # a game restart takes about 85 s; the setup's start ceiling bounds each start
NFR_PF_02 = 90.0           # s, plus the rebuild, which copies one file

PATHS = {}                 # what project_toml worked out, for the batch and the restore


def project_toml(project, game, install):
    """The mods and the setup of M6, and the staged copy of @skd_fixture_cba's sources."""
    install = Path(install)
    fixtures = install / "fixtures"
    installed = fixtures / "skdfx_cba" / "addons" / "main" / "functions" / "fnc_version.sqf"
    stage_root = Path(project) / "stage" / "skdfx_cba" / "addons"
    shutil.copytree(fixtures / "skdfx_cba" / "addons", stage_root)
    PATHS.update(
        installed=installed, original=installed.read_bytes(),
        staged=stage_root / "main" / "functions" / "fnc_version.sqf",
        new=fixtures / "skdfx" / "addons" / "main" / "functions" / "fn_new.sqf",
        config=fixtures / "skdfx" / "addons" / "main" / "config.cpp",
        source=fixtures / "skdfx" / "addons", stage=stage_root)
    rebuild = f'copy /y "{PATHS["staged"]}" "{installed}"'
    return (
        f"\n[mods.fixture-rld]\npath = '{Path(game) / '@skd_fixture'}'\npatch = \"skdfx_main\"\n"
        f"source = '{PATHS['source']}'\nprefix = 'skdfx\\addons'\n\n"
        f"[mods.fixture-cba-rld]\npath = '{fixtures / '@skd_fixture_cba'}'\npatch = \"skdfx_cba\"\n"
        f"source = '{stage_root}'\nprefix = 'skdfx_cba\\addons'\nrebuild = '{rebuild}'\n\n"
        f"[setups.{SETUP}]\nmode = \"dedicated\"\nclients = 2\nheadless = 1\n"
        "mods = [\"cba\", \"fixture-rld\", \"fixture-cba-rld\"]\n")


def put_back(ctx, setup=SETUP):
    """FIX-03: the installed copy as it was, once no game process of the session runs."""
    if setup != SETUP or not PATHS:
        return
    if PATHS["new"].exists():
        PATHS["new"].unlink()
        ctx.log("M6: deleted fn_new.sqf from the installed copy")
    if PATHS["installed"].read_bytes() != PATHS["original"]:
        PATHS["installed"].write_bytes(PATHS["original"])
        ctx.log("M6: put the installed fnc_version.sqf back as it was")


AFTER_SESSION.append(put_back)


def set_staged(value):
    PATHS["staged"].write_bytes(PATHS["original"].replace(b'"1"', f'"{value}"'.encode()))


def rel(path, ctx):
    return os.path.relpath(path, ctx.project)


def status(ctx):
    return ctx.sh("status", "--json", ceiling=30).json()


def starts(st):
    return {m["name"]: m["start"] for m in (st.get("session") or {}).get("machines", [])}


def states(st):
    return {m["name"]: m["state"] for m in (st.get("session") or {}).get("machines", [])}


def same_path(a, b):
    return os.path.normcase(os.path.normpath(str(a))) == os.path.normcase(os.path.normpath(str(b)))


def file_entry(reload, path):
    found = [f for f in reload.get("files", []) if same_path(f["path"], path)]
    assert found, f"the reload does not list {path}; it lists {[f['path'] for f in reload.get('files', [])]}"
    return found[0]


def utc(text):
    return datetime.fromisoformat(text.replace("Z", "+00:00")).timestamp()


def kill_when_new(ctx, bg, name, old_start, ceiling=200):
    """Polls status until `name` runs a process with a start number above old_start, then ends that
    process by its id. Returns (pid, seconds after the poll began) or (None, why)."""
    began = time.monotonic()
    for _ in range(int(ceiling * 2)):
        if bg.poll() is not None:
            return None, f"the command ended first, exit {bg.returncode}"
        try:
            m = {m["name"]: m for m in (status(ctx).get("session") or {}).get("machines", [])}.get(name)
        except AssertionError:
            m = None
        if m and m["start"] > old_start and m.get("pid"):
            subprocess.run(["taskkill", "/PID", str(m["pid"]), "/F"], capture_output=True, timeout=30)
            ctx.log(f"M6: ended {name} start {m['start']}, pid {m['pid']}, by its id")
            return m["pid"], time.monotonic() - began
        time.sleep(0.5)
    return None, f"{name} showed no start above {old_start} within {ceiling} s"


def finish(ctx, bg, args, ceiling=CEILING):
    started = time.monotonic()
    try:
        out, err = bg.communicate(ceiling)
    except subprocess.TimeoutExpired:
        driver.kill(bg.pid)
        raise AssertionError(f"`shakedown {' '.join(args)}` did not end within {ceiling} s; stopped it by id")
    answer = driver.Answer(list(args), bg.returncode, out, err, time.monotonic() - started)
    ctx.log(f"{answer}")
    return answer


def add_run(ctx, answer):
    """Keeps a run made in the background with the suite's runs, so the checks over every run read it."""
    out = answer.json()
    result = json.loads(Path(out["result_file"]).read_text(encoding="utf-8"))
    ctx.runs.append((answer, out, result, []))
    ctx.session_runs.append((answer, out, result, []))
    return out, result


def step(ctx, key, fn):
    try:
        ctx.m06[key] = fn()
    except Exception as e:  # noqa: BLE001 - the checks that read this key fail with it
        ctx.m06[key] = e
        ctx.log(f"M6 batch step {key}: {e!r}")


def shown(ctx, what, answer):
    """Logs a reload or refusal answer, as evidence for the coverage."""
    ctx.log(f"M6 {what}: {(answer.stdout.strip() or answer.stderr.strip())[:1200]}")
    return answer


def got(ctx, key):
    assert key in ctx.m06, f"the M6 batch did not reach {key}"
    value = ctx.m06[key]
    if isinstance(value, Exception):
        raise AssertionError(f"the M6 batch step {key} failed: {value!r}")
    return value


def m6check(name, needs=()):
    return check(name, "session", needs, setup=SETUP)


# ---------------------------------------------------------------- the batch

@batch(SETUP)
def m06_batch(ctx, checks):
    ctx.m06 = {}
    if not any(c.name.split(" ")[0] in ("US-011", "US-012", "US-040", "US-055", "US-056", "US-057", "NFR-PF-02")
               for c in checks):
        return
    edit = ctx.project / "tests" / "m06" / "US-040-edit.sqf"
    original_check = edit.read_bytes()
    with Mcp(ctx.project, ctx.install, ctx.log) as m:
        step(ctx, "us040", lambda: us040(ctx, m, edit, original_check))
        edit.write_bytes(original_check)
        step(ctx, "us011", lambda: us011(ctx))
        step(ctx, "us056", lambda: us056(ctx))
        if PATHS["new"].exists():
            PATHS["new"].unlink()
        step(ctx, "us012", lambda: us012(ctx))
        step(ctx, "us055", lambda: us055(ctx))
        step(ctx, "us057", lambda: us057(ctx, m))
    set_staged("1")
    durations = ctx.m06_keep.setdefault("durations", [])
    for key in ("us056", "us055"):
        v = ctx.m06.get(key)
        if isinstance(v, dict) and isinstance(v.get("reload"), dict) and v["reload"].get("duration_s") is not None:
            durations.append((key, v["reload"]["duration_s"]))
    ctx.log(f"M6 game-restart reload durations so far: {durations}")


def us040(ctx, m, edit, original):
    before = starts(status(ctx))
    first = ctx.run([EDIT])
    edited = original.replace(b'[1, "==", 1,', b'[1, "==", 2,')
    assert edited != original, "the US-040 check holds no `[1, \"==\", 1,` to edit"
    edit.write_bytes(edited)
    saved = time.time()
    second = ctx.run([EDIT])
    after = starts(status(ctx))
    edit.write_bytes(original)
    cli = ctx.sh("reload", rel(edit, ctx), "--json", ceiling=120)
    tool, seconds = m.call("reload", {"files": [Path(rel(edit, ctx)).as_posix()]})
    return {"before": before, "first": first, "second": second, "saved": saved, "after": after,
            "cli": cli, "tool": tool, "tool_s": seconds, "path": edit}


def us011(ctx):
    before = starts(status(ctx))
    answer, out, result = ctx.run([SET_VAR, RESET])
    after = status(ctx)
    return {"before": before, "answer": answer, "result": result, "after": after}


def us056(ctx):
    before = starts(status(ctx))
    PATHS["new"].write_text('"new"\n', encoding="utf-8")
    answer = shown(ctx, "US-056 reload", ctx.sh("reload", rel(PATHS["new"], ctx), rel(PATHS["config"], ctx), "--json", ceiling=CEILING))
    after = status(ctx)
    return {"before": before, "answer": answer, "reload": answer.json() if answer.stdout.strip() else None,
            "after": after}


def us012(ctx):
    before = starts(status(ctx))
    args = ("run", R12, AFTER12, "--json")
    bg = ctx.sh.start(*args)
    killed = kill_when_new(ctx, bg, "p2", before.get("p2", 0))
    answer = finish(ctx, bg, args)
    out, result = add_run(ctx, answer)
    broken = status(ctx)
    back = ctx.sh("reload", "--json", ceiling=CEILING)
    after = status(ctx)
    return {"before": before, "killed": killed, "answer": answer, "result": result, "broken": broken,
            "back": back, "back_reload": back.json() if back.stdout.strip() else None, "after": after}


def us055(ctx):
    before = starts(status(ctx))
    set_staged("2")
    answer = shown(ctx, "US-055 reload", ctx.sh("reload", "--json", ceiling=CEILING))
    installed = PATHS["installed"].read_bytes()
    after = status(ctx)
    run = ctx.run([V2])
    return {"before": before, "answer": answer, "reload": answer.json() if answer.stdout.strip() else None,
            "after": after, "run": run, "installed": installed}


def us057(ctx, m):
    before = starts(status(ctx))
    set_staged("1")
    args = ("reload", "--json")
    bg = ctx.sh.start(*args)
    killed = kill_when_new(ctx, bg, "p2", before.get("p2", 0))
    answer = shown(ctx, "US-057 failed reload", finish(ctx, bg, args))
    broken = status(ctx)
    ctx.log(f"M6 US-057 status session: {json.dumps({k: (broken.get('session') or {}).get(k) for k in ('state', 'reason')})[:800]}")
    broken_tool = structured(m.call("status")[0])
    refused = shown(ctx, "US-057 refused run", ctx.sh("run", V1, "--json", ceiling=120))
    refused_tool = m.call("run_start", {"ids": [V1]})[0]
    # The way back through the reload tool, which answers within 60 s, applying or done.
    t0 = time.monotonic()
    first, first_s = m.call("reload", {}, ceiling=70)
    first = structured(first)
    last = first
    for _ in range(60):
        if last.get("state") != "applying":
            break
        time.sleep(5)
        last = structured(m.call("status")[0]).get("last_reload") or {}
    back_s = time.monotonic() - t0
    ctx.log(f"M6 US-057 reload tool: first {json.dumps(first)[:600]}; last {json.dumps(last)[:600]}")
    after = status(ctx)
    run = ctx.run([V1])
    return {"before": before, "killed": killed, "answer": answer,
            "reload": answer.json() if answer.stdout.strip() else None, "broken": broken,
            "broken_tool": broken_tool, "refused": refused, "refused_tool": refused_tool,
            "first": first, "first_s": first_s, "last": last, "back_s": back_s, "after": after, "run": run,
            "installed": PATHS["installed"].read_bytes()}


# ---------------------------------------------------------------- judging helpers

def text_of(answer):
    return (answer.stdout + "\n" + answer.stderr)


def all_ready_one_higher(st, before, who=ALL):
    s, now = starts(st), states(st)
    assert (st.get("session") or {}).get("state") == "ready", f"the session is {(st.get('session') or {}).get('state')}"
    bad = [f"{n}: start {s.get(n)} {now.get(n)}, was {before.get(n)}" for n in who
           if now.get(n) != "ready" or s.get(n) is None or before.get(n) is None or s[n] <= before[n]]
    assert not bad, f"not every machine is ready with a higher start number: {bad}"
    return s


def done_game_restart(r, what):
    assert r is not None, f"{what}: no Reload JSON"
    assert r.get("state") == "done", f"{what}: state {r.get('state')}, reason {r.get('reason')}"
    assert r.get("level") == "game-restart", f"{what}: level {r.get('level')}"
    assert sorted(r.get("machines") or []) == sorted(ALL), f"{what}: machines {r.get('machines')}, expected {ALL}"
    assert isinstance(r.get("duration_s"), (int, float)) and r["duration_s"] > 0, f"{what}: duration_s {r.get('duration_s')}"
    assert r.get("id") and r.get("started_at"), f"{what}: no id or started_at: {r}"


# ---------------------------------------------------------------- US-040

@m6check("US-040 RLD-04 API-02 a check edited from == 1 to == 2 and run again fails with the new expectation in its message, and every machine keeps its start number", [EDIT])
def us040_edit(ctx):
    v = got(ctx, "us040")
    _, _, r1 = v["first"]
    _, _, r2 = v["second"]
    e1, e2 = execution(r1, EDIT), execution(r2, EDIT)
    assert e1["outcome"] == "pass", f"the first run: {e1['outcome']} {e1['reason']}"
    assert e2["outcome"] == "fail" and e2["reason"]["kind"] == "assertion", f"the edited run: {e2['outcome']} {e2['reason']}"
    msg = e2["reason"]["message"]
    assert "2" in msg and "acc_m06 the edited expectation" in msg, f"the failure message does not show the expectation 2: {msg!r}"
    s1 = {(m["name"], m["start"]) for m in r1["session"]["machines"]}
    s2 = {(m["name"], m["start"]) for m in r2["session"]["machines"]}
    assert s1 == s2, f"the machine starts differ between the runs: {sorted(s1)} then {sorted(s2)}"
    assert v["before"] == v["after"], f"status start numbers {v['before']} then {v['after']}"


@m6check("US-040 RLD-01 RLD-02 shakedown reload naming only a check file reports level none with the reason that checks are read fresh for every execution, and the reload tool says the same", [EDIT])
def us040_reload_none(ctx):
    v = got(ctx, "us040")
    cli = v["cli"]
    assert cli.code == 0, f"{cli}: {text_of(cli)[:400]}"
    for what, r in (("command", cli.json()), ("tool", structured(v["tool"]))):
        assert r.get("level") == "none" and r.get("state") == "done", f"{what}: level {r.get('level')} state {r.get('state')}"
        f = file_entry(r, v["path"])
        assert f["level"] == "none", f"{what}: the file's level is {f['level']}"
        assert "read fresh" in f["reason"].lower(), f"{what}: the reason does not say checks are read fresh: {f['reason']!r}"
    assert v["tool_s"] < 60, f"the reload tool answered in {v['tool_s']:.1f} s"


@m6check("US-040 NFR-PF-01 on a ready dedicated session the time from saving the edited check to its outcome is under 30 s", [EDIT])
def us040_time(ctx):
    v = got(ctx, "us040")
    e2 = execution(v["second"][2], EDIT)
    at = (e2["reason"] or {}).get("at") or e2["ended_at"]
    took = utc(at) - v["saved"]
    ctx.log(f"US-040: {took:.1f} s from saving the edit to the outcome")
    assert 0 <= took < 30, f"{took:.1f} s from saving the edit to the outcome"


# ---------------------------------------------------------------- US-011

@m6check("US-011 RST-06 API-01 after a check set a public missionNamespace variable, a check with @reset mission sees it nil on every machine and passes", [SET_VAR, RESET])
def us011_nil(ctx):
    v = got(ctx, "us011")
    r = v["result"]
    for cid in (SET_VAR, RESET):
        e = execution(r, cid)
        assert e["outcome"] == "pass", f"{cid}: {e['outcome']} {e['reason']}"
    assert v["answer"].code == 0, f"{v['answer']}"


@m6check("US-011 RST-06 RST-02 the result file lists each client and headless client at its old start and one higher, the server once, every machine's last start ready, and the session ready after", [SET_VAR, RESET])
def us011_starts(ctx):
    v = got(ctx, "us011")
    r = v["result"]
    errors = ctx.validate(r)
    assert not errors, f"the result file is not valid: {errors[:5]}"
    by = {}
    for m in r["session"]["machines"]:
        by.setdefault(m["name"], []).append(m)
    for name in ("p1", "p2", "hc1"):
        got_starts = sorted(m["start"] for m in by.get(name, []))
        s = v["before"][name]
        assert got_starts == [s, s + 1], f"{name}: the result file lists starts {got_starts}, expected {[s, s + 1]}"
        last = max(by[name], key=lambda m: m["start"])
        assert last["end_state"] == "ready", f"{name} start {last['start']} ends {last['end_state']}"
    assert [m["start"] for m in by.get("server", [])] == [v["before"]["server"]], f"server starts {by.get('server')}"
    assert by["server"][0]["end_state"] == "ready", f"server ends {by['server'][0]['end_state']}"
    st = v["after"]
    all_ready_one_higher(st, v["before"], ("p1", "p2", "hc1"))
    assert starts(st)["server"] == v["before"]["server"], f"the server's start changed: {starts(st)}"


# ---------------------------------------------------------------- US-012

@m6check("US-012 RST-06 P-05 INS-04 p2 ended during the mission restart before a @reset mission check: that check ends invalid reset-failed naming p2, the next one invalid session-broken", [R12, AFTER12])
def us012_invalid(ctx):
    v = got(ctx, "us012")
    pid, when = v["killed"]
    assert pid, f"p2 was not ended during the mission restart: {when}"
    r = v["result"]
    e = execution(r, R12)
    assert e["outcome"] == "invalid" and e["reason"]["kind"] == "reset-failed", f"{R12}: {e['outcome']} {e['reason']}"
    assert "p2" in e["reason"]["message"], f"the message does not name p2: {e['reason']['message']!r}"
    a = execution(r, AFTER12)
    assert a["outcome"] == "invalid" and a["reason"]["kind"] == "session-broken", f"{AFTER12}: {a['outcome']} {a['reason']}"
    st = v["broken"]["session"] or {}
    assert st.get("state") == "broken", f"status after the run shows the session {st.get('state')}"


@m6check("US-012 NFR-RL-01 the run whose mission restart lost p2 exits 2 and its result file is valid against result.schema.json", [R12, AFTER12])
def us012_exit(ctx):
    v = got(ctx, "us012")
    assert v["answer"].code == 2, f"{v['answer']}: {text_of(v['answer'])[:300]}"
    errors = ctx.validate(v["result"])
    assert not errors, f"the result file is not valid: {errors[:5]}"


@m6check("US-012 RLD-01 shakedown reload of the session broken by the mission restart applies game-restart and brings every machine back ready with a higher start number", [R12, AFTER12])
def us012_back(ctx):
    v = got(ctx, "us012")
    assert v["back"].code == 0, f"{v['back']}: {text_of(v['back'])[:400]}"
    done_game_restart(v["back_reload"], "the reload of the broken session")
    all_ready_one_higher(v["after"], v["before"])


# ---------------------------------------------------------------- US-056

@m6check("US-056 RLD-02 RLD-06 shakedown reload of a new .sqf not in the build reports game-restart for it with the reason that the mod must be rebuilt, because file patching finds only files in the built PBO")
def us056_new(ctx):
    v = got(ctx, "us056")
    a = v["answer"]
    assert a.code == 0, f"{a}: {text_of(a)[:400]}"
    r = v["reload"]
    f = file_entry(r, PATHS["new"])
    reason = f["reason"].lower()
    assert f["level"] == "game-restart", f"fn_new.sqf has level {f['level']}"
    assert "rebuil" in reason and "pbo" in reason and "file patching" in reason, \
        f"the reason does not say the mod must be rebuilt because file patching finds only files in the built PBO: {f['reason']!r}"


@m6check("US-056 RLD-02 NFR-US-01 a reload naming config.cpp reports game-restart with the reason that config is read at game start, and every machine comes back ready with a start number one higher")
def us056_config(ctx):
    v = got(ctx, "us056")
    r = v["reload"]
    done_game_restart(r, "the config.cpp reload")
    f = file_entry(r, PATHS["config"])
    assert f["level"] == "game-restart", f"config.cpp has level {f['level']}"
    assert "config is read at game start" in f["reason"].lower(), f"the reason: {f['reason']!r}"
    s = all_ready_one_higher(v["after"], v["before"])
    off = {n: (v["before"][n], s[n]) for n in ALL if s[n] != v["before"][n] + 1}
    assert not off, f"start numbers not one higher (before, after): {off}"


# ---------------------------------------------------------------- US-055

@m6check("US-055 RLD-02 RLD-03 RLD-06 FIX-03 the staged fnc_version.sqf changed to \"2\" and shakedown reload reports game-restart, the file with the reasons new process and rebuild, the machines and the time", [V2])
def us055_reload(ctx):
    v = got(ctx, "us055")
    a = v["answer"]
    assert a.code == 0, f"{a}: {text_of(a)[:400]}"
    r = v["reload"]
    done_game_restart(r, "the reload of the staged change")
    f = file_entry(r, PATHS["staged"])
    reason = f["reason"].lower()
    assert f["level"] == "game-restart", f"the file has level {f['level']}"
    assert "new process" in reason, f"the reason does not say the game reads a changed file only in a new process: {f['reason']!r}"
    assert "rebuild" in reason, f"the reason does not say the mod's rebuild runs: {f['reason']!r}"
    assert v["installed"] == PATHS["original"].replace(b'"1"', b'"2"'), "the rebuild did not copy the staged file into the installed copy"


@m6check("US-055 RST-02 NFR-RL-04 after the reload skdfx_cba_fnc_version returns \"2\" on every machine, the session is ready, the next check passes from the start state, and every machine has a start number one higher in its result file", [V2])
def us055_after(ctx):
    v = got(ctx, "us055")
    s = all_ready_one_higher(v["after"], v["before"])
    off = {n: (v["before"][n], s[n]) for n in ALL if s[n] != v["before"][n] + 1}
    assert not off, f"start numbers not one higher (before, after): {off}"
    _, _, r = v["run"]
    e = execution(r, V2)
    assert e["outcome"] == "pass", f"{V2}: {e['outcome']} {e['reason']}"
    in_file = {m["name"]: m["start"] for m in r["session"]["machines"]}
    off = {n: (v["before"][n], in_file.get(n)) for n in ALL if in_file.get(n) != v["before"][n] + 1}
    assert not off, f"the result file's start numbers are not one higher (before, in file): {off}"


@m6check("NFR-PF-02 US-055 every game-restart reload of the round, on a dedicated session with two clients and a headless client, took under 90 s plus its one-file rebuild by duration_s", [V2])
def nfr_pf_02(ctx):
    d = ctx.m06_keep.get("durations", [])
    assert d, "no game-restart reload reported a duration"
    over = [(k, s) for k, s in d if s >= NFR_PF_02 + 1]
    assert not over, f"reloads over {NFR_PF_02} s plus the rebuild: {over}; all: {d}"


# ---------------------------------------------------------------- US-057

@m6check("US-057 RLD-03 DM-05 P-08 INS-04 p2 ended during a game-restart reload of a changed .sqf: reload exits 4 with state failed naming the step and p2, and status shows the session broken with that reason", [V1])
def us057_failed(ctx):
    v = got(ctx, "us057")
    pid, when = v["killed"]
    assert pid, f"p2 was not ended during the reload: {when}"
    a = v["answer"]
    assert a.code == 4, f"{a}, expected exit 4: {text_of(a)[:400]}"
    r = v["reload"]
    assert r and r.get("state") == "failed", f"the reload JSON: {str(r)[:400]}"
    reason = r.get("reason") or {}
    assert reason.get("machine") == "p2", f"the reason names machine {reason.get('machine')!r}: {reason}"
    assert reason.get("step"), f"the reason names no step: {reason}"
    assert "game-restart" in str(reason.get("step")), f"the step is not the level it was applying: {reason}"
    for what, st in (("command", v["broken"]), ("tool", v["broken_tool"])):
        s = st.get("session") or {}
        assert s.get("state") == "broken", f"status ({what}) shows the session {s.get('state')}"
        sr = s.get("reason") or {}
        assert sr.get("machine") == "p2" or "p2" in str(sr.get("message")), f"status ({what}) reason does not name p2: {sr}"


@m6check("US-057 NFR-RL-01 NFR-US-01 a run asked on the session the failed reload broke exits 4 saying the session is broken and naming shakedown reload, and run_start returns session-broken", [V1])
def us057_refused(ctx):
    v = got(ctx, "us057")
    a = v["refused"]
    said = text_of(a)
    assert a.code == 4, f"{a}, expected exit 4: {said[:400]}"
    assert "broken" in said.lower() and "shakedown reload" in said, f"the refusal does not say broken and name shakedown reload: {said[:400]}"
    text = tool_error(v["refused_tool"], "session-broken")
    assert "reload" in text, f"run_start's error does not name reload: {text}"


@m6check("US-057 RLD-01 AGT-03 the reload tool on the broken session answers within 60 s and applies game-restart, every machine comes back ready with a new start number, and the next check sees \"1\"", [V1])
def us057_back(ctx):
    v = got(ctx, "us057")
    assert v["first_s"] < 60, f"the reload tool answered in {v['first_s']:.1f} s"
    assert v["first"].get("state") in ("applying", "done"), f"the reload tool answered state {v['first'].get('state')}"
    assert v["first"].get("level") == "game-restart", f"the reload tool gives level {v['first'].get('level')}"
    last = v["last"]
    assert last.get("id") == v["first"].get("id"), f"status's last_reload is {last.get('id')}, the tool's {v['first'].get('id')}"
    done_game_restart(last, "the reload of the broken session")
    all_ready_one_higher(v["after"], v["before"])
    assert v["installed"] == PATHS["original"], "the installed fnc_version.sqf is not \"1\" again after the rebuilds"
    e = execution(v["run"][2], V1)
    assert e["outcome"] == "pass", f"{V1}: {e['outcome']} {e['reason']}"
