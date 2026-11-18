r"""The checks of M14, reload without a game restart: US-081 and US-082.

They run in two sessions of the generated project, each loading CBA, @skd_fixture and @skd_fixture_cba,
with @skd_fixture_cba watched in the staged copy of its sources that M6 makes in `stage\`, and a rebuild
that copies the staged fnc_version.sqf into the installed copy, as spec/contract/fixture-mod.md gives it
(FIX-03):

- m14, a dedicated server with two clients, as US-081 1, 2 and 4 have it;
- m14sp, singleplayer, as US-081 3 has it.

Per repeat the batch of each session drives, in this order:

1. US-081: ceil(10 / repeats) times, the staged file changed to a value the session has not had,
   `shakedown reload --json` with no files, and the check `m14/US-081-version` (written from
   `templates/US-081-version.sqf` with the value it expects) run at once. Over a round of three that is
   twelve reloads per session, so US-081 2's "10 of 10" is judged over the round.
2. US-082: `skdfx_fnc_holdFile` of the installed fnc_version.sqf on one machine (the server on the
   dedicated session, p1 in singleplayer), so it is held after the release reads (FIX-04); the staged file
   changed again, `shakedown reload --json`, and the same check.

The installed and the staged file are put back after the session stops.
"""

import math
import os
import re
import time
from pathlib import Path

from . import m06
from .core import AFTER_SESSION, batch, check, execution
from .m06 import same_path, shown, status, text_of
from .m07 import outcome_at

SETUPS14 = {"m14": "on a dedicated session with two clients", "m14sp": "in singleplayer"}
LEAD = {"m14": "server", "m14sp": "p1"}
HOLDER = {"m14": "server", "m14sp": "p1"}
CHECK_ID = "m14/US-081-version"
HELD = r"skdfx_cba\addons\main\functions\fnc_version.sqf"
CEILING = 400
NFR_PF_02 = {"m14": 90.0, "m14sp": 20.0}   # s, a mission-restart reload, plus the rebuild
REBUILD = 1.0                               # s, a ceiling for a copy of one file
PATHS = {}
COUNTER = {}


def project_toml(project, game, install):
    """The two setups of M14 and their check, written as the mod "1" is installed. Needs m06's mods."""
    PATHS.update(game=Path(game), check=Path(project) / "tests" / "m14" / "US-081-version.sqf",
                 template=(Path(__file__).resolve().parent.parent / "templates" / "US-081-version.sqf")
                 .read_text(encoding="utf-8"))
    write_check("1")
    mods = "mods = [\"cba\", \"fixture\", \"fixture-cba-rld\"]\n"
    return ("\n[setups.m14]\nmode = \"dedicated\"\nclients = 2\nheadless = 0\n" + mods
            + "\n[setups.m14sp]\nmode = \"singleplayer\"\n" + mods)


def write_check(value):
    PATHS["check"].parent.mkdir(parents=True, exist_ok=True)
    PATHS["check"].write_text(PATHS["template"].replace("{V}", value), encoding="utf-8")


def put_back(ctx, setup):
    """FIX-03: the staged and installed fnc_version.sqf as they were, once no game process runs."""
    if setup not in SETUPS14 or not m06.PATHS:
        return
    COUNTER.pop(setup, None)
    write_check("1")
    if m06.PATHS["staged"].read_bytes() != m06.PATHS["original"]:
        m06.PATHS["staged"].write_bytes(m06.PATHS["original"])
    if m06.PATHS["installed"].read_bytes() != m06.PATHS["original"]:
        m06.PATHS["installed"].write_bytes(m06.PATHS["original"])
        ctx.log("M14: put the installed fnc_version.sqf back as it was")


AFTER_SESSION.append(put_back)


def machines(st):
    return {m["name"]: m for m in (st.get("session") or {}).get("machines", [])}


def next_value(setup):
    COUNTER[setup] = COUNTER.get(setup, 1) + 1
    return str(COUNTER[setup])


def reload_and_check(ctx, value, before_reload=None):
    """Changes the staged file to `value`, reloads with no files and runs the check at once."""
    before = machines(status(ctx))
    write_check(value)
    out = {"value": value, "before": before}
    if before_reload:
        out.update(before_reload())
    m06.set_staged(value)
    saved = time.time()
    answer = shown(ctx, f"M14 reload to {value}", ctx.sh("reload", "--json", ceiling=CEILING))
    out.update(answer=answer, reload=answer.json() if answer.stdout.strip() else None,
               installed=m06.PATHS["installed"].read_bytes(), after=machines(status(ctx)),
               session_state=(status(ctx).get("session") or {}).get("state"))
    run = ctx.run([CHECK_ID], ceiling=180)
    out["run"] = run
    e = execution(run[2], CHECK_ID)
    out["took"] = outcome_at(e) - saved
    ctx.log(f"M14 reload to {value}: duration_s {(out['reload'] or {}).get('duration_s')}, "
            f"{out['took']:.1f} s from the save to the outcome {e['outcome']}")
    return out


def make_batch(setup):
    @batch(setup)
    def m14_batch(ctx, checks):
        ctx.m14 = {"reloads": [], "held": None}
        if not any(c.setup == setup and ("US-081" in c.name or "US-082" in c.name) for c in checks):
            return
        keep = ctx.m14_keep.setdefault(setup, {"reloads": [], "repeats": 0})
        keep["repeats"] += 1
        for _ in range(math.ceil(10 / max(1, ctx.repeat_total))):
            try:
                v = reload_and_check(ctx, next_value(setup))
            except Exception as e:  # noqa: BLE001 - the checks that read it fail with it
                ctx.log(f"M14 batch reload: {e!r}")
                v = e
            ctx.m14["reloads"].append(v)
            keep["reloads"].append(v)
        holder = HOLDER[setup]

        def hold():
            code = f'["{HELD}"] call skdfx_fnc_holdFile'
            a = ctx.sh("exec", "--machine", holder, "--code", code, "--json", ceiling=30)
            ctx.log(f"M14 holdFile on {holder}: {(a.stdout.strip() or a.stderr.strip())[:400]}")
            return {"hold": a}
        try:
            ctx.m14["held"] = reload_and_check(ctx, next_value(setup), hold)
        except Exception as e:  # noqa: BLE001
            ctx.log(f"M14 batch held: {e!r}")
            ctx.m14["held"] = e
    return m14_batch


for _s in SETUPS14:
    make_batch(_s)


def reloads(ctx):
    rs = ctx.m14.get("reloads") or []
    assert rs, "the M14 batch made no reload this repeat"
    bad = [r for r in rs if isinstance(r, Exception)]
    assert not bad, f"reloads of this repeat failed in the batch: {bad}"
    return rs


def held(ctx):
    v = ctx.m14.get("held")
    assert v is not None, "the M14 batch did not reach the held file"
    if isinstance(v, Exception):
        raise AssertionError(f"the M14 batch step of the held file failed: {v!r}")
    return v


def entry(r):
    found = [f for f in r.get("files", []) if same_path(f["path"], m06.PATHS["staged"])]
    assert found, f"the reload does not list the staged file; it lists {[f['path'] for f in r.get('files', [])]}"
    return found[0]


def passed(v):
    e = execution(v["run"][2], CHECK_ID)
    assert e["outcome"] == "pass", f"the check after the reload to {v['value']}: {e['outcome']} {e['reason']}"
    return e


def register(setup, label):
    names = ["server", "p1", "p2"] if setup == "m14" else ["p1"]
    lead = LEAD[setup]

    def c(name):
        return check(f"{name} {label}", "session", [CHECK_ID], setup=setup)

    @c("US-081 RLD-01 RLD-02 RLD-03 RLD-06 FIX-03 the staged fnc_version.sqf changed and shakedown reload with no files exits 0, level mission-restart, the file with the reason that the game is made to let go of it and a mission restart compiles it and that the rebuild runs, every machine reached and the time, every reload of the repeat")
    def us081_reload(ctx):
        for v in reloads(ctx):
            a, r = v["answer"], v["reload"]
            assert a.code == 0, f"reload to {v['value']}: {a}: {text_of(a)[:400]}"
            assert r and r.get("state") == "done", f"reload to {v['value']}: {str(r)[:400]}"
            assert r.get("level") == "mission-restart", f"reload to {v['value']}: level {r.get('level')}"
            assert sorted(r.get("machines") or []) == sorted(names), f"reload to {v['value']}: machines {r.get('machines')}"
            assert isinstance(r.get("duration_s"), (int, float)) and r["duration_s"] > 0, f"duration_s {r.get('duration_s')}"
            f = entry(r)
            reason = f["reason"].lower()
            assert f["level"] == "mission-restart", f"reload to {v['value']}: the file has level {f['level']}"
            assert "let go" in reason and "mission restart" in reason and "rebuild" in reason, \
                f"reload to {v['value']}: the file's reason: {f['reason']!r}"
            assert v["installed"] == m06.PATHS["original"].replace(b'"1"', f'"{v["value"]}"'.encode()), \
                f"reload to {v['value']}: the rebuild did not copy the staged file into the installed copy"

    @c("US-081 P-08 after each mission-restart reload the lead keeps its process id and start number, every other machine has a start number one higher, and the session is ready")
    def us081_processes(ctx):
        for v in reloads(ctx):
            b, a = v["before"], v["after"]
            assert v["session_state"] == "ready", f"reload to {v['value']}: the session is {v['session_state']}"
            assert a[lead]["pid"] == b[lead]["pid"], f"reload to {v['value']}: {lead} pid {b[lead]['pid']} then {a[lead]['pid']}"
            assert a[lead]["start"] == b[lead]["start"], f"reload to {v['value']}: {lead} start {b[lead]['start']} then {a[lead]['start']}"
            off = {n: (b[n]["start"], a[n]["start"], a[n]["state"]) for n in names if n != lead
                   and (a[n]["start"] != b[n]["start"] + 1 or a[n]["state"] != "ready")}
            assert not off, f"reload to {v['value']}: not ready one higher (before, after, state): {off}"

    @c("US-081 RLD-05 DM-08 the check right after each reload sees the new value on every machine with reload full, 10 of 10 over the round")
    def us081_value(ctx):
        for v in reloads(ctx):
            e = passed(v)
            assert e.get("reload") == "full", f"reload to {v['value']}: the execution's reload is {e.get('reload')!r}; a CBA PREP change recompiles no CfgFunctions function"
        keep = ctx.m14_keep[setup]
        allr = keep["reloads"]
        failed = [r for r in allr if isinstance(r, Exception) or
                  execution(r["run"][2], CHECK_ID)["outcome"] != "pass"]
        assert not failed, f"{len(failed)} of {len(allr)} reloads of the round did not reach the new value"
        if keep["repeats"] >= ctx.repeat_total:
            assert len(allr) >= 10, f"the round made {len(allr)} reloads, fewer than 10"

    @c(f"US-081 NFR-PF-02 every mission-restart reload of the round took under {NFR_PF_02[setup]:.0f} s plus its one-file rebuild by duration_s")
    def us081_target(ctx):
        reloads(ctx)
        d = [r["reload"]["duration_s"] for r in ctx.m14_keep[setup]["reloads"]
             if isinstance(r, dict) and r.get("reload") and r["reload"].get("duration_s") is not None]
        assert d, "no reload reported a duration"
        ctx.log(f"M14 {setup} mission-restart reload durations so far: {d}")
        over = [s for s in d if s >= NFR_PF_02[setup] + REBUILD]
        assert not over, f"reloads over {NFR_PF_02[setup]} s plus the rebuild: {over}; all: {d}"

    if setup == "m14sp":
        @c("US-081 NFR-PF-01 from saving the staged file to the outcome of the check right after, whose body takes under 10 s, under 30 s for every reload")
        def us081_loop(ctx):
            took = [(v["value"], round(v["took"], 1)) for v in reloads(ctx)]
            over = [t for t in took if not 0 < t[1] < 30]
            assert not over, f"save to outcome over 30 s: {over}; all: {took}"

    @c("US-081 RLD-07 the release files are in skd_release of the game folder: 80 junctions l00 to l79 to one folder holding a file of 8 MiB")
    def us081_release(ctx):
        reloads(ctx)
        root = PATHS["game"] / "skd_release"
        assert root.is_dir(), f"no folder {root}"
        targets = set()
        for i in range(80):
            p = root / f"l{i:02}"
            assert p.exists(), f"no {p}"
            assert os.path.isjunction(p), f"{p} is not a directory junction"
            targets.add(os.path.normcase(os.path.realpath(p)))
        assert len(targets) == 1, f"the junctions lead to {len(targets)} folders: {sorted(targets)[:5]}"
        folder = Path(targets.pop())
        sizes = [f.stat().st_size for f in folder.iterdir() if f.is_file()]
        assert 8 * 1024 * 1024 in sizes, f"{folder} holds no file of 8 MiB; sizes {sizes}"

    holder = HOLDER[setup]

    @c(f"US-082 RLD-02 RLD-03 FIX-04 with fnc_version.sqf held by {holder}'s process through skdfx_fnc_holdFile, the reload of its change applies game-restart and the file's entry gives game-restart with a reason naming the file, {holder} and that the game did not let go of it")
    def us082_fallback(ctx):
        v = held(ctx)
        h = v["hold"]
        assert h.code == 0 and h.json()["results"][holder].get("value") is True, \
            f"skdfx_fnc_holdFile on {holder}: {h}: {text_of(h)[:400]}"
        a, r = v["answer"], v["reload"]
        assert a.code == 0, f"{a}: {text_of(a)[:400]}"
        assert r and r.get("state") == "done", f"the reload: {str(r)[:400]}"
        assert r.get("level") == "game-restart", f"the reload applied level {r.get('level')}, expected game-restart"
        assert sorted(r.get("machines") or []) == sorted(names), f"machines {r.get('machines')}"
        f = entry(r)
        assert f["level"] == "game-restart", f"the file's entry has level {f['level']}"
        reason = f["reason"]
        assert "fnc_version.sqf" in reason, f"the reason does not name the file: {reason!r}"
        assert re.search(rf"\b{holder}\b", reason), f"the reason does not name {holder}: {reason!r}"
        assert "let go" in reason.lower() or "held" in reason.lower(), f"the reason does not say the file is held: {reason!r}"

    @c("US-082 P-08 after the fallback every machine is ready with a new process and a start number one higher, and the check after it sees the new value on every machine")
    def us082_after(ctx):
        v = held(ctx)
        b, a = v["before"], v["after"]
        assert v["session_state"] == "ready", f"the session is {v['session_state']}"
        off = {n: (b[n]["start"], a[n]["start"], a[n]["state"]) for n in names
               if a[n]["start"] != b[n]["start"] + 1 or a[n]["state"] != "ready" or a[n]["pid"] == b[n]["pid"]}
        assert not off, f"machines not ready in a new process one start higher (before, after, state): {off}"
        passed(v)


for _s, _label in SETUPS14.items():
    register(_s, _label)
