---
type: Specification
title: Check API
description: The contract for writing a check - the check file and its header, where the body runs, and every skd_fnc function a check may call, with its arguments, result, ceilings and the outcome it causes when it fails.
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# Check API

A check is SQF. It needs no build step, no `#include`, no macro and no other language (`API-01` to `API-05`). The words used here are defined in [../00-overview.md](../00-overview.md); the outcome rules are `DM-01` to `DM-09` in [../01-domain-model.md](../01-domain-model.md).

## The check file

A check is one file with the extension `.sqf` anywhere under the project's checks folder (default `checks/`). Its **check id** is its path relative to that folder, with `/` as separator and without `.sqf`: `checks/rope/US-012-hook-holds.sqf` has the id `rope/US-012-hook-holds`.

The file starts with a header: one or more lines of the form `// @key value`, before any other line except blank lines. The header ends at the first line that is not a header line.

| Key | Required | Value | Default |
|---|---|---|---|
| `@check` | yes | The check's name: the story or requirement ID it proves, then a short sentence. | - |
| `@tags` | no | Tags separated by spaces, such as story IDs, finding IDs, `smoke`. | none |
| `@setups` | no | Setup names separated by spaces; the check runs only in these (`RUN-07`). | every setup |
| `@timeout` | no | The check's ceiling in seconds, a whole number from 1 to 3600. | `120` |
| `@reset` | no | `in-place` or `mission` (`RST-06`). | `in-place` |

A key given twice, an unknown key, a missing `@check` or a value of the wrong form ends the check `fail` / `check-error` naming the line.

The rest of the file is the **body**. It is compiled with `compile` (the preprocessor does not run: no `#include`, no `#define`) and runs scheduled on the lead machine: `server` on a dedicated session, `p1` in singleplayer. `_this` is empty. The body may use any SQF. A check passes when its body runs to its end without a failed assertion, a wait at its ceiling or an unexpected script error on any machine (`DM-02`).

```sqf
// @check US-012 A hook thrown at a wall holds a player's weight
// @tags US-012 smoke
// @setups sp dedicated-2

private _wall = ["place a wall in front of p1", {
    ["p1", AGLToASL [100, 104, 0], 0] call skd_fnc_place;
    ["Land_Wall_IndCnc_4_F", AGLToASL [100, 110, 0], 180] call skd_fnc_create
}] call skd_fnc_step;

["throw the hook", {
    ["p1", "agh", "throwHook"] call skd_fnc_pressKey;
}] call skd_fnc_step;

["every machine sees the rope", {
    [
        "all",
        { count ropes (_this select 0) },
        ">=", 1,
        "ropes attached to the wall",
        10, [_wall]
    ] call skd_fnc_expectEach;
}] call skd_fnc_step;
```

## Rules for every function

- **Arguments** are one array, passed with `call`, as `[a, b, c] call skd_fnc_name`. Optional arguments may be left off from the end, or given as `nil`. An argument of the wrong type or a value out of range ends the check `fail` / `check-error` naming the function and the argument (`DM-04`).
- **Machines** are named by a string: `"server"`, `"p1"` to `"pN"`, `"hc1"` to `"hcN"` (`DM-10`). Where a function takes several machines, it takes an array of names or one of the group names `"all"`, `"clients"`, `"headless"` (`DM-12`). A name the session does not have ends the check `fail` / `check-error`.
- **Code sent to a machine** is a code value (`{ ... }`). It is sent as text and compiled on the target, so it does **not** see the check's local variables: pass what it needs in `args`, which arrive as `_this` (`CTL-02`). Objects and groups in `args` and in results travel by net id and arrive as the object or group (`DM-13`, `DM-14`). The code runs scheduled (`CTL-03`), so it may `waitUntil`. It may call any SQF, but not `skd_fnc_*` functions: those run only in the body on the lead machine, and calling one elsewhere ends the check `fail` / `check-error`.
- **Ceilings** are in seconds of real time (`OBS-01`). Every function that waits takes a `ceiling` argument; the default is 10. A ceiling is a number above 0; `nil` takes the default, and any other value ends the check `fail` / `check-error`. A ceiling that reaches past the check's own `@timeout` is cut to the time left. The runner also waits the ceiling plus 10 s on the wall clock; a machine that has not answered by then is `lost` and the check is `invalid` / `machine-lost` (`OBS-03`, `DM-07`).
- **Failure ends the check at once.** A function that fails does not return: the body's script is terminated, the outcome and reason are recorded against the innermost step, and the evidence of `RES-03` is kept.

## Context

| Function | Arguments | Returns |
|---|---|---|
| `skd_fnc_machines` | `[]` | Array of the session's machine names, in the order `server`, `p1`..`pN`, `hc1`..`hcN`. In singleplayer `["p1"]`. |
| `skd_fnc_setup` | `[]` | HashMap with `"name"`, `"mode"` (`"singleplayer"` or `"dedicated"`), `"clients"`, `"headless"`, `"world"`. |
| `skd_fnc_player` | `[machine]` | The unit the player of that machine controls, as the lead machine sees it. `objNull` for `server` on a dedicated session and for a headless client. |

## Steps

| Function | Arguments | Returns |
|---|---|---|
| `skd_fnc_step` | `[title, code, args]` - `title` String; `code` Code; `args` Array, default `[]`. | The value of `code`. |

`code` runs with `call` in the body's own script, so it sees the body's local variables, and `_this` is `args`. Steps nest. The step's path (the titles from the outermost step in), its start and end on every clock of `OBS-05`, and its state are recorded (`CR-42`). Every script error, failed wait and failed assertion inside it is recorded against it (`CR-23`).

## Running code on machines

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_on` | `[machine, code, args, ceiling]` - `args` default `[]`; `ceiling` default 10. | The value `code` returned on that machine. | `fail` / `wait-ceiling` when the code is still running at the ceiling; `fail` / `check-error` when it does not compile there. |
| `skd_fnc_onEach` | `[machines, code, args, ceiling]` | HashMap from machine name to the value `code` returned there. | As `skd_fnc_on`, naming each machine that did not finish. |

A script error the code raises does not stop it: the value still comes back, and the error fails the check `fail` / `script-error` when the body ends, unless an `expectError` block matches it (`CTL-05`).

## Waits

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_waitUntil` | `[machine, condition, description, ceiling, args]` - `condition` Code returning a Boolean; `description` String saying what is awaited; `ceiling` default 10; `args` default `[]`. | `true` when the condition held. | `fail` / `wait-ceiling` with the description and the ceiling. |
| `skd_fnc_tryWaitUntil` | Same as `skd_fnc_waitUntil`. | `true` if the condition held within the ceiling, `false` if not. | Never on the ceiling. |

The condition is evaluated on that machine every frame until it returns `true` or the ceiling passes (`OBS-01`). There is no sleep in the API (`OBS-02`). A condition that raises a script error or returns anything but a Boolean ends the wait at that evaluation, with no retry: `fail` / `script-error` when the machine caught a script error during it, otherwise `fail` / `check-error` naming the type it returned. `skd_fnc_tryWaitUntil` ends the check the same way; it returns `false` only at its ceiling.

## Assertions

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_expect` | `[actual, op, expected, description]` | `true` | At once, `fail` / `assertion`, with the actual and expected values. |
| `skd_fnc_expectOn` | `[machine, value, op, expected, description, ceiling, args]` - `value` Code returning the value to test; `ceiling` default 10; `args` default `[]`. | The last value read. | `fail` / `assertion` at the ceiling, with the last value read (`OBS-04`). |
| `skd_fnc_expectEach` | `[machines, value, op, expected, description, ceiling, args]` | HashMap from machine name to the last value read. | `fail` / `assertion` at the ceiling, with every machine's last value and which ones did not meet it (`OBS-06`). |
| `skd_fnc_expectAgree` | `[machines, value, description, tolerance, ceiling, args]` - `tolerance` Number, default 0. | HashMap from machine name to the last value read. | `fail` / `assertion` at the ceiling, with every machine's last value (`OBS-06`). |

`skd_fnc_expectOn` and `skd_fnc_expectEach` read the value on each machine every frame until the comparison holds on every machine at once, or the ceiling passes. `skd_fnc_expectAgree` holds when every machine's value equals the first machine's value of the list, within `tolerance` for numbers and for positions (arrays of 2 or 3 numbers, by distance).

The operators:

| `op` | Holds when |
|---|---|
| `"=="` | `actual isEqualTo expected` (case-sensitive, works for every type). |
| `"!="` | `!(actual isEqualTo expected)`. |
| `"<"`, `"<="`, `">"`, `">="` | Both are numbers and the comparison holds. |
| `"~="` | `expected` is `[target, tolerance]`; `actual` is a number within `tolerance` of `target`, or a position within `tolerance` metres of it. |
| `"in"` | `actual` is an element of the array `expected` (by `isEqualTo`). |
| `"contains"` | `expected` is an element of the array `actual`, or a substring of the string `actual` (case-sensitive). |

Any other `op`, or operands of the wrong type for it, ends the check `fail` / `check-error`.

## Script errors

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_expectError` | `[text, code, args]` - `text` String; `code` Code run with `call` in the body; `args` default `[]`. | The value of `code`. | `fail` / `expected-error-missing` at the check's end when no error containing `text` was caught. |

Any script error on any machine whose text contains `text` (case-insensitive), from the start of `code` until the check ends, is marked expected and does not fail the check (`ERR-04`). Every other script error on any machine during the check fails it with `fail` / `script-error`, naming the machine, the error, its file and line, and the step (`ERR-03`).

## Building a scene

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_create` | `[class, position, direction]` - `class` String (`CfgVehicles`); `position` ASL `[x, y, z]`; `direction` degrees, default 0. | The object. For a class that is a kind of `CAManBase`, a unit in a new group of the side its config gives. | `fail` / `check-error` for an unknown class. |
| `skd_fnc_place` | `[unit, position, direction, ceiling]` - `unit` an object or a machine name (that machine's player); `position` ASL; `direction` degrees; `ceiling` default 10. | `true` once the server sees the unit within 0.1 m and 3 degrees. | `fail` / `wait-ceiling` with the position the server last saw. |
| `skd_fnc_setDate` | `[date, ceiling]` - `date` `[year, month, day, hour, minute]`; `ceiling` default 10. | `true` once every machine reports that date and time to the minute. | `fail` / `wait-ceiling` with each machine's date. |
| `skd_fnc_setWeather` | `[overcast, fog, rain, ceiling]` - each 0 to 1; `ceiling` default 10. | `true` once every machine reports the overcast within 0.01. | `fail` / `wait-ceiling` with each machine's values. |

`skd_fnc_create` runs on the server with vehicle randomisation off, sets the direction before the position, and records the object for the reset (`SCN-01`). `skd_fnc_place` runs where the unit is local, direction first (`SCN-02`). The reset deletes everything a check created, whether through these functions or through its own SQF on any machine (`SCN-04`).

## Acting through the mod

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_useAction` | `[machine, target, title, ceiling]` - `machine` the acting player's machine; `target` the object carrying the action; `title` String; `ceiling` default 10. | `true` once the action was fired. | At the ceiling, `fail` / `assertion` with the reason: no action with that title (listing the titles on the target), its condition false, or the player's distance and the action's radius. `fail` / `check-error` when the machine has no player. |
| `skd_fnc_pressKey` | `[machine, mod, action, mode]` - `mod` and `action` the names the mod registered its CBA keybind with; `mode` `"press"` (down then up), `"down"` or `"up"`, default `"press"`. | `true` once the keybind's code ran. | `fail` / `check-error` when CBA is not loaded on that machine or the keybind is not registered, listing the keybinds that are (`ACT-03`). |

`skd_fnc_useAction` matches the title after removing structured-text tags (`<...>`) from the action's title, exactly and case-sensitively. It waits until the action exists, its condition holds and the player is within its radius, then fires it with `action ["User", target, id]` on the acting machine (`ACT-01`). `skd_fnc_pressKey` calls the code `CBA_fnc_getKeybind` returns with `[findDisplay 46, DIK, shift, ctrl, alt]` for the first bound key (`ACT-02`).

## Ending a check and keeping evidence

| Function | Arguments | Returns |
|---|---|---|
| `skd_fnc_cleanup` | `[code, args]` - `args` default `[]`. | Nothing. Registers `code` to run on the lead machine at the start of the next reset, last registered first (`RST-01`). For state the reset cannot name, such as a mod setting. A cleanup still running at the reset's ceiling (`RST-04`) is terminated and fails the reset, naming the check that registered it. |
| `skd_fnc_evidence` | `[label, value]` - `label` String. | Nothing. Adds the value to the check's evidence in the result file. |
| `skd_fnc_fail` | `[message]` | Does not return. Ends the check `fail` / `explicit`. |
| `skd_fnc_invalid` | `[message]` | Does not return. Ends the check `invalid` / `explicit`, for a cause outside the mod that the check detects itself. |

## Not in this contract yet

ACE interaction actions (`ACT-04`), event recording and waits on events (`EVT-01`), real key input (`ACT-05`), repeats (`OUT-02`), screenshots (`SHT-01`) and failure simulation (`FLT-01`) are added to this file when a milestone takes them up.
