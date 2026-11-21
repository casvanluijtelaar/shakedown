r"""The checks of M17 ACE interaction: US-089 and US-090's acceptance criteria.

Once per repeat, with no session running, in the project `m17` of its own in the work folder, whose `tests\` holds
the checks of `acceptance/m17/` and whose `shakedown.toml` names CBA, both fixtures and ACE3 (from the Steam Workshop,
item 463939057, or the `SHAKEDOWN_ACE` variable, with `patch = "ace_main"`). Its setups are `ded-ace` (a dedicated
server with two clients loading CBA, both fixtures and ACE), `sp-ace` (singleplayer, the same mods) and `sp-noace`
(singleplayer, CBA and both fixtures, no ACE). One `run --setup <setup> --json` of that setup's checks per setup,
each starting and stopping its own session. A run lost to `Error 20 reading file` of a base-game file starts again,
at most three times in all.
"""

import os
import shutil
import time
import traceback
from datetime import datetime
from pathlib import Path

from . import driver
from .core import check, execution
from .m08 import game_folder
from .m11 import result_of
from .m12 import cba_folder
from .m13 import mcp_json, need_quiet, tidy

ACC = Path(__file__).resolve().parent.parent
CRATE, SELF, MISSING, NEVER, NOACE = "US-089-crate", "US-089-self", "US-090-missing", "US-090-never", "US-090-noace"
RUNS = {"ded-ace": [CRATE, SELF, MISSING], "sp-ace": [SELF, NEVER], "sp-noace": [NOACE]}


def keep(ctx):
    if not hasattr(ctx, "m17_keep"):
        ctx.m17_keep = {}
    return ctx.m17_keep


def ace_folder(game):
    env = os.environ.get("SHAKEDOWN_ACE")
    return Path(env) if env else Path(game).parent.parent / "workshop" / "content" / "107410" / "463939057"


def project(ctx):
    store = keep(ctx)
    if "project" not in store:
        game = game_folder(ctx.project)
        root = ctx.work / "m17"
        if root.exists():
            shutil.rmtree(root)
        shutil.copytree(ACC / "m17", root / "tests")
        mcp_json(root)
        fixture_cba = Path(ctx.install) / "fixtures" / "@skd_fixture_cba"
        (root / "shakedown.toml").write_text(
            f"game = '{game}'\n\n"
            f"[mods.cba]\npath = '{cba_folder(ctx.project)}'\npatch = \"cba_main\"\n\n"
            f"[mods.fixture]\npath = '{game / '@skd_fixture'}'\npatch = \"skdfx_main\"\n\n"
            f"[mods.fixture-cba]\npath = '{fixture_cba}'\npatch = \"skdfx_cba\"\n\n"
            f"[mods.ace]\npath = '{ace_folder(game)}'\npatch = \"ace_main\"\n\n"
            "[setups.ded-ace]\nmode = \"dedicated\"\nclients = 2\nmods = [\"cba\", \"fixture\", \"fixture-cba\", \"ace\"]\n\n"
            "[setups.sp-ace]\nmode = \"singleplayer\"\nmods = [\"cba\", \"fixture\", \"fixture-cba\", \"ace\"]\n\n"
            "[setups.sp-noace]\nmode = \"singleplayer\"\nmods = [\"cba\", \"fixture\", \"fixture-cba\"]\n",
            encoding="utf-8")
        store["project"] = (root, driver.Shakedown(ctx.sh.exe, root, ctx.log))
    return store["project"]


def machine_fault(root, since):
    sessions = root / ".shakedown" / "sessions"
    for rpt in sessions.glob("*/machines/*/*.rpt") if sessions.is_dir() else []:
        if rpt.stat().st_mtime >= since - 1:
            for line in rpt.read_text(encoding="utf-8", errors="replace").splitlines():
                if "Error 20 reading file" in line:
                    return line.strip()[:200]
    return None


def setup_run(ctx, setup):
    """`run --setup <setup> --json` of that setup's checks, once per repeat; the runs of a repeat share it."""
    store = keep(ctx)
    k = (setup, getattr(ctx, "repeat_n", 1))
    if k not in store:
        root, sh = project(ctx)
        for attempt in range(1, 4):
            began = time.time()
            pids = set()
            try:
                need_quiet(sh)
                before = driver.game_pids()
                answer = sh("run", *RUNS[setup], "--setup", setup, "--json", ceiling=900)
                pids |= driver.game_pids() - before
                got = result_of(answer)
                got["seconds"] = answer.seconds
                ctx.log(f"M17 {setup}: {answer}")
                store[k] = got
                break
            except Exception as e:  # noqa: BLE001
                ctx.log(f"M17 {setup}: {traceback.format_exc()}")
                store[k] = e
                fault = machine_fault(root, began)
                if not fault or attempt == 3:
                    break
                ctx.log(f"M17 {setup}: a game process could not read a base-game file ({fault}); starting again, "
                        f"attempt {attempt + 1} of 3")
            finally:
                tidy(ctx, sh, pids)
    value = store[k]
    if isinstance(value, Exception):
        raise AssertionError(f"the M17 run of {setup} failed: {value!r}")
    return value


def utc(text):
    return datetime.fromisoformat(text.replace("Z", "+00:00")).timestamp()


def valid(ctx, r, setup):
    errors = ctx.validate(r["result"])
    assert not errors, f"{setup}: the result file is not valid against result.schema.json: {errors[:5]}"


def ended(r, cid):
    e = execution(r["result"], cid)
    reason = e.get("reason") or {}
    return e, (e["outcome"], reason.get("kind")), reason.get("message") or ""


def m17check(name):
    return check(name, "start", repeat=True)


# ================================================================ US-089

@m17check("US-089 ACT-04 INT-ACE-03 with ACE on a dedicated session with two clients, p1 opens the crate through skd_fnc_useAceAction with the path ACE_MainActions > skdfx_aceOpen, and the server and both clients see it open")
def us089_crate(ctx):
    r = setup_run(ctx, "ded-ace")
    valid(ctx, r, "ded-ace")
    e, k, msg = ended(r, CRATE)
    assert k == ("pass", None), f"{CRATE} ended {k}: {msg}"


@m17check("US-089 ACT-04 the self action ACE_SelfActions > skdfx_aceSignal used by p1 is counted once on every machine, on a dedicated session with two clients and in singleplayer")
def us089_self(ctx):
    for setup in ("ded-ace", "sp-ace"):
        r = setup_run(ctx, setup)
        valid(ctx, r, setup)
        e, k, msg = ended(r, SELF)
        assert k == ("pass", None), f"{SELF} in {setup} ended {k}: {msg}"


@m17check("US-089 RES-01 INT-ACE-01 the result file records the ACE version each machine loaded, as ace_main, on the dedicated session and in singleplayer")
def us089_versions(ctx):
    seen = {}
    for setup, names in (("ded-ace", ["server", "p1", "p2"]), ("sp-ace", ["p1"])):
        r = setup_run(ctx, setup)
        machines = r["result"]["session"]["machines"]
        assert [m["name"] for m in machines] == names, f"{setup}: session.machines {[m['name'] for m in machines]}"
        for m in machines:
            ace = [x for x in m["mods"] if x["mod"] == "ace"]
            assert len(ace) == 1 and ace[0]["patch"] == "ace_main" and ace[0]["version"], \
                f"{setup}: {m['name']} records {ace} for the mod ace"
            seen[f"{setup}/{m['name']}"] = ace[0]["version"]
    ctx.log(f"M17 US-089 3: the ACE versions recorded: {seen}")
    assert len(set(seen.values())) == 1, f"the machines record different ACE versions: {seen}"


# ================================================================ US-090

def at_ceiling(e, ceiling):
    took = utc(e["reason"]["at"]) - utc(e["started_at"])
    assert ceiling - 0.2 <= took <= ceiling + 5, f"the failure came {took:.1f} s after the check started, ceiling {ceiling} s"
    return took


@m17check("US-090 ACT-04 DM-03 a path whose last name skdfx_noSuchAction does not exist ends fail assertion at the 3 s ceiling, naming the name and listing the actions at ACE_MainActions, skdfx_aceOpen among them, with reason.machine p1")
def us090_missing(ctx):
    r = setup_run(ctx, "ded-ace")
    e, k, msg = ended(r, MISSING)
    assert k == ("fail", "assertion"), f"{MISSING} ended {k}: {msg}"
    for word in ("skdfx_noSuchAction", "ACE_MainActions", "skdfx_aceOpen"):
        assert word in msg, f"the message does not name {word}: {msg!r}"
    assert e["reason"].get("machine") == "p1", f"reason.machine {e['reason'].get('machine')!r}"
    ctx.log(f"M17 US-090 1: {at_ceiling(e, 3):.1f} s; {msg}")


@m17check("US-090 ACT-04 DM-03 the self action skdfx_aceNever, whose condition is always false, ends fail assertion at the 3 s ceiling saying its condition was false")
def us090_never(ctx):
    r = setup_run(ctx, "sp-ace")
    e, k, msg = ended(r, NEVER)
    assert k == ("fail", "assertion"), f"{NEVER} ended {k}: {msg}"
    assert "skdfx_aceNever" in msg and "condition" in msg.lower() and "false" in msg.lower(), \
        f"the message does not say the condition of skdfx_aceNever was false: {msg!r}"
    ctx.log(f"M17 US-090 2: {at_ceiling(e, 3):.1f} s; {msg}")


@m17check("US-090 ACT-04 DM-04 skd_fnc_useAceAction on a singleplayer machine without ACE ends fail check-error at once, before its 5 s ceiling, saying ACE is not loaded")
def us090_noace(ctx):
    r = setup_run(ctx, "sp-noace")
    valid(ctx, r, "sp-noace")
    e, k, msg = ended(r, NOACE)
    assert k == ("fail", "check-error"), f"{NOACE} ended {k}: {msg}"
    assert "ace is not loaded" in msg.lower(), f"the message does not say ACE is not loaded: {msg!r}"
    took = utc(e["reason"]["at"]) - utc(e["started_at"])
    ctx.log(f"M17 US-090 3: {took:.1f} s; {msg}")
    assert took < 4, f"the check-error came {took:.1f} s after the check started, not at once"
