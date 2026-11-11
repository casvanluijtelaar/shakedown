"""The checks of M7, reload inside the mission: US-054 and US-058.

They run in two sessions, each watching @skd_fixture in a staged copy of its sources in the project, with
a rebuild that copies the staged fn_version.sqf into the installed copy, as spec/contract/fixture-mod.md
gives it for a test that reloads a target the way a mod developer does (FIX-03):

- rld7, a dedicated server with two clients, loading @skd_fixture alone, as the M7 demo has it;
- rld7sp, singleplayer, loading @skd_fixture alone.

Per repeat the batch of each session drives, in this order:

1. US-058: two checks run with no reload asked, then the same two again; while the first of them waits,
   the staged file is changed and `shakedown reload`, `shakedown reset` and the reload and reset tools are
   asked; the staged file is put back before the run ends.
2. US-054: the staged file changed to "2" and `shakedown reload` with no files; while it applies, a reset
   and the reload tool are asked; then the three checks tagged m07-demo run by their tag. Then the staged
   file put back to "1", reloaded, and a check sees "1".

Every repeat ends with the installed copy as it was. After each session stops, the installed copy is put
back once more, whatever happened.
"""

import shutil
import time
from pathlib import Path

from .core import AFTER_SESSION, batch, check, execution
from .m06 import add_run, finish, same_path, shown, starts, states, status, text_of, utc
from .mcp import Mcp, structured, tool_error

SETUPS7 = {"rld7": "on a dedicated session with two clients", "rld7sp": "in singleplayer"}
TAG = "m07-demo"
DEMO = ["m07/US-054-flag", "m07/US-054-start-state", "m07/US-054-version-2"]  # check id order, as RUN-03 runs a tag
V2, V1 = "m07/US-054-version-2", "m07/US-054-version-1"
SLOW, AFTER = "m07/US-058-slow", "m07/US-058-after"
CEILING = 400
NFR_PF_02 = 90.0            # s, plus the rebuild, which copies one file
REBUILD = 1.0               # s, a ceiling for a copy of one file
PATHS = {}


def project_toml(project, game, install):
    """The staged mod and the two setups of M7."""
    fixtures = Path(install) / "fixtures"
    stage = Path(project) / "stage" / "skdfx" / "addons"
    shutil.copytree(fixtures / "skdfx" / "addons", stage)
    installed = fixtures / "skdfx" / "addons" / "main" / "functions" / "fn_version.sqf"
    PATHS.update(installed=installed, original=installed.read_bytes(),
                 staged=stage / "main" / "functions" / "fn_version.sqf")
    rebuild = f'copy /y "{PATHS["staged"]}" "{installed}"'
    return (
        f"\n[mods.fixture-stage]\npath = '{Path(game) / '@skd_fixture'}'\npatch = \"skdfx_main\"\n"
        f"source = '{stage}'\nprefix = 'skdfx\\addons'\nrebuild = '{rebuild}'\n\n"
        "[setups.rld7]\nmode = \"dedicated\"\nclients = 2\nheadless = 0\nmods = [\"fixture-stage\"]\n\n"
        "[setups.rld7sp]\nmode = \"singleplayer\"\nmods = [\"fixture-stage\"]\n")


def put_back(ctx, setup):
    """FIX-03: the installed copy as it was, once no game process of the session runs."""
    if setup not in SETUPS7 or not PATHS:
        return
    if PATHS["staged"].read_bytes() != PATHS["original"]:
        PATHS["staged"].write_bytes(PATHS["original"])
    if PATHS["installed"].read_bytes() != PATHS["original"]:
        PATHS["installed"].write_bytes(PATHS["original"])
        ctx.log("M7: put the installed fn_version.sqf back as it was")


AFTER_SESSION.append(put_back)


def set_staged(value):
    PATHS["staged"].write_bytes(PATHS["original"].replace(b'"1"', f'"{value}"'.encode()))


def step(ctx, key, fn):
    try:
        ctx.m07[key] = fn()
    except Exception as e:  # noqa: BLE001 - the checks that read this key fail with it
        ctx.m07[key] = e
        ctx.log(f"M7 batch step {key}: {e!r}")


def got(ctx, key):
    assert key in ctx.m07, f"the M7 batch did not reach {key}"
    value = ctx.m07[key]
    if isinstance(value, Exception):
        raise AssertionError(f"the M7 batch step {key} failed: {value!r}")
    return value


def poll(ctx, bg, ceiling, cond):
    """Polls status twice a second until cond(status) holds; returns (status, seconds) or (None, why)."""
    began = time.monotonic()
    for _ in range(int(ceiling * 2)):
        if bg.poll() is not None:
            return None, f"the command ended first, exit {bg.returncode}"
        st = status(ctx)
        if cond(st):
            return st, time.monotonic() - began
        time.sleep(0.5)
    return None, f"not within {ceiling} s"


def outcome_at(e):
    return utc((e.get("reason") or {}).get("at") or e["ended_at"])


# ---------------------------------------------------------------- the batch

def make_batch(setup):
    @batch(setup)
    def m07_batch(ctx, checks):
        ctx.m07 = {}
        if not any(c.name.split(" ")[0] in ("US-054", "US-058", "NFR-PF-02") and c.setup == setup for c in checks):
            return
        set_staged("1")
        with Mcp(ctx.project, ctx.install, ctx.log) as m:
            step(ctx, "us058", lambda: us058(ctx, m))
            set_staged("1")
            step(ctx, "us054", lambda: us054(ctx, m))
        set_staged("1")
        durations = ctx.m07_keep.setdefault(setup, [])
        v = ctx.m07.get("us054")
        if isinstance(v, dict):
            for key in ("reload", "back_reload"):
                r = v.get(key)
                if isinstance(r, dict) and r.get("duration_s") is not None:
                    durations.append(r["duration_s"])
        ctx.log(f"M7 {setup} game-restart reload durations so far: {durations}")
    return m07_batch


for _s in SETUPS7:
    make_batch(_s)


def us058(ctx, m):
    before = starts(status(ctx))
    base = ctx.run([SLOW, AFTER], ceiling=180)
    args = ("run", SLOW, AFTER, "--json")
    bg = ctx.sh.start(*args)
    during, why = poll(ctx, bg, 60, lambda st: (st.get("run") or {}).get("current_check") == SLOW)
    out = {"before": before, "base": base, "during": during, "why": why}
    if during:
        set_staged("2")
        out["reload"] = shown(ctx, "US-058 reload during a run", ctx.sh("reload", "--json", ceiling=60))
        out["reset"] = shown(ctx, "US-058 reset during a run", ctx.sh("reset", "--json", ceiling=60))
        out["reload_tool"], out["reload_tool_s"] = m.call("reload", {}, ceiling=70)
        out["reset_tool"], out["reset_tool_s"] = m.call("reset", {}, ceiling=70)
        ctx.log(f"M7 US-058 tools: reload {str(out['reload_tool'])[:400]}; reset {str(out['reset_tool'])[:400]}")
        out["mid"] = status(ctx)
        set_staged("1")
    answer = finish(ctx, bg, args, ceiling=180)
    out["answer"] = answer
    out["result"] = add_run(ctx, answer)[1]
    out["after"] = status(ctx)
    return out


def us054(ctx, m):
    st = status(ctx)
    before = starts(st)
    set_staged("2")
    t0 = time.time()
    args = ("reload", "--json")
    bg = ctx.sh.start(*args)
    busy, why = poll(ctx, bg, 60, lambda s: (s.get("session") or {}).get("state") == "reloading")
    out = {"before": before, "names": sorted(before), "t0": t0, "busy": busy, "busy_why": why}
    if busy:
        out["busy_reset"] = shown(ctx, "US-054 reset during the reload", ctx.sh("reset", "--json", ceiling=60))
        out["busy_reload"] = shown(ctx, "US-054 reload during the reload", ctx.sh("reload", "--json", ceiling=60))
        out["busy_tool"], out["busy_tool_s"] = m.call("reload", {}, ceiling=70)
        ctx.log(f"M7 US-054 reload tool during the reload: {str(out['busy_tool'])[:400]}")
    answer = shown(ctx, "US-054 reload", finish(ctx, bg, args, ceiling=CEILING))
    out["answer"] = answer
    out["reload"] = answer.json() if answer.stdout.strip() else None
    out["installed"] = PATHS["installed"].read_bytes()
    out["after"] = status(ctx)
    out["run"] = ctx.run([], tags=[TAG], ceiling=180)
    result = out["run"][2]
    ats = [outcome_at(e) for e in result["checks"]]
    out["took"] = (max(ats) - t0) if ats else None
    ctx.log(f"M7 US-054: {out['took']} s from the reload order to the last tagged outcome; "
            f"reload duration_s {(out['reload'] or {}).get('duration_s')}")
    # Put the staged file back and reload, so the next repeat and session start from "1".
    set_staged("1")
    back = shown(ctx, "US-054 reload back to 1", ctx.sh("reload", "--json", ceiling=CEILING))
    out["back"] = back
    out["back_reload"] = back.json() if back.stdout.strip() else None
    out["back_installed"] = PATHS["installed"].read_bytes()
    out["back_run"] = ctx.run([V1], ceiling=120)
    return out


# ---------------------------------------------------------------- judging

def done_game_restart(r, names, what):
    assert r is not None, f"{what}: no Reload JSON"
    assert r.get("state") == "done", f"{what}: state {r.get('state')}, reason {r.get('reason')}"
    assert r.get("level") == "game-restart", f"{what}: level {r.get('level')}"
    assert sorted(r.get("machines") or []) == names, f"{what}: machines {r.get('machines')}, expected {names}"
    assert isinstance(r.get("duration_s"), (int, float)) and r["duration_s"] > 0, f"{what}: duration_s {r.get('duration_s')}"


def one_higher(st, before):
    s, now = starts(st), states(st)
    assert (st.get("session") or {}).get("state") == "ready", f"the session is {(st.get('session') or {}).get('state')}"
    off = {n: (before[n], s.get(n), now.get(n)) for n in before if s.get(n) != before[n] + 1 or now.get(n) != "ready"}
    assert not off, f"machines not ready with a start number one higher (before, after, state): {off}"


def busy_refusal(answer, what, *words):
    said = text_of(answer)
    assert answer.code == 4, f"{what}: {answer}, expected exit 4: {said[:400]}"
    low = said.lower()
    missing = [w for w in words if w.lower() not in low]
    assert not missing, f"{what}: the message does not name {missing}: {said[:500]}"
    return said


def register(setup, label):
    def c(name, needs=()):
        return check(f"{name} {label}", "session", needs, setup=setup)

    @c("US-054 RLD-01 RLD-02 RLD-03 RLD-06 FIX-03 the staged fn_version.sqf changed to \"2\" and shakedown reload with no files exits 0, level game-restart, the file with its reason, every machine reached and the time", [V2])
    def us054_reload(ctx):
        v = got(ctx, "us054")
        a = v["answer"]
        assert a.code == 0, f"{a}: {text_of(a)[:400]}"
        r = v["reload"]
        done_game_restart(r, v["names"], "the reload of the staged change")
        found = [f for f in r.get("files", []) if same_path(f["path"], PATHS["staged"])]
        assert found, f"the reload does not list the staged file; it lists {[f['path'] for f in r.get('files', [])]}"
        f = found[0]
        assert f["level"] == "game-restart", f"the file has level {f['level']}"
        reason = f["reason"].lower()
        assert "new process" in reason and "rebuild" in reason, f"the file's reason: {f['reason']!r}"
        assert v["installed"] == PATHS["original"].replace(b'"1"', b'"2"'), \
            "the rebuild did not copy the staged file into the installed copy"

    @c("US-054 RUN-01 RLD-05 DM-08 after the reload onEach of skdfx_fnc_version gives \"2\" from every machine, every machine has a start number one higher, and the rerun by the finding's tag runs its three checks, each reload full", DEMO)
    def us054_rerun(ctx):
        v = got(ctx, "us054")
        one_higher(v["after"], v["before"])
        answer, _, r = v["run"]
        assert answer.code == 0, f"the run by tag: {answer}: {text_of(answer)[:400]}"
        errors = ctx.validate(r)
        assert not errors, f"the result file is not valid: {errors[:5]}"
        ids = [e["check_id"] for e in r["checks"]]
        assert ids == DEMO, f"`run --tag {TAG}` ran {ids}, expected {DEMO}"
        for e in r["checks"]:
            assert e["outcome"] == "pass", f"{e['check_id']}: {e['outcome']} {e['reason']}"
            assert e.get("reload") == "full", f"{e['check_id']}: reload {e.get('reload')!r}, expected full"
        in_file = {m["name"]: m["start"] for m in r["session"]["machines"]}
        off = {n: (v["before"][n], in_file.get(n)) for n in v["before"] if in_file.get(n) != v["before"][n] + 1}
        assert not off, f"the result file's start numbers are not one higher (before, in file): {off}"

    @c("US-054 NFR-PF-02 the time from shakedown reload to the outcome of the last of the three checks tagged with the finding is within 90 s plus the rebuild plus 10 s", DEMO)
    def us054_time(ctx):
        v = got(ctx, "us054")
        took = v["took"]
        assert took is not None, "the run by tag holds no execution"
        limit = NFR_PF_02 + REBUILD + 10
        assert 0 < took < limit, f"{took:.1f} s from the reload order to the last outcome, limit {limit:.0f} s"

    @c("US-058 RLD-01 RST-03 a reset, a second reload and the reload tool asked while a reload applies are refused, exit 4 and session-busy", [V2])
    def us058_busy_reload(ctx):
        v = got(ctx, "us054")
        assert v["busy"], f"status never showed the session reloading during the reload: {v['busy_why']}"
        busy_refusal(v["busy_reset"], "reset during the reload")
        busy_refusal(v["busy_reload"], "reload during the reload")
        tool_error(v["busy_tool"], "session-busy")
        assert v["busy_tool_s"] < 10, f"the reload tool answered in {v['busy_tool_s']:.1f} s"

    @c("US-058 RLD-01 NFR-US-01 the refusal of a reset, a reload and the reload tool during a reload names the reload in progress, not a reload or a reset", [V2])
    def us058_busy_names(ctx):
        v = got(ctx, "us054")
        assert v["busy"], f"status never showed the session reloading during the reload: {v['busy_why']}"
        said = {"reset": text_of(v["busy_reset"]).strip(), "reload": text_of(v["busy_reload"]).strip(),
                "reload tool": tool_error(v["busy_tool"], "session-busy")}
        vague = {k: t for k, t in said.items() if "reload" not in t.lower() or "a reload or a reset" in t.lower()}
        assert not vague, f"refusals that do not say a reload is in progress: {vague}"

    @c("US-054 FIX-03 the staged file put back to \"1\" and reloaded, every machine sees \"1\" again", [V1])
    def us054_back(ctx):
        v = got(ctx, "us054")
        assert v["back"].code == 0, f"{v['back']}: {text_of(v['back'])[:400]}"
        done_game_restart(v["back_reload"], v["names"], "the reload back to \"1\"")
        assert v["back_installed"] == PATHS["original"], "the installed fn_version.sqf is not \"1\" again"
        e = execution(v["back_run"][2], V1)
        assert e["outcome"] == "pass", f"{V1}: {e['outcome']} {e['reason']}"

    @c("NFR-PF-02 US-054 every game-restart reload of the round took under 90 s plus its one-file rebuild by duration_s", [V2])
    def nfr_pf_02(ctx):
        d = ctx.m07_keep.get(setup, [])
        assert d, "no game-restart reload reported a duration"
        over = [s for s in d if s >= NFR_PF_02 + REBUILD]
        assert not over, f"reloads over {NFR_PF_02} s plus the rebuild: {over}; all: {d}"

    @c("US-058 RLD-01 NFR-US-01 shakedown reload during a run exits 4 naming the run's id, its progress and shakedown run stop, and the reload tool returns session-busy", [SLOW, AFTER])
    def us058_reload(ctx):
        v = got(ctx, "us058")
        assert v["during"], f"status never showed the run at {SLOW}: {v['why']}"
        run_id = v["during"]["run"]["id"]
        total = v["during"]["run"]["total"]
        said = busy_refusal(v["reload"], "reload during the run", run_id, "shakedown run stop", SLOW)
        assert f"of {total}" in said, f"the message does not give the run's progress (done of {total}): {said[:500]}"
        text = tool_error(v["reload_tool"], "session-busy")
        assert run_id in text, f"the reload tool's error does not name the run {run_id}: {text}"
        assert v["reload_tool_s"] < 10, f"the reload tool answered in {v['reload_tool_s']:.1f} s"

    @c("US-058 RST-03 RLD-01 shakedown reset and the reset tool during a run are refused, exit 4 and session-busy, naming the run", [SLOW, AFTER])
    def us058_reset(ctx):
        v = got(ctx, "us058")
        assert v["during"], f"status never showed the run at {SLOW}: {v['why']}"
        run_id = v["during"]["run"]["id"]
        busy_refusal(v["reset"], "reset during the run", run_id, "shakedown run stop")
        text = tool_error(v["reset_tool"], "session-busy")
        assert run_id in text, f"the reset tool's error does not name the run {run_id}: {text}"

    @c("US-058 the run a reload was asked during ends with the same outcomes as the run with no reload, and no machine's start number changes", [SLOW, AFTER])
    def us058_untouched(ctx):
        v = got(ctx, "us058")
        assert v["during"], f"status never showed the run at {SLOW}: {v['why']}"
        assert (v["mid"].get("run") or {}).get("id") == v["during"]["run"]["id"], \
            f"the run is not active after the refusals: {v['mid'].get('run')}"
        base = {e["check_id"]: e["outcome"] for e in v["base"][2]["checks"]}
        now = {e["check_id"]: e["outcome"] for e in v["result"]["checks"]}
        assert base == {SLOW: "pass", AFTER: "pass"}, f"the run with no reload: {base}"
        assert now == base, f"the run with the reload asked: {now}, with none: {base}"
        assert v["answer"].code == v["base"][0].code == 0, f"exit codes {v['base'][0].code} then {v['answer'].code}"
        for what, r in (("no reload", v["base"][2]), ("reload asked", v["result"])):
            in_file = {m["name"]: m["start"] for m in r["session"]["machines"]}
            assert in_file == v["before"], f"the result file ({what}) lists starts {in_file}, status gave {v['before']}"
        assert starts(v["after"]) == v["before"], f"status start numbers {v['before']} then {starts(v['after'])}"


for _s, _label in SETUPS7.items():
    register(_s, _label)
