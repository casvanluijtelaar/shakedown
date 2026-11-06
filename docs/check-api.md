---
type: Documentation
title: The check API by example
description: Every skd_fnc function a check can call, with an example of each - the context, code on a machine, waits, assertions, the scene, actions and keybinds, cleanups and ending a check - and what each does when it fails.
status: draft
---

# The check API by example

A check's body calls these functions, each as `[arguments] call skd_fnc_name`. Their contract is [spec/contract/check-api.md](../spec/contract/check-api.md); this page shows each one at work in a singleplayer session, where the only machine is `p1` and `"server"` names it too. On a dedicated session the same functions reach the server, the clients and the headless clients by name; [Dedicated sessions](dedicated-sessions.md) shows how. [Writing and running a check](writing-and-running-checks.md) covers the file, the header and the commands.

Rules for all of them:

- An argument of the wrong type, or a value out of range, ends the check `fail` / `check-error` with the function, the argument's number and name, what it takes and what it got: `skd_fnc_place: argument 2 (position) must be an ASL position [x, y, z] of three numbers, got STRING "here".`
- A machine is `"p1"` (or `"server"`) in singleplayer, and `"server"`, `"p1"` to `"pN"` or `"hc1"` to `"hcN"` on a dedicated session; where a function takes several, `"all"`, `"clients"`, `"headless"` or an array of names. Any other name ends the check `fail` / `check-error` naming the session's machines.
- Code given to a machine is compiled from its text there: it does not see the check's local variables, so pass what it needs in `args`, which arrive as `_this`. Objects and groups in `args` and in results arrive as the object or group; code arrives as a tagged value, not as code. `skd_fnc_*` functions cannot be called inside it.
- Every function that waits takes a `ceiling` in seconds of real time, default 10. A ceiling that reaches past the check's `@timeout` is cut to the time left, and the check then ends `check-timeout`.
- A function that fails does not return. The check ends there, and the reason is recorded against the innermost step.

## Context

```sqf
private _machines = call skd_fnc_machines;      // ["p1"]
private _setup = call skd_fnc_setup;            // HashMap: name, mode, clients, headless, world
private _unit = ["p1"] call skd_fnc_player;     // the unit p1's player controls; objNull for "server" and "hc1" on a dedicated session
[_setup get "mode", "==", "singleplayer", "the session is singleplayer"] call skd_fnc_expect;
```

## Steps and evidence

```sqf
private _n = ["count the boxes", {
    count (allMissionObjects "Land_WoodenBox_F")
}] call skd_fnc_step;
["boxes", _n] call skd_fnc_evidence;
```

`skd_fnc_step [title, code, args]` runs `code` in the body's own script, so it sees the body's local variables; `args` arrive as `_this`. `skd_fnc_evidence [label, value]` adds a value to the result for every outcome.

## Code on a machine

```sqf
private _dir = ["p1", { getDir (_this select 0) }, [_box]] call skd_fnc_on;
private _each = ["all", { getDir (_this select 0) }, [_box]] call skd_fnc_onEach;   // HashMap: "p1" -> 45
```

`skd_fnc_on [machine, code, args, ceiling]` returns the value the code returned. `skd_fnc_onEach [machines, code, args, ceiling]` returns a HashMap from machine name to value; a machine whose code returned nothing has no key. Code still running at its ceiling ends the check `fail` / `wait-ceiling`; code that does not compile ends it `fail` / `check-error` with the engine's message.

## Waits

```sqf
["p1", { player getVariable ["agh_climbState", "OFF"] == "OFF" }, "p1 is off the rope", 5] call skd_fnc_waitUntil;
private _landed = ["p1", { isTouchingGround player }, "p1 landed", 3] call skd_fnc_tryWaitUntil;
```

`skd_fnc_waitUntil [machine, condition, description, ceiling, args]` evaluates the condition every frame and returns `true` once it holds; at the ceiling the check ends `fail` / `wait-ceiling` with the description. `skd_fnc_tryWaitUntil` takes the same arguments and returns `false` at its ceiling instead. A condition that raises a script error ends the check at that evaluation, `fail` / `script-error` naming the machine and the error; one that returns anything but `true` or `false` ends it `fail` / `check-error` naming what it returned, such as `a String`. `skd_fnc_tryWaitUntil` ends the check the same way; it returns `false` only at its ceiling. There is no sleep: every wait has a condition.

## Assertions

```sqf
[_n, "==", 0, "no box is left"] call skd_fnc_expect;
["p1", { damage player }, "<", 0.1, "p1 is unhurt"] call skd_fnc_expectOn;
["all", { count ropes (_this select 0) }, ">=", 1, "every machine sees the rope", 10, [_anchor]] call skd_fnc_expectEach;
["all", { getPosASL player }, "every machine agrees where p1 is", 0.1] call skd_fnc_expectAgree;
```

`skd_fnc_expect [actual, op, expected, description]` asserts at once. `skd_fnc_expectOn [machine, value, op, expected, description, ceiling, args]` reads the value every frame until the comparison holds and returns the last value; at the ceiling the check ends `fail` / `assertion` with the last value read, which the result also keeps under `evidence.observed`. `skd_fnc_expectEach` does the same on several machines at once and returns a HashMap of their values. `skd_fnc_expectAgree [machines, value, description, tolerance, ceiling, args]` holds when every machine's value equals the first one's, within `tolerance` for numbers and positions.

The operators: `==` and `!=` (any type, case-sensitive), `<`, `<=`, `>`, `>=` (numbers), `~=` (expected `[target, tolerance]`, a number or a position), `in` (actual is an element of the array expected), `contains` (expected is an element of the array actual, or a substring of the string actual). Operands of the wrong type end the check `fail` / `check-error`.

## Script errors

A script error on any machine during a check fails it `fail` / `script-error` once the body ends, naming the machine, the error with its file and line, and the step. `skd_fnc_expectError [text, code, args]` runs `code` with `call` and returns its value; every error on any machine whose text contains `text`, case-insensitively, from then to the end of the check is expected and fails nothing:

```sqf
["_skdfx_undefined", { ["p2", { [] call skdfx_fnc_throw; true }] call skd_fnc_on }] call skd_fnc_expectError;
```

When no such error came by the end of a check that otherwise passes, it ends `fail` / `expected-error-missing`, quoting the text. `text` must be a String that is not empty. [Script errors, logs and evidence](script-errors-and-logs.md) says which errors count and how each is traced to its step.

## Building a scene

```sqf
private _wall = ["Land_Wall_IndCnc_4_F", AGLToASL [100, 110, 0], 180] call skd_fnc_create;
private _enemy = ["O_Soldier_F", AGLToASL [100, 140, 0], 180] call skd_fnc_create;
["p1", AGLToASL [100, 104, 0], 0] call skd_fnc_place;
[[2035, 6, 24, 22, 0]] call skd_fnc_setDate;
[0.9, 0.2, 0.6] call skd_fnc_setWeather;
```

- `skd_fnc_create [class, position, direction]` creates a `CfgVehicles` class at an ASL position, direction first, and returns it; a soldier class is a unit in a new group of its config's side. An unknown class ends the check `fail` / `check-error`: `skd_fnc_create: the class "B_Soldier_Typo" is not in CfgVehicles (or is not creatable). ...` It returns once the server has made the object. On a dedicated session another machine sees it a moment later: a check that reads it there at once can find no object, or the object at the spot the engine made it, before it is in place. Such a check waits for it first, as [dedicated sessions](dedicated-sessions.md) shows.
- `skd_fnc_place [unit, position, direction, ceiling]` takes an object or a machine name (that machine's player) and returns once the unit stands within 0.1 m and 3 degrees of it.
- `skd_fnc_setDate [[year, month, day, hour, minute], ceiling]` and `skd_fnc_setWeather [overcast, fog, rain, ceiling]` (each 0 to 1) return once the machines report the new date, or the overcast within 0.01.

Whatever a check creates, through these functions or its own SQF, the next reset deletes.

## Acting through the mod

```sqf
["p1", player, "Get on rope"] call skd_fnc_useAction;
["p1", "Grappling hooks", "agh_letGo"] call skd_fnc_pressKey;
["p1", "skd_fixture", "signal", "down"] call skd_fnc_pressKey;
```

`skd_fnc_useAction [machine, target, title, ceiling]` finds the action on `target` by its title, with structured-text tags such as `<t color='#ff0000'>` removed, exactly and case-sensitively. It waits until the action exists, its condition holds and the player is within its radius, then fires it as the player's own choice from the scroll menu. The engine does not check the condition or the radius of an action fired this way, so the framework does. At the ceiling the check ends `fail` / `assertion` with the reason: no action of that title (with the titles there are), its condition false, or the player's distance and the radius.

`skd_fnc_pressKey [machine, mod, action, mode]` presses a keybind a mod registered with CBA, by the mod and action names it registered: it calls the keybind's own key-down code, key-up code or both (`mode` `"press"`, the default, `"down"` or `"up"`) as the game's key handler would, with its first bound key. A keybind that is not registered ends the check `fail` / `check-error` with the keybinds registered for that mod, in lower case; without CBA in the setup, with `skd_fnc_pressKey: CBA is not loaded on p1, ...`. A check that presses no key needs no CBA.

## Cleanups and ending a check

```sqf
private _id = player addAction ["Wave", {}];
[{ player removeAction (_this select 0) }, [_id]] call skd_fnc_cleanup;

if (isNil "agh_fnc_placeHook") then { ["arma-grappling-hook is not loaded"] call skd_fnc_invalid };
if (_n > 3) then { ["too many boxes"] call skd_fnc_fail };
```

`skd_fnc_cleanup [code, args]` registers code that runs at the start of the next reset, last registered first, for state the reset cannot name: an action added to the player, a variable, a mod's setting. `skd_fnc_fail [message]` ends the check `fail` / `explicit`; `skd_fnc_invalid [message]` ends it `invalid` / `explicit`, for a cause outside the mod that the check itself detects.

## The reset between checks

Before every check the session is reset in place, in under a second: the previous check's cleanups run, every unit, vehicle and object created since the mission started is deleted (whatever created it, on whichever machine), every player is brought out of any vehicle, alive, with the loadout, position, direction, damage and animation it had at the mission's start, and the date and weather are put back on every machine. In singleplayer a player who dies during a check is handed a new unit where it fell at once, so the mission goes on; on a dedicated session it respawns at once. The reset brings that unit back to the start and deletes the body. `shakedown reset` runs the same reset by hand. What the reset does not touch: variables set on units or in `missionNamespace`, actions added to the player, a mod's own state. A check that changes those registers a cleanup.

A reset that cannot bring the session back within 15 s ends the next check `invalid` / `reset-failed` with what did not match, and leaves the session broken: every check after it is `invalid` / `session-broken` until the session is stopped and started again.
