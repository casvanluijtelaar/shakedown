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

A check is one file with the extension `.sqf` anywhere under the project's tests folder (the project file's `tests`, default `tests/`), except under the library folder ([Shared helpers](#shared-helpers-the-library-folder)). Its **check id** is its path relative to that folder, with `/` as separator and without `.sqf`: `tests/rope/US-012-hook-holds.sqf` has the id `rope/US-012-hook-holds`.

The file starts with a header: one or more lines of the form `// @key value`, before any other line except blank lines. The header ends at the first line that is not a header line. A UTF-8 byte order mark before the first line, as some Windows editors write, is skipped.

| Key | Required | Value | Default |
|---|---|---|---|
| `@check` | yes | The check's name: the story or requirement ID it proves, then a short sentence. The ID is capital letters and digits in parts joined by `-`, holding a digit, such as `US-012`, `NFR-PF-02` or `B3`. | - |
| `@tags` | no | Tags separated by spaces, such as story IDs, finding IDs, `smoke`. | none |
| `@setups` | no | Setup names separated by spaces, each one the project file declares; the check runs only in these (`RUN-07`). | every setup |
| `@timeout` | no | The check's ceiling in seconds, a whole number from 1 to 3600. | `120` |
| `@reset` | no | `in-place` or `mission` (`RST-06`). | `in-place` |

A key given twice, an unknown key, a missing `@check`, a value of the wrong form, or an `@setups` that names a setup the project file does not declare ends the check `fail` / `check-error` naming the line; for an unknown setup it also names the setup and the setups the file declares. `list` shows the same message as the check's `error`.

The rest of the file is the **body**. It is compiled with `compile` (the preprocessor does not run: no `#include`, no `#define`) and runs scheduled on the lead machine: `server` on a dedicated session, `p1` in singleplayer. `_this` is empty. The body may use any SQF. On a dedicated session the server has no player, so `player` in the body is `objNull` there; a check that runs in both kinds of setup names a player's unit with `["p1"] call skd_fnc_player`.

Comments, `//` to the end of the line and `/* ... */`, may stand anywhere in the file. The runner turns every comment into spaces before it sends the body, because `compile` takes none, and keeps the line breaks, so a compile message names the line of the file. The game stands still while it compiles the body, and a long string literal compiles in time that grows with the square of its length: about half a second for 100 KB, seconds beyond. Keep data longer than a few kilobytes out of string literals in a check. A check passes when its body runs to its end without a failed assertion, a wait at its ceiling or an unexpected script error on any machine (`DM-02`).

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

## Shared helpers: the library folder

Helper code that several checks use is written once, in the project's **library folder** (`API-06`): `_lib` inside the tests folder by default, `tests/_lib/`, or the folder the project file's `lib` key names ([cli.md](cli.md)). Every file with the extension `.sqf` directly in that folder is a library file; a subfolder of it is not read. `list` and `run` never take a file under the library folder as a check.

Each library file is compiled on every machine of the session, after each mission start (the session's start, a mission restart, a game restart and a client's join) and after each `reload`, and the compiled code is stored in the global variable `skd_lib_<file name>`, the file name without `.sqf`: `tests/_lib/rope.sqf` becomes `skd_lib_rope`, and `call skd_lib_rope` runs the file and returns its value. The body of every check and the code sent to any machine can call it. A file name is letters, digits and `_`. The runner reads the library folder when the session starts and at each `reload`, so a changed library file reaches the game with `shakedown reload`, at level `none`, without a game restart; it is not read again before each check, as a check file is.

A library file follows the rules of the body: no header, no preprocessor, comments turned into spaces with the line breaks kept. A file that holds its helpers as values returns them, such as a HashMap of code values, and a check takes them with `private _rope = call skd_lib_rope;`; a file that defines global variables when it is called defines them on the machine that calls it.

```sqf
// tests/_lib/rope.sqf
createHashMapFromArray [
    ["anchor", { params ["_pos"]; ["Land_Wall_IndCnc_4_F", _pos, 0] call skd_fnc_create }],
    ["length", { params ["_rope"]; ropeLength _rope }]
]
```

```sqf
// @check US-012 A hook holds on the wall the library builds
private _rope = call skd_lib_rope;
private _wall = [AGLToASL [100, 110, 0]] call (_rope get "anchor");
["p1", { count (call skd_lib_rope) }, "==", 2, "p1 has the library too"] call skd_fnc_expectOn;
```

A library file that does not compile, or whose name is not letters, digits and `_`, ends every check of every run `fail` / `check-error` before its body runs, naming the file, relative to the project folder, its line and the engine's message, until it is fixed and `shakedown reload` reads it again. The run goes on to its end.

## Rules for every function

- **Arguments** are one array, passed with `call`, as `[a, b, c] call skd_fnc_name`. Optional arguments may be left off from the end, or given as `nil`. An argument of the wrong type or a value out of range ends the check `fail` / `check-error` naming the function and the argument (`DM-04`).
- **Machines** are named by a string: `"server"`, `"p1"` to `"pN"`, `"hc1"` to `"hcN"` (`DM-10`). Where a function takes several machines, it takes an array of names or one of the group names `"all"`, `"clients"`, `"headless"` (`DM-12`). A name the session does not have ends the check `fail` / `check-error`.
- **Code sent to a machine** is a code value (`{ ... }`). It is sent as text and compiled on the target, so it does **not** see the check's local variables: pass what it needs in `args`, which arrive as `_this` (`CTL-02`). Objects and groups in `args` and in results travel by net id and arrive as the object or group (`DM-13`, `DM-14`). Code that runs once, given to `skd_fnc_on`, `skd_fnc_onEach` or `skd_fnc_cleanup`, has each one looked up once, when it starts on the target; a wait or a polled assertion has them looked up again before every evaluation. An object the target does not know yet, such as one the server has just created, arrives as `objNull`, so a check that reads a just-created object on another machine waits for it there first with `skd_fnc_waitUntil`, which finds it as soon as it reaches that machine. The code runs scheduled (`CTL-03`), so it may `waitUntil`. It may call any SQF, but not `skd_fnc_*` functions: those run only in the body on the lead machine, and calling one elsewhere ends the check `fail` / `check-error`.
- **Ceilings** are in seconds of real time (`OBS-01`). Every function that waits takes a `ceiling` argument; the default is 10. A ceiling is a number above 0; `nil` takes the default, and any other value ends the check `fail` / `check-error`. A ceiling that reaches past the check's own `@timeout` is cut to the time left. The runner also waits the ceiling plus 10 s on the wall clock; a machine that has not answered by then is `lost` and the check is `invalid` / `machine-lost` (`OBS-03`, `DM-07`).
- **Failure ends the check at once.** A function that fails does not return: the body's script is terminated, the outcome and reason are recorded against the innermost step, and the evidence of `RES-03` is kept.

## Context

| Function | Arguments | Returns |
|---|---|---|
| `skd_fnc_machines` | `[]` | Array of the names of the session's machines whose process runs, in the order `server`, `p1`..`pN`, `hc1`..`hcN`: a late client only once it has joined, a client that left not until it joins again. In singleplayer `["p1"]`. |
| `skd_fnc_setup` | `[]` | HashMap with `"name"`, `"mode"` (`"singleplayer"` or `"dedicated"`), `"clients"`, `"headless"`, `"world"`. |
| `skd_fnc_player` | `[machine]` | The unit the player of that machine controls, as the lead machine sees it: on a dedicated session the player whose name is the machine's name (`DM-11`). `objNull` for `server` on a dedicated session, for a headless client, and for a client with no unit. |

## Steps

| Function | Arguments | Returns |
|---|---|---|
| `skd_fnc_step` | `[title, code, args]` - `title` String; `code` Code; `args` Array, default `[]`. | The value of `code`. |

`code` runs with `call` in the body's own script, so it sees the body's local variables, and `_this` is `args`. Steps nest. The step's path (the titles from the outermost step in), its start and end on every clock of `OBS-05`, and its state are recorded (`CR-42`). Every script error, failed wait and failed assertion inside it is recorded against it (`CR-23`).

## Running code on machines

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_on` | `[machine, code, args, ceiling]` - `args` default `[]`; `ceiling` default 10. | The value `code` returned on that machine. | `fail` / `wait-ceiling` when the code is still running at the ceiling; `fail` / `check-error` when it does not compile there. |
| `skd_fnc_onEach` | `[machines, code, args, ceiling]` | HashMap from machine name to the value `code` returned there; a machine whose code returned nothing has no key. | As `skd_fnc_on`, naming each machine that did not finish. |

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
| `skd_fnc_expectAgree` | `[machines, value, description, tolerance, ceiling, args]` - `tolerance` Number, default 0. | HashMap from machine name to the last value read. | `fail` / `assertion` at the ceiling, with every machine's last value; the message says what each machine that differs read where the first machine read its value, and `reason.machine` is the first that differs (`OBS-06`). |

`skd_fnc_expectOn` and `skd_fnc_expectEach` read the value on each machine every frame until the comparison holds on every machine at once, or the ceiling passes. A machine other than the lead sends each new value it reads to the lead through the runner, at most one every 0.05 s (`P-03` step 6), so the lead compares each machine's newest value. `skd_fnc_expectEach` names the first machine that did not meet it in `reason.machine`. `skd_fnc_expectAgree` holds when every machine's value equals the first machine's value of the list, within `tolerance` for numbers and for positions (arrays of 2 or 3 numbers, by distance).

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
| `skd_fnc_expectError` | `[text, code, args]` - `text` a String that is not empty; `code` Code run with `call` in the body; `args` default `[]`. | The value of `code`. | `fail` / `expected-error-missing` at the check's end when no error containing `text` was caught. |

Any script error on any machine whose text contains `text` (case-insensitive), from the start of `code` until the check ends, is marked expected and does not fail the check (`ERR-04`). Every other script error on any machine during the check fails it with `fail` / `script-error` once the body ends, naming the machine, the error, its file and line, and the step, unless an earlier cause already decided the check (`ERR-03`, `DM-06`); `evidence.failed_step` is then the error's step. An `expectError` block that saw no matching error by the end of a check that would otherwise pass ends it `fail` / `expected-error-missing`, quoting `text`; `evidence.failed_step` is the step the block was called in. The text of an error is the engine's message without the `Error ` prefix, such as `Undefined variable in expression: _x`. The engine reports no undefined variable in unscheduled code, only in scheduled code such as the body and the code sent to a machine (research `25-spikes-errors-and-logs.md`).

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
| `skd_fnc_useAction` | `[machine, target, title, ceiling]` - `machine` the acting player's machine; `target` the object carrying the action; `title` String; `ceiling` default 10. | `true` once the action was fired. | At the ceiling, `fail` / `assertion` with the reason: no action with that title (listing the titles on the target), its condition false, or the player's distance and the action's radius; `reason.machine` is the acting machine. `fail` / `check-error` when the machine has no player, and when `target` is `objNull`; on a dedicated session that message says `player` in the body is `objNull` and names `skd_fnc_player`. |
| `skd_fnc_pressKey` | `[machine, mod, action, mode]` - `mod` and `action` the names the mod registered its CBA keybind with; `mode` `"press"` (down then up), `"down"` or `"up"`, default `"press"`. | `true` once the keybind's code ran. | `fail` / `check-error` when CBA is not loaded on that machine or the keybind is not registered, listing the keybinds registered for that mod, in lower case as CBA keeps them (`ACT-03`). |

`skd_fnc_useAction` matches the title after removing structured-text tags (`<...>`) from the action's title, exactly and case-sensitively. It waits on the acting machine until the action exists, its condition holds and the player is within its radius, then fires it there with `action ["User", target, id]` (`ACT-01`); the target travels by net id and is looked up every frame, so an object the server has just created is found once it reaches the acting machine. The condition is evaluated with `_target` the object carrying the action and `_this` the player; a radius of -1 is no limit. The engine itself runs an action fired that way whatever its condition and radius (`SPK-10`). `skd_fnc_pressKey` calls the code `CBA_fnc_getKeybind` returns with `[findDisplay 46, DIK, shift, ctrl, alt]` for the first bound key (`ACT-02`).

## Joining and leaving

On a dedicated session a check can make a game client join late, leave, and join again (`MCH-01`). The setup's `late` key names the late clients, which the session does not start ([cli.md](cli.md)): `clients = 2` and `late = 1` give `p1` and `p2` at the start and the late `p3`.

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_join` | `[machine, ceiling]` - `machine` a late client or a client that left; `ceiling` default the setup's `start_timeout`. | `true` once the machine is `ready` (`P-02` step 7). | `invalid` / `machine-lost` naming the machine when it is not `ready` within the ceiling, with its last 20 RPT lines in the evidence, or at once when its process exits during the join, with its exit code; its process is stopped and the session stays `ready` without it. |
| `skd_fnc_leave` | `[machine, ceiling]` - `machine` a game client in the session; `ceiling` default 20, the server's disconnect timeout of 15 s plus 5. | `true` once the server's `PlayerDisconnected` event has fired for the machine. | `invalid` / `machine-lost` naming the machine when the event has not fired within the ceiling. |

Each runs as a step of its own, titled `join <machine>` or `leave <machine>`, so the result file records when it started and how long it took.

A join starts a new process for the machine, with its own profile folder and the next start number of its role: 1 for a late client's first join, 2 for a client of the start that joins again. `status` lists it from its launch. Once it is `ready` it has a unit of its own (`skd_fnc_player` gives it), it is in `skd_fnc_machines`, every function that takes a machine takes it, and the result file records the versions it loaded. A leave has the server kick the client, which drops it at once rather than after its disconnect timeout, and ends the client's process. The machine then leaves `skd_fnc_machines` and `status`, and its start ends `stopped`, not `lost`: no check is `invalid` for it and the session stays `ready`.

Joining or leaving `server`, a headless client, a machine the setup does not name, a client already in the session or one already out ends the check `fail` / `check-error`, naming the machine and the machines that can join and leave at that moment. In singleplayer either function ends the check `fail` / `check-error` saying that joining and leaving needs a dedicated session.

The reset before the next check first brings the session back to the machines it started with: a late client that joined leaves, and a client that left joins again, within the start ceiling (`RST-01`). A client that left and joined again in the check keeps that start. The unit a client left behind when it left stays, as a body, until that reset deletes it.

## Recording events

A check records the events a mod or the engine announces on chosen machines, reads them with the clocks of each, and waits for one, without handlers of its own (`EVT-01`). An event is a CBA event, raised with `CBA_fnc_localEvent`, `CBA_fnc_globalEvent`, `CBA_fnc_serverEvent` or another of CBA's event functions, or an engine mission event, such as `EntityKilled`, that `addMissionEventHandler` takes. Its name is matched without regard to case.

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_record` | `[machines, event, kind, ceiling]` - `event` String, the event's name; `kind` `"cba"` or `"mission"`, default `"cba"`; `ceiling` default 10. | `true` once every machine records the event. | `fail` / `check-error` when CBA is not loaded on a machine for a `"cba"` event, when the engine has no mission event of that name, or when the event is recorded there already as the other kind. |
| `skd_fnc_events` | `[machine, event]` | Array of the events of that name recorded on the machine, oldest first, each a HashMap with `"args"`, the event's arguments as the machine received them, and `"at"`, its clocks (`OBS-05`) as a HashMap with `"utc"` (String), `"tick"`, `"time"` and `"frame"` (Numbers). | `fail` / `check-error` when the check does not record the event on that machine. |
| `skd_fnc_waitEvent` | `[machine, event, ceiling, after]` - `ceiling` default 10; `after` the number of events of that name on the machine to pass over, default 0. | The arguments of the event number `after + 1` recorded on the machine, once it is recorded. | `fail` / `wait-ceiling` at the ceiling, naming the event and the machine and listing, for that machine and every other machine the check records an event on, how many events of each name it recorded; `evidence.observed` holds the same counts. `fail` / `check-error` when the check does not record the event on that machine. |

Recording starts when `skd_fnc_record` returns on each machine and ends with the check: every handler the check's recording added is removed on every machine when the check ends, whatever its outcome, so the next check records nothing until it records again. Recording an event already recorded on a machine keeps what it recorded. `skd_fnc_waitEvent` returns at once when the event it waits for was recorded before the call, so a check that triggers the event and then waits for it does not miss it; to wait for the next event after those it has seen, it passes their count as `after`. Objects in the arguments travel by net id (`DM-13`, `DM-14`), so an object the lead does not know arrives as `objNull`. The result file lists every event recorded during the check, on every machine, under the check ([result-file.md](result-file.md)).

```sqf
// @check US-072 p2 hears the signal p1 gives
// @setups dedicated-2
["all", "skdfx_signalled"] call skd_fnc_record;
["p1", "skd_fixture", "signal"] call skd_fnc_pressKey;
private _args = ["p2", "skdfx_signalled", 10] call skd_fnc_waitEvent;
[_args select 0, "==", ["p1"] call skd_fnc_player, "the signal names p1's unit"] call skd_fnc_expect;
[count (["server", "skdfx_signalled"] call skd_fnc_events), "==", 1, "the server heard it once"] call skd_fnc_expect;
```

## Ending a check and keeping evidence

| Function | Arguments | Returns |
|---|---|---|
| `skd_fnc_cleanup` | `[code, args, machine]` - `args` default `[]`; `machine` the machine it runs on, default the lead machine. | Nothing. Registers `code` to run on `machine` at the start of the next reset, each machine's cleanups last registered first (`RST-01`). A cleanup for state that lives on one client, such as an action added to its player or `allowDamage` on its unit, names that client: on a dedicated session the lead is the server, where `player` is `objNull`. `code` follows the rules of code sent to a machine: it sees no local variable of the check, only `args` as `_this`, and calls no `skd_fnc_*` function. For state the reset cannot name, such as a mod setting. A cleanup still running at the reset's ceiling (`RST-04`) is terminated and fails the reset, naming the check that registered it. |
| `skd_fnc_evidence` | `[label, value]` - `label` String. | Nothing. Adds the value to the check's evidence in the result file. |
| `skd_fnc_fail` | `[message]` | Does not return. Ends the check `fail` / `explicit`. |
| `skd_fnc_invalid` | `[message]` | Does not return. Ends the check `invalid` / `explicit`, for a cause outside the mod that the check detects itself. |

## Not in this contract yet

Handing over to a manual tester (`MAN-01`), ACE interaction actions (`ACT-04`), real key input (`ACT-05`), repeats (`OUT-02`), screenshots (`SHT-01`) and failure simulation (`FLT-01`) are added to this file by the milestone that builds them.
