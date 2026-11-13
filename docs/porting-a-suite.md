---
type: Documentation
title: Porting an existing suite
description: How to move a mod's existing checks, driven by PowerShell scripts that start the game and read its RPT, to Shakedown - what each part of the old suite becomes, the order of the work, how to rewrite one check, and how to show that the port reaches the same verdicts.
status: draft
---

# Porting an existing suite

Many Arma 3 mods test themselves with a set of PowerShell scripts: a script copies a test mission or an `init.sqf` into place, starts `arma3_x64.exe` with the mod and `-autotest` or a mission, sleeps, reads the RPT for lines such as `TEST PASS` or `TEST FAIL`, and kills the game. This page is for the person who moves such a suite to Shakedown. It uses only what [the contract](../spec/contract/index.md) and the other pages of these docs say; each section points to the page with the details.

The short version: the PowerShell goes away. What stays is the SQF that sets a scene, makes the player act and reads the outcome; it moves into one check file per check, and its waits, verdicts and logs become calls to the check API.

## What each part becomes

| In the PowerShell suite | In Shakedown |
|---|---|
| The script that finds the game, lists the mods and builds the `-mod=` line | `shakedown.toml`: `game`, one `[mods.<name>]` per mod with its folder, and one `[setups.<name>]` per combination of mods and mode ([The project file](project-file.md)). |
| A variant of the launcher for CBA, CBA with ACE, or a dedicated server with clients | One setup each, such as `sp-cba`, `sp-ace` and `dedicated` with `clients = 2`. A check names the setups it runs in with `// @setups` ([Writing and running a check](writing-and-running-checks.md#a-check)). |
| A test mission, `init.sqf` or `-autotest` config written per check | Nothing. The session generates its own mission on the setup's `world`, with `p1` at `spawn` ([The project file](project-file.md#keys)). The check builds what it needs with `skd_fnc_create` and `skd_fnc_place` ([The check API by example](check-api.md#building-a-scene)). |
| One game start per check | One session for many checks. Before every check the session is reset in place, in under a second ([The reset between checks](check-api.md#the-reset-between-checks)). |
| `sleep 5` in SQF, or `Start-Sleep` in PowerShell before reading the RPT | `skd_fnc_waitUntil`, `skd_fnc_expectOn` or `skd_fnc_expectEach` with a condition and a ceiling. There is no sleep ([Waits](check-api.md#waits)). |
| `diag_log "TEST PASS"` and a `Select-String` over the RPT | `skd_fnc_expect` and its kin. A check that ends without a failed assertion passes; the verdict is in `result.json` and the exit code, not in the RPT ([Assertions](check-api.md#assertions)). |
| `diag_log` of values to read after a failure | `skd_fnc_evidence`, kept in the result for every outcome, and each machine's log lines for the check in `checks\<n>\<machine>.log` ([Script errors, logs and evidence](script-errors-and-logs.md)). |
| A scan of the RPT for `Error in expression` | Automatic: a script error on any machine fails the check, traced to its machine and step. An error the check means to cause is wrapped in `skd_fnc_expectError` ([Script errors](check-api.md#script-errors)). |
| Calling the mod's action code, or a key sent with SendKeys | `skd_fnc_useAction` for an action by its title and `skd_fnc_pressKey` for a CBA keybind by its mod and action names. Both go through the mod's real path on the named machine, and nothing touches the desktop's mouse or keyboard ([Acting through the mod](check-api.md#acting-through-the-mod)). |
| A timeout after which the script kills the game | `// @timeout` in the check's header (default 120 s) and the ceiling of each wait. The runner owns every process: a game that hangs or crashes ends the check `invalid` with its cause, not `fail` ([Outcomes and a broken session](outcomes-and-broken-sessions.md)). |
| `Stop-Process -Name arma3_x64` at the end | `shakedown session stop`, or `shakedown run --setup <setup>`, which stops its own session and leaves no game process running ([Running a suite unattended](unattended-runs.md)). |
| A summary the script prints, and its exit code | `run` prints one line per check and the path of `result.json`, valid against [result.schema.json](../spec/contract/result.schema.json). Exit 0 every check passed, 1 a check failed, 2 none failed and one was `invalid` ([Running checks](writing-and-running-checks.md#running-checks)). |

## The order of the work

1. **Install Shakedown** and put its folder on `PATH` ([Installing Shakedown](install.md)).
2. **Write `shakedown.toml`** in the mod's repository, in its own folder or at the root. Point `mods.<name>.path` at the build the old suite loaded, such as HEMTT's `.hemttout\build`, and at the Workshop folders of CBA_A3 and ACE3. Give each mod its main `CfgPatches` class as `patch`, so every result file records the version the game loaded. Add `source`, `prefix` and `rebuild` for the mod itself if you want `shakedown reload` ([Reloading and the mission restart](reloading.md#setting-a-mod-up-for-reload)).
3. **Start one session per setup by hand** and see it come up: `shakedown session start --setup sp-cba`, then `shakedown status`. A setup that does not start says why in `status`.
4. **Port one check**, the simplest, as the next section shows, and run it in the session that is up: `shakedown run <id>`. Edit and rerun; the runner reads the check from disk each time, so no restart is needed.
5. **Port the rest**, one file per old check. `shakedown list` reads every header without the game and names any header mistake.
6. **Compare the verdicts** with the old suite's, as the last section says.
7. **Replace the PowerShell entry point** with one `shakedown run --setup <setup>` per setup.

## Rewriting one check

Take an old check that sets up a scene in SQF, makes the player act, sleeps, and logs a verdict:

```sqf
// init.sqf of the old test mission
player setPosASL [100, 104, 0];
private _crate = "Land_WoodenBox_F" createVehicle [100, 108, 0];
player action ["UseWeapon", player, player, 0];
sleep 5;
if (damage player < 0.1) then { diag_log "TEST PASS unhurt" } else { diag_log "TEST FAIL unhurt" };
```

As a Shakedown check, in `checks\US-012-unhurt.sqf`:

```sqf
// @check US-012 A player who uses the crate is unhurt
// @tags US-012 ported
// @setups sp-cba sp-ace

private _crate = ["build the scene", {
    private _crate = ["Land_WoodenBox_F", AGLToASL [100, 108, 0], 0] call skd_fnc_create;
    ["p1", AGLToASL [100, 104, 0], 0] call skd_fnc_place;
    _crate
}] call skd_fnc_step;

["use the crate through the mod's action", {
    ["p1", _this select 0, "Open crate"] call skd_fnc_useAction;
}, [_crate]] call skd_fnc_step;

["p1 is unhurt", {
    ["p1", { damage player }, "<", 0.1, "p1's damage", 5] call skd_fnc_expectOn;
}] call skd_fnc_step;
```

What changed, and why:

- **The header** names the story or requirement the old check proved, so the result, the tags and a later finding can point at it. Keep the old check's name in `@tags` too if other documents refer to it.
- **Steps** name each phase, so a failure says where it happened.
- **The scene** is made with `skd_fnc_create` and `skd_fnc_place`, which return only once the object exists and the player stands where asked. Positions are ASL. Whatever the check creates, by these functions or by its own SQF, the next reset deletes.
- **The action** is fired as the player's own choice from the scroll menu, after the framework has checked its condition and radius, the way a player would reach it.
- **The sleep and the verdict** become one polled assertion. It passes as soon as the condition holds and fails at its ceiling with the last value it read.

### Points to watch

- **Code sent to a machine does not see the check's locals.** Pass what it needs in `args`, which arrive as `_this`. `skd_fnc_*` functions are called only in the check body, never inside that code ([The check API by example](check-api.md)).
- **No `#include` and no macros** in a check. Write `agh_fnc_placeHook` where the old code wrote `FUNC(placeHook)`.
- **The check body runs on the lead machine**: `p1` in singleplayer, the server on a dedicated session, which has no player. A check that runs in both kinds of setup names a player's unit with `["p1"] call skd_fnc_player` and runs player code with `skd_fnc_on` ([Dedicated sessions](dedicated-sessions.md#machines-and-groups-in-a-check)).
- **State the reset does not touch** stays from one check to the next: variables on units or in `missionNamespace`, actions added to the player, a mod's own state and settings. A check that changes them registers a `skd_fnc_cleanup`. A check that depends on a freshly started mission, as the old suite's one-game-per-check gave it, says `// @reset mission` ([A check that needs a fresh mission](reloading.md#a-check-that-needs-a-fresh-mission)). Start without it, and add it only to a check that passes alone and fails after another.
- **ACE medical**: with ACE loaded, the reset gives every player ACE's full heal, so a check that wounds a player through ACE does not leave the next one wounded ([The reset between checks](check-api.md#the-reset-between-checks)).
- **A check that stops on purpose** for a cause outside the mod, such as a missing dependency, calls `skd_fnc_invalid`, so the run does not count it as the mod's failure.
- **Several machines.** An old dedicated test that read each client's RPT becomes one check with `skd_fnc_expectEach` or `skd_fnc_expectAgree` over the machines it names, which fails naming the machine that differs ([Code, waits and agreement across machines](dedicated-sessions.md#code-waits-and-agreement-across-machines)).

## Showing the port reaches the same verdicts

A port is done when, on the same build of the mod, each ported check reaches the verdict the old check reached. To show it:

1. Build the mod once, and run the old suite on that build. Note each check's verdict.
2. Run the ported suite on the same build, once per setup, each with its own session: `shakedown run --setup sp-cba`, and so on. Each run leaves one result file, which names the mod versions the game loaded.
3. Compare check by check. A ported check that passes where the old one failed, or the other way, is either a mistake in the port or a flaw in the old check; find which before you go on. A check that ends `invalid` names its cause and says nothing about the mod: run it again.
4. Make one check fail on purpose, by changing one expected value, and see it fail with its step and the value it read. A suite in which nothing can fail proves nothing.

An AI agent runs the same suite through the [agent tools](agent-tools.md) with the same outcomes, and can look into a failed check with `inspect`, `exec` and `logs` while the session is still up.
