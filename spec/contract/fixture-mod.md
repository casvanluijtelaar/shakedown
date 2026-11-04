---
type: Specification
title: Fixture mods
description: The contract for the two small mods the framework's own tests and the acceptance suite run against - what each one does, where, what a check can observe, the failures a test causes with them on purpose and the files a test may change - so neither tester nor writer needs their source.
status: draft
---

# Fixture mods

The framework is developed and accepted against two small mods of its own, kept in `fixtures/` (`07-tech-stack.md`). Each behaviour below exists to give one framework capability a target whose result is known in advance, so a failing check points at the framework, not at the mod. `arma-grappling-hook` is the real-world proof at the end of the MVP (PRD section 9); the fixtures are the target for everything before it.

This file is the whole of what the fixtures promise. A check against them is written from this file alone. Behaviour this file does not name is not promised.

| Mod | Folder | Needs | CfgPatches class | Tag |
|---|---|---|---|---|
| `@skd_fixture` | `fixtures\@skd_fixture`, with its extension `skdfx_x64.dll` | nothing but the game | `skdfx_main` | `skdfx` |
| `@skd_fixture_cba` | `fixtures\@skd_fixture_cba` | CBA_A3, and `@skd_fixture` | `skdfx_cba` | `skdfx_cba` |

The two are separate mods so a setup without CBA loads `@skd_fixture` alone; a dedicated server would stop at a missing-addon warning if a CBA-dependent addon were loaded without CBA.

## Where the fixtures are installed

The fixtures are installed as a **copy** in the install folder, beside `@shakedown`: `fixtures\@skd_fixture`, `fixtures\@skd_fixture_cba`, and their unpacked file trees `fixtures\skdfx\` and `fixtures\skdfx_cba\`. The install links `<game>\skdfx` and `<game>\skdfx_cba` to those trees, so file patching reads them, and `<game>\@skd_fixture` to the installed `fixtures\@skd_fixture`, because a dedicated server and a headless client load an extension only from inside the game's folder tree (`INT-AR-06`). The installed copy is not the fixtures' source in `fixtures/`; editing it changes nothing in the repository.

A project uses them like any mod:

```toml
[mods.fixture]
path = '<game>\@skd_fixture'
patch = "skdfx_main"
source = '<install>\fixtures\skdfx\addons'
prefix = 'skdfx\addons'

[mods.fixture-cba]
path = '<install>\fixtures\@skd_fixture_cba'
patch = "skdfx_cba"
source = '<install>\fixtures\skdfx_cba\addons'
prefix = 'skdfx_cba\addons'
```

## `@skd_fixture`

### The crate and its action (`ACT-01`, `CTL-01`, `OBS-06`; PRD scenario "as planned" 1)

- `skdfx_Crate` is a `CfgVehicles` class: a vanilla ammo box (`Box_NATO_Ammo_F`) with an `init` event handler. Created anywhere with `createVehicle` or `skd_fnc_create`, it adds on every machine with an interface an `addAction` titled exactly `Open crate`, with radius 3 m and the condition that the crate is not open (`!(_target getVariable ["skdfx_open", false])`).
- Firing the action sends a request to the server. The server opens the crate only if, as the server sees it, the caller is alive and within 3 m of the crate:
  - **Opened**: the server sets `skdfx_open` to `true` on the crate (public), adds one `FirstAidKit` to the caller's inventory, and adds 1 to `skdfx_openCount` in `missionNamespace` (public).
  - **Refused** (too far or dead, as the server sees it): nothing above changes, and the server adds 1 to `skdfx_refusedCount` in `missionNamespace` (public).
- Observable on every machine once replicated: `crate getVariable "skdfx_open"`, `{_x == "FirstAidKit"} count items unit`, `skdfx_openCount`, `skdfx_refusedCount`. Both counts start at `0` at every mission start.

### A replicated value (`OBS-06`, `CTL-01`)

| Function | Runs on | Does |
|---|---|---|
| `skdfx_fnc_setFlag` | the server | `[value] call skdfx_fnc_setFlag` sets `skdfx_flag` in `missionNamespace` to `value`, public. Called elsewhere, it does nothing and returns `false`; on the server it returns `true`. |
| `skdfx_fnc_setFlagLater` | the server | `[value, delay] call skdfx_fnc_setFlagLater` sets `skdfx_flag` as `skdfx_fnc_setFlag` does, `delay` seconds of real time later (`diag_tickTime`), and returns at once. |

`skdfx_flag` is `nil` at every mission start. `setFlagLater` gives a wait a target whose arrival time is known: a wait with a ceiling above `delay` sees the value; one below it does not.

### A script error on demand (`ERR-01` to `ERR-04`)

- `skdfx_fnc_throw`: `[] call skdfx_fnc_throw` raises one script error on the machine it runs on, whose text contains `_skdfx_undefined` (an undefined local variable used in an expression), from the file `\skdfx\addons\main\functions\fn_throw.sqf`. It then returns `nil`. Called with `spawn` the error is raised in spawned code; called through `remoteExec`, on the target.

### A reload target inside the mission (`RLD-01`, `RLD-02`, level `mission`)

- `skdfx_fnc_version` is a `CfgFunctions` function from the file `skdfx\addons\main\functions\fn_version.sqf` (game path `\skdfx\addons\main\functions\fn_version.sqf`). It returns a string. As installed, the whole file is:

  ```sqf
  "1"
  ```

- To test a reload, the installed copy of this file may be replaced as [Files a test may change](#files-a-test-may-change-fix-03) says.

## `@skd_fixture_cba`

### A CBA keybind (`ACT-02`, `ACT-03`)

- Registered on every machine with an interface, with `CBA_fnc_addKeybind`: mod name `skd_fixture`, action name `signal`, display name `Fixture signal`, default key F7 (DIK `65`) with no modifier, not a hold key.
- Its down code adds 1 to `skdfx_signalDown` on the player's unit; its up code adds 1 to `skdfx_signalUp` on the player's unit. Both are set public, start at `0` on a unit that has never signalled, and are read with `unit getVariable ["skdfx_signalDown", 0]`.
- Its down code also raises the CBA global event `skdfx_signalled` with `[unit]`, for event recording (`EVT-01`, after the MVP).

### A reload target behind a mission restart (`RLD-02`, level `mission-restart`)

- `skdfx_cba_fnc_version` is defined with CBA's `PREP` from the file `skdfx_cba\addons\main\functions\fnc_version.sqf` (game path `\skdfx_cba\addons\main\functions\fnc_version.sqf`). It is final, as CBA compiles it, so a change reaches it only at a mission restart with CBA's cache off. As installed, the whole file is:

  ```sqf
  "1"
  ```

- It may be replaced in the installed copy as [Files a test may change](#files-a-test-may-change-fix-03) says.

## Failures on demand (`FIX-01`, `FIX-02`)

These give a test the failures of the environment it cannot otherwise cause at a known moment. Each is in `@skd_fixture` and works on every kind of machine unless it says otherwise.

| Function | Does |
|---|---|
| `skdfx_fnc_crash` | `[delay] call skdfx_fnc_crash`, `delay` seconds of real time from 0 to 60. Returns `true` at once. After `delay`, the game process it ran on faults in native code inside `skdfx_x64.dll` (an access violation), so the engine's own crash handling runs and the process ends unasked with a nonzero exit code. Whether the engine leaves a `.mdmp` or `.bidmp` in the machine's profile folder is measured by `SPK-16`. |
| `skdfx_fnc_exit` | `[delay, code] call skdfx_fnc_exit`, `delay` seconds of real time from 0 to 60, `code` a whole number from 1 to 2147483647. Returns `true` at once. After `delay`, the game process it ran on ends with exit code `code`, leaving no dump. |
| `skdfx_fnc_holdClock` | `[seconds] call skdfx_fnc_holdClock`, `seconds` from 1 to 30. Singleplayer `p1` only. Returns `true` at once, then holds `time` still for `seconds` of real time while `diag_frameNo` keeps advancing and the machine keeps answering orders within their ceilings plus the grace of `OBS-03`; then `time` runs again at the real-time rate. A hold under 10 s ends no order as lost. On any other machine, or in a dedicated session, it does nothing and returns `false`. Measured by `SPK-17`. |

A delay lets a check place the failure between its own steps: `["p2", {[0.5] call skdfx_fnc_crash}] call skd_fnc_on` followed by an assertion that fails at once puts the assertion first; a crash with no delay followed by a wait puts the lost machine first (`DM-06`).

A stall and an end from outside need no fixture. `status` gives each machine's process id and start time (`INS-04`). A process suspended by its id stops answering, which the runner sees as a stall (`P-07`); a process ended by its id exits unasked, which the runner sees as a crash with the exit code it was ended with (`P-09`); a dump written of the live process into its profile folder before it is ended is taken as its crash dump.

## Files a test may change (`FIX-03`)

Only in the installed copy, never in `fixtures/`, and each put back as it was after the test:

| File (in the installed copy) | A test may |
|---|---|
| `skdfx\addons\main\functions\fn_version.sqf` | Replace its whole content with another quoted string, such as `"2"`. |
| `skdfx_cba\addons\main\functions\fnc_version.sqf` | The same. |
| `skdfx\addons\main\functions\fn_new.sqf` | Create it, with any SQF, and delete it after. It is in no built PBO, so file patching does not find it and a reload of it needs a rebuild and a game restart (`RLD-02`). |
| Any other installed fixture file, such as `skdfx\addons\main\config.cpp` | Name it to `reload` without changing it. |

No other fixture file is changed by a test.

## What the fixtures do not do

Unless a test calls a function of [Failures on demand](#failures-on-demand-fix-01-fix-02), they change no time, weather or player state on their own, run no per-frame code, create nothing at mission start, and log nothing to the RPT except through the engine's own messages and the deliberate error of `skdfx_fnc_throw`. A reset therefore returns them to their start state, except `skdfx_openCount`, `skdfx_refusedCount` and `skdfx_flag`, which only a mission restart clears.
