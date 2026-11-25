---
type: Documentation
title: The check API
description: Every skd_fnc function a check can call, with its arguments, what it returns, an example and how it fails - the context, steps, code on machines, waits, assertions, expected errors, the scene, actions, keybinds and ACE actions, joining and leaving, recorded events, cleanups, ending a check and handing over - and the rules every function keeps.
status: draft
---

# The check API

A check's body calls these functions, each as `[arguments] call skd_fnc_name`. This page is the reference: every function, its arguments in order, what it returns, an example and how it fails. The pages linked from each section show the functions at work.

## Rules for every function

- **Arguments are one array**, passed with `call`: `[a, b, c] call skd_fnc_name`. Optional arguments may be left off from the end, or given as `nil`. A function with no arguments is called as `call skd_fnc_name` or `[] call skd_fnc_name`.
- **A wrong argument** of the wrong type, or a value out of range, ends the check `fail` / `check-error` with the function, the argument's number and name, what it takes and what it got:

  ```
  skd_fnc_place: argument 2 (position) must be an ASL position [x, y, z] of three numbers, got STRING "here".
  ```

- **Machines** are named by a string: `"server"`, `"p1"` to `"pN"`, `"hc1"` to `"hcN"`. In singleplayer the only machine is `"p1"`, which `"server"` also names. Where a function takes several machines, it takes an array of names or a group: `"all"`, `"clients"` (`p1` to `pN`) or `"headless"` (`hc1` to `hcN`). A name the session does not have ends the check `fail` / `check-error`, naming the session's machines.
- **Code sent to a machine** is a code value, sent as text and compiled there. It does not see the check's local variables; pass what it needs in `args`, which arrive as `_this`. Objects and groups in `args` and in results travel by net id and arrive as the object or group. It may call any SQF but no `skd_fnc_*` function ([Code sent to another machine](writing-checks.md#code-sent-to-another-machine)).
- **Ceilings** are in seconds of real time. Every function that waits takes a `ceiling`; the default is 10. A ceiling is a number above 0, and `nil` takes the default. A ceiling that reaches past the check's own `@timeout` is cut to the time left, and the check then ends `check-timeout`. The runner also waits the ceiling plus 10 s on its own clock; a machine that has not answered by then is lost and the check is `invalid` / `machine-lost`.
- **Failure ends the check at once.** A function that fails does not return: the body is terminated, and the outcome and reason are recorded against the innermost step.

## Context

| Function | Arguments | Returns |
|---|---|---|
| `skd_fnc_machines` | `[]` | The names of the session's machines whose process runs, in the order `server`, `p1`..`pN`, `hc1`..`hcN`: a late client only once it has joined, a client that left not until it joins again. In singleplayer `["p1"]`. |
| `skd_fnc_setup` | `[]` | A HashMap with `"name"`, `"mode"` (`"singleplayer"` or `"dedicated"`), `"clients"`, `"headless"` and `"world"`. |
| `skd_fnc_player` | `[machine]` | The unit the player of that machine controls, as the lead machine sees it. `objNull` for `server` on a dedicated session, for a headless client, and for a client with no unit. |

```sqf
private _machines = call skd_fnc_machines;      // ["p1"] in singleplayer; ["server", "p1", "p2", "hc1"] on a dedicated session
private _setup = call skd_fnc_setup;
private _unit = ["p1"] call skd_fnc_player;
if ((_setup get "mode") == "dedicated") then { /* ... */ };
private _players = _machines select { !isNull ([_x] call skd_fnc_player) };   // the machines with a player
```

## Steps

| Function | Arguments | Returns |
|---|---|---|
| `skd_fnc_step` | `[title, code, args]` - `title` String; `code` Code; `args` Array, default `[]`. | The value of `code`. |

`code` runs with `call` in the body's own script, so it sees the body's local variables, and `_this` is `args`. Steps nest. See [Steps](writing-checks.md#steps).

```sqf
private _n = ["count the boxes", {
    count (allMissionObjects "Land_WoodenBox_F")
}] call skd_fnc_step;
```

## Running code on machines

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_on` | `[machine, code, args, ceiling]` - `args` default `[]`; `ceiling` default 10. | The value `code` returned on that machine. | `fail` / `wait-ceiling` when the code is still running at the ceiling; `fail` / `check-error` when it does not compile there, with the engine's message. |
| `skd_fnc_onEach` | `[machines, code, args, ceiling]` | A HashMap from machine name to the value `code` returned there; a machine whose code returned nothing has no key. | As `skd_fnc_on`, naming each machine that did not finish. |

```sqf
private _dir = ["p1", { getDir (_this select 0) }, [_box]] call skd_fnc_on;
private _each = ["all", { getDir (_this select 0) }, [_box]] call skd_fnc_onEach;   // HashMap: "server" -> 45, "p1" -> 45, ...
private _names = ["all", { profileName }] call skd_fnc_onEach;                       // server -> "server", p1 -> "p1", hc1 -> "skd_hc1", ...
```

The code runs scheduled, so it may `waitUntil`. A script error it raises does not stop it: the value still comes back, and the error fails the check `fail` / `script-error` when the body ends, unless `skd_fnc_expectError` expects it. Objects in `args` are looked up once, when the code starts on the target, so an object the target does not know yet arrives as `objNull`.

## Waits

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_waitUntil` | `[machine, condition, description, ceiling, args]` - `condition` Code returning a Boolean; `description` String saying what is awaited; `ceiling` default 10; `args` default `[]`. | `true` when the condition held. | `fail` / `wait-ceiling` with the description and the ceiling. |
| `skd_fnc_tryWaitUntil` | Same as `skd_fnc_waitUntil`. | `true` if the condition held within the ceiling, `false` if not. | Never on the ceiling. |

```sqf
["p1", { player getVariable ["mymod_state", "IDLE"] == "IDLE" }, "p1 is idle again", 5] call skd_fnc_waitUntil;
private _landed = ["p1", { isTouchingGround player }, "p1 landed", 3] call skd_fnc_tryWaitUntil;
```

The condition is evaluated on that machine every frame. There is no sleep in the API. See [Waits and assertions](waits-and-assertions.md) for the failures and their messages.

## Assertions

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_expect` | `[actual, op, expected, description]` | `true` | At once, `fail` / `assertion`, with the actual and expected values. |
| `skd_fnc_expectOn` | `[machine, value, op, expected, description, ceiling, args]` - `value` Code returning the value to test; `ceiling` default 10; `args` default `[]`. | The last value read. | `fail` / `assertion` at the ceiling, with the last value read. |
| `skd_fnc_expectEach` | `[machines, value, op, expected, description, ceiling, args]` | A HashMap from machine name to the last value read. | `fail` / `assertion` at the ceiling, with every machine's last value and which ones did not meet it. |
| `skd_fnc_expectAgree` | `[machines, value, description, tolerance, ceiling, args]` - `tolerance` Number, default 0. | A HashMap from machine name to the last value read. | `fail` / `assertion` at the ceiling, with every machine's last value and what each machine that differs read where the first machine read its value. |

```sqf
[_n, "==", 0, "no box is left"] call skd_fnc_expect;
["p1", { damage player }, "<", 0.1, "p1 is unhurt"] call skd_fnc_expectOn;
["all", { count itemCargo (_this select 0) }, ">=", 1, "every machine sees the kit in the crate", 10, [_crate]] call skd_fnc_expectEach;
["all", { getPosASL (_this select 0) }, "every machine agrees where p1 is", 0.1, 10, [["p1"] call skd_fnc_player]] call skd_fnc_expectAgree;
```

The operators are `==`, `!=`, `<`, `<=`, `>`, `>=`, `~=` (expected `[target, tolerance]`), `in` and `contains`. See [Waits and assertions](waits-and-assertions.md) for what each holds on and the messages.

## Script errors

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_expectError` | `[text, code, args]` - `text` a String that is not empty; `code` Code run with `call` in the body; `args` default `[]`. | The value of `code`. | `fail` / `expected-error-missing` at the check's end when no error containing `text` was caught. |

```sqf
["_mymod_undefined", {
    ["p2", { [] call mymod_fnc_rejectBadCall; true }] call skd_fnc_on;
}] call skd_fnc_expectError;
```

Every script error on any machine whose text contains `text`, case-insensitively, from the start of `code` until the check ends, is expected and fails nothing. See [Script errors, logs and evidence](script-errors-and-logs.md).

## Building a scene

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_create` | `[class, position, direction]` - `class` String (`CfgVehicles`); `position` ASL `[x, y, z]`; `direction` degrees, default 0. | The object. For a kind of `CAManBase`, a unit in a new group of the side its config gives. | `fail` / `check-error` for an unknown class. |
| `skd_fnc_place` | `[unit, position, direction, ceiling]` - `unit` an object or a machine name (that machine's player); `position` ASL; `direction` degrees; `ceiling` default 10. | `true` once the server sees the unit within 0.1 m and 3 degrees. | `fail` / `wait-ceiling` with the position the server last saw. |
| `skd_fnc_setDate` | `[date, ceiling]` - `date` `[year, month, day, hour, minute]`; `ceiling` default 10. | `true` once every machine reports that date and time to the minute. | `fail` / `wait-ceiling` with each machine's date. |
| `skd_fnc_setWeather` | `[overcast, fog, rain, ceiling]` - each 0 to 1; `ceiling` default 10. | `true` once every machine reports the overcast within 0.01. | `fail` / `wait-ceiling` with each machine's values. |

```sqf
private _wall = ["Land_Wall_IndCnc_4_F", AGLToASL [100, 110, 0], 180] call skd_fnc_create;
private _enemy = ["O_Soldier_F", AGLToASL [100, 140, 0], 180] call skd_fnc_create;
["p1", AGLToASL [100, 104, 0], 0] call skd_fnc_place;
[[2035, 6, 24, 22, 0]] call skd_fnc_setDate;
[0.9, 0.2, 0.6] call skd_fnc_setWeather;
```

See [Scenes, resets and cleanups](scenes-resets-and-cleanups.md).

## Acting through the mod

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_useAction` | `[machine, target, title, ceiling]` - `machine` the acting player's machine; `target` the object carrying the action; `title` String; `ceiling` default 10. | `true` once the action was fired. | At the ceiling, `fail` / `assertion` with the reason: no action with that title (listing the titles on the target), its condition false, or the player's distance and the action's radius. `fail` / `check-error` when the machine has no player, and when `target` is `objNull`. |
| `skd_fnc_pressKey` | `[machine, mod, action, mode]` - `mod` and `action` the names the mod registered its CBA keybind with; `mode` `"press"` (down then up), `"down"` or `"up"`, default `"press"`. | `true` once the keybind's code ran. | `fail` / `check-error` when CBA is not loaded on that machine or the keybind is not registered, listing the keybinds registered for that mod. |
| `skd_fnc_useAceAction` | `[machine, target, path, ceiling]` - `target` the object whose menu holds the action, or a machine name for that machine's player; `path` an Array of one or more Strings, the action names from the menu's root; `ceiling` default 10. | `true` once ACE's key-up code ran the action. | At the ceiling, `fail` / `assertion`: a name not found, a condition false, or the path ending at a folder of actions. At once, `fail` / `check-error` when ACE is not loaded on the acting machine, when the loaded ACE lacks the internals it uses, when the machine has no player, and when `target` is `objNull`. |

```sqf
["p1", _crate, "Open crate"] call skd_fnc_useAction;
["p1", "mymod", "stowKit"] call skd_fnc_pressKey;
["p1", "mymod", "signal", "down"] call skd_fnc_pressKey;
["p1", _crate, ["ACE_MainActions", "mymod_open"]] call skd_fnc_useAceAction;
["p1", "p1", ["ACE_SelfActions", "mymod_signal"]] call skd_fnc_useAceAction;   // p1's own menu
```

See [Acting through the mod](acting-through-the-mod.md) and [ACE interaction actions](ace-interaction.md).

## Joining and leaving

On a dedicated session a check makes a game client join late, leave and join again.

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_join` | `[machine, ceiling]` - `machine` a late client or a client that left; `ceiling` default the setup's `start_timeout`. | `true` once the machine is ready. | `invalid` / `machine-lost` naming the machine when it is not ready within the ceiling, with its last 20 RPT lines in the evidence, or at once when its process exits during the join, with its exit code; its process is stopped and the session stays ready without it. |
| `skd_fnc_leave` | `[machine, ceiling]` - `machine` a game client in the session; `ceiling` default 20, the server's disconnect timeout of 15 s plus 5. | `true` once the server's `PlayerDisconnected` event has fired for the machine. | `invalid` / `machine-lost` naming the machine when the event has not fired within the ceiling. |

```sqf
["p3"] call skd_fnc_join;      // the late client p3 starts and is ready
["p2"] call skd_fnc_leave;     // p2 goes
["p2", 120] call skd_fnc_join; // p2 comes back with start number 2, ceiling 120 s
```

Each runs as a step of its own, titled `join <machine>` or `leave <machine>`. Joining or leaving `server`, a headless client, a machine the setup does not name, a client already in or one already out ends the check `fail` / `check-error`, naming the machines that can join and leave at that moment; in singleplayer either function ends it `fail` / `check-error` saying that joining and leaving needs a dedicated session. See [Joining late, leaving and joining again](dedicated-sessions.md#joining-late-leaving-and-joining-again).

## Recording events

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_record` | `[machines, event, kind, ceiling]` - `event` String, the event's name; `kind` `"cba"` or `"mission"`, default `"cba"`; `ceiling` default 10. | `true` once every machine records the event. | `fail` / `check-error` when CBA is not loaded on a machine for a `"cba"` event, when the engine has no mission event of that name, or when the event is recorded there already as the other kind. |
| `skd_fnc_events` | `[machine, event]` | An Array of the events of that name recorded on the machine, oldest first, each a HashMap with `"args"`, the event's arguments as the machine received them, and `"at"`, a HashMap of its clocks `"utc"` (String), `"tick"`, `"time"` and `"frame"` (Numbers). | `fail` / `check-error` when the check does not record the event on that machine. |
| `skd_fnc_waitEvent` | `[machine, event, ceiling, after]` - `ceiling` default 10; `after` the number of events of that name on the machine to pass over, default 0. | The arguments of event number `after + 1` recorded on the machine, once it is recorded. | `fail` / `wait-ceiling` at the ceiling, listing the events each recording machine did record; `fail` / `check-error` when the check does not record the event on that machine. |

```sqf
["all", "mymod_signalled"] call skd_fnc_record;
["p1", "mymod", "signal"] call skd_fnc_pressKey;
private _args = ["p2", "mymod_signalled", 10] call skd_fnc_waitEvent;
[count (["server", "mymod_signalled"] call skd_fnc_events), "==", 1, "the server heard it once"] call skd_fnc_expect;
```

See [Recording the mod's events](recording-events.md).

## Cleanups, evidence and ending a check

| Function | Arguments | Returns |
|---|---|---|
| `skd_fnc_cleanup` | `[code, args, machine]` - `args` default `[]`; `machine` the machine it runs on, default the lead machine. | Nothing. Registers `code` to run on `machine` at the start of the next reset, each machine's cleanups last registered first. `code` follows the rules of code sent to a machine. A cleanup still running at the reset's 15 s ceiling is terminated and fails the reset, naming the check that registered it. |
| `skd_fnc_evidence` | `[label, value]` - `label` String. | Nothing. Adds the value to the check's evidence in the result file, for every outcome. |
| `skd_fnc_fail` | `[message]` | Does not return. Ends the check `fail` / `explicit`. |
| `skd_fnc_invalid` | `[message]` | Does not return. Ends the check `invalid` / `explicit`, for a cause outside the mod that the check detects itself. |

```sqf
["p1", { player allowDamage false }] call skd_fnc_on;
[{ player allowDamage true }, [], "p1"] call skd_fnc_cleanup;     // undone on p1, where it lives

["boxes", _n] call skd_fnc_evidence;
if (isNil "mymod_fnc_stowKit") then { ["mymod is not loaded"] call skd_fnc_invalid };
if (_n > 3) then { ["too many boxes"] call skd_fnc_fail };
```

See [Scenes, resets and cleanups](scenes-resets-and-cleanups.md#cleanups).

## Handing over to a person

| Function | Arguments | Returns | Fails |
|---|---|---|---|
| `skd_fnc_handOver` | `[machine, items]` - `machine` a game client, `"p1"` to `"pN"`; `items` an Array of 1 to 10 Strings that are not empty, each one thing to look for. | Does not return: the body ends there. | `fail` / `check-error` for `server` or a headless client on a dedicated session, naming the game clients it can hand over to, and for `items` of the wrong form; `fail` / `wait-ceiling` when the list is not on the client's screen within 10 s. |

```sqf
["p1", [
    "An ammo crate stands 2 m in front of you.",
    "Looking at it, the action menu offers Open crate."
]] call skd_fnc_handOver;
```

See [Handing over to a person](handing-over.md).

## Not in the API

- **No sleep.** Every wait has a condition and a ceiling.
- **No real key input.** Arma 3 reads the keyboard through DirectInput, so a key posted to a game window reaches neither a key handler nor a CBA keybind; a check presses a keybind with `skd_fnc_pressKey` instead.
- **No screenshots and no failure simulation** functions. To make a failure on purpose, see [Making a failure on purpose](outcomes-and-broken-sessions.md#making-a-failure-on-purpose).
