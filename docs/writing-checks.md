---
type: Documentation
title: Writing checks
description: How a check is written in SQF - its file and id, the header with @check, @tags, @setups, @timeout and @reset, the body and the lead machine it runs on, steps, what _this holds, what code sent to another machine sees, shared helpers in the library folder, and the message each mistake gives.
status: draft
---

# Writing checks

A check is one SQF file that sets up a situation in the running game, makes the mod act, and asserts what should follow. It needs no build step, no `#include`, no macro and no other language. This page covers the file; [The check API](check-api.md) lists every function a check can call.

## A check

```sqf
// @check US-012 A kit p1 stows in a crate shows on every machine
// @tags US-012 smoke crate
// @setups sp dedicated
// @timeout 60

private _crate = ["place a crate in front of p1", {
    ["p1", AGLToASL [100, 104, 0], 0] call skd_fnc_place;
    ["mymod_Crate", AGLToASL [100, 106, 0], 180] call skd_fnc_create
}] call skd_fnc_step;

["stow a kit with the mod's key", {
    ["p1", "mymod", "stowKit"] call skd_fnc_pressKey;
}] call skd_fnc_step;

["every machine sees the kit in the crate", {
    ["all", { count itemCargo (_this select 0) }, ">=", 1, "items in the crate", 10, [_crate]] call skd_fnc_expectEach;
}] call skd_fnc_step;
```

## The file and its id

A check is a file with the extension `.sqf` anywhere under the project's tests folder (`tests` unless `shakedown.toml` sets `tests`), except under the library folder. Its **check id** is its path relative to the tests folder, with `/` as separator and without `.sqf`:

| File | Check id |
|---|---|
| `tests\smoke.sqf` | `smoke` |
| `tests\crate\US-012-kit-in-crate.sqf` | `crate/US-012-kit-in-crate` |

The command line also takes an id with `.sqf` or with `\`, and reads it as the same check. The runner reads every check from disk at the start of each execution, so an edited check needs no reload and no restart: save it and run it again.

## The header

The file starts with a header: one or more lines of the form `// @key value`, before any other line except blank lines. The header ends at the first line that is not a header line. A file saved with a UTF-8 byte order mark, as some Windows editors write, reads the same as one without.

| Key | Required | Value | Default |
|---|---|---|---|
| `@check` | yes | The check's name: an ID for what it proves, then a short sentence. The ID is capital letters and digits in parts joined by `-`, holding a digit, such as `US-012`, `NFR-PF-02`, `BUG-7`, `SMOKE-01` or `B3`. | - |
| `@tags` | no | Tags separated by spaces, such as story IDs, issue numbers or `smoke`. `run --tag` and `list --tag` select by them. | none |
| `@setups` | no | Setup names separated by spaces, each one the project file declares. The check runs only in these; in any other setup it is listed as not run, reason `setup`. | every setup |
| `@timeout` | no | The check's ceiling in seconds, a whole number from 1 to 3600. A body still running at it ends `fail` / `check-timeout`. | `120` |
| `@reset` | no | `in-place`: the session is reset in place before the check, in under a second. `mission`: the check starts in a restarted mission, for state the reset cannot name ([A check that needs a fresh mission](scenes-resets-and-cleanups.md#a-check-that-needs-a-fresh-mission)). | `in-place` |

`@setups` also decides which setups `run --each-setup` starts ([Every setup, repeats and slow steps](every-setup-and-repeats.md)). Give a check that joins clients a longer `@timeout`: a join takes about 30 s.

## The body

The rest of the file is the **body**. It is compiled with `compile` and runs scheduled, so it may `waitUntil` and `sleep`, though a check waits with a condition instead ([Waits and assertions](waits-and-assertions.md)).

- **It runs on the lead machine**: `p1` in singleplayer, `server` on a dedicated session.
- **`_this` is empty** in the body.
- **`player` is `objNull` on a dedicated server**, which has no player. A check that runs in both kinds of setup names a player's unit with `["p1"] call skd_fnc_player`, and runs code as a player with `skd_fnc_on`:

  ```sqf
  private _p1 = ["p1"] call skd_fnc_player;          // p1's unit, wherever the body runs
  private _pos = ["p1", { getPosASL player }] call skd_fnc_on;   // read on p1 itself
  ```

- **No preprocessor.** `#include`, `#define` and macros such as `FUNC(name)` do not work: write the function's full name, such as `mymod_fnc_stowKit`.
- **Comments** of both kinds, `//` to the end of the line and `/* ... */`, may stand anywhere in the file. The runner turns them into spaces before it sends the body, keeping the line breaks, so a compile message names the line of the file.
- **Keep long data out of string literals.** The game stands still while it compiles the body, and a long string literal compiles in time that grows with the square of its length: about half a second for 100 KB, seconds beyond. Keep data longer than a few kilobytes out of a check.

A check **passes** when its body runs to its end without a failed assertion, a wait at its ceiling or an unexpected script error on any machine. A function that fails does not return: the body stops there and the check ends ([Outcomes and a broken session](outcomes-and-broken-sessions.md)).

## Steps

`[title, code, args] call skd_fnc_step` runs `code` as a named step and returns its value. Steps make a failure say where it happened, and the result file records each step's path, the machine it ran on, its state, its start and end on every clock, and its duration.

```sqf
private _crate = ["build the scene", {
    ["Land_WoodenBox_F", AGLToASL [100, 105, 0], 0] call skd_fnc_create
}] call skd_fnc_step;

["the crate is unharmed", {
    ["check its damage", {
        [damage (_this select 0), "==", 0, "the crate's damage"] call skd_fnc_expect;
    }, _this] call skd_fnc_step;
}, [_crate]] call skd_fnc_step;
```

- The code runs with `call` in the body's own script, so it **sees the body's local variables**: `_crate` above could be used inside the step directly.
- `_this` inside the step is `args`, default `[]`.
- Steps nest. A nested step's path is the titles from the outermost in, as `["the crate is unharmed", "check its damage"]`.
- A step longer than the project's `slow_step` is flagged slow ([Slow steps](every-setup-and-repeats.md#slow-steps-and-slow-checks)).
- Every failure, failed wait and script error is recorded against the innermost step running.

## Code sent to another machine

Functions such as `skd_fnc_on`, the waits and the polled assertions send a code value to a machine. That code is sent as **text** and compiled there, so:

- **It does not see the check's local variables.** Pass what it needs in `args`, which arrive as `_this`.
- **Objects and groups travel by net id** and arrive as the object or group. An object the target does not know yet, such as one the server has just created, arrives as `objNull`; a wait or a polled assertion looks it up again every frame, while `skd_fnc_on` looks it up once ([Code, waits and agreement across machines](dedicated-sessions.md#code-waits-and-agreement-across-machines)).
- **Code values in `args` arrive as a tagged value**, not as code. Send the code itself, or call a function the mod or the library defines.
- **It runs scheduled**, so it may `waitUntil` and `sleep`.
- **It cannot call `skd_fnc_*` functions.** They run only in the body on the lead machine; calling one elsewhere ends the check `fail` / `check-error`.
- **A script error in it does not stop it.** Its value still comes back, and the error fails the check once the body ends, unless `skd_fnc_expectError` expects it ([Script errors, logs and evidence](script-errors-and-logs.md)).
- **Code that ends in an assignment**, such as `my_count = my_count + 1`, returns nothing, which the check API takes as `nil`.

```sqf
private _crate = ["Land_WoodenBox_F", AGLToASL [100, 105, 0], 0] call skd_fnc_create;
private _limit = 3;

// Wrong: _crate and _limit are not defined on p1.
// ["p1", { player distance _crate < _limit }] call skd_fnc_on;

// Right: pass them as args.
["p1", { params ["_crate", "_limit"]; player distance _crate < _limit }, [_crate, _limit]] call skd_fnc_on;
```

## Shared helpers: the library folder

Helper code that several checks use is written once, in the project's **library folder**: `tests\_lib\` by default, or the folder the project file's `lib` key names. Every `.sqf` file directly in that folder is a library file; a subfolder of it is not read, and `list` and `run` never take a file under it as a check.

Each library file is compiled on every machine of the session after each mission start (the session's start, a mission restart, a game restart and a client's join) and after each `reload`, into the global variable `skd_lib_<file name>`: `tests\_lib\kit.sqf` becomes `skd_lib_kit`, and `call skd_lib_kit` runs the file and returns its value. The body of every check and the code sent to any machine can call it. A file name is letters, digits and `_`.

A library file follows the rules of the body: no header, no preprocessor, comments turned into spaces. A file that holds its helpers as values returns them, such as a HashMap of code values:

```sqf
// tests\_lib\kit.sqf
createHashMapFromArray [
    ["crate", { params ["_pos"]; ["mymod_Crate", _pos, 0] call skd_fnc_create }],
    ["items", { params ["_crate"]; count itemCargo _crate }],
    ["settle", { params ["_unit"]; waitUntil { vectorMagnitude velocity _unit < 0.1 }; true }]
]
```

```sqf
// @check US-012 A kit stows in the crate the library builds
private _kit = call skd_lib_kit;
private _crate = [AGLToASL [100, 106, 0]] call (_kit get "crate");      // in the body, on the lead
["p1", { [player] call ((call skd_lib_kit) get "settle") }] call skd_fnc_on;   // on p1 too
```

A helper that calls `skd_fnc_*`, such as `crate` above, works only in the body. A file that defines global variables when it is called defines them on the machine that calls it: `["p1", { call skd_lib_kit }] call skd_fnc_on` defines them on `p1`.

The runner reads the library folder when the session starts and at each `reload`, not before each check as it reads a check file. A changed library file reaches the game with `shakedown reload`, which lists it at level `none` and restarts nothing ([Reloading changed mod files](reloading.md)).

A library file that does not compile, or whose name is not letters, digits and `_`, ends every check of every run `fail` / `check-error` before its body runs, until it is fixed and `shakedown reload` reads it again. The run goes on to its end:

```
The library file tests/_lib/kit.sqf does not compile: Invalid number in expression, at line 4. Fix it and run `shakedown reload`; until then every check fails.
```

## When the check itself is wrong

A broken check ends `fail` with reason `check-error` and a message that says what to fix. It is never `invalid`, and the next check of the run still runs. A header mistake is found without the game: `shakedown list` shows it as `<id>  (header error: <message>)`, and the check's `error` in `list --json`.

| Mistake | Message |
|---|---|
| No `@check` line | `The check header is wrong, the header has no @check line. Start the file with `// @check <story or requirement ID> <what it proves>`. ...` |
| `// @check A kit stows` | `... line 1: @check must start with the story or requirement ID the check proves, such as US-012 or NFR-PF-02, then a short sentence; it starts with `A`.` |
| `// @check` with nothing after it | `... line 1: @check has no name. Give the story or requirement ID it proves, then a short sentence.` |
| `// @timeout fast` | `... line 2: @timeout must be a whole number of seconds from 1 to 3600, got `fast`.` |
| `// @reset fresh` | `... line 2: @reset must be `in-place` or `mission`, got `fresh`.` |
| `// @owner me` | `... line 2: unknown header key @owner; a header has only @check, @tags, @setups, @timeout and @reset.` |
| A key given twice | `... line 3: @tags is given twice, first on line 2. Keep one.` |
| `// @setups` with nothing after it | `... line 2: @setups names no setup. Name one or more, or leave the line out.` |
| `// @setups nosuch` | `... line 2: @setups names the setup `nosuch`, which shakedown.toml does not declare; it declares: sp, dedicated. Fix the name.` |
| A syntax error in the body | `The check body does not compile: Invalid number in expression, at line 4 of the check file. ...` |
| `skd_fnc_*` inside code sent to a machine | `skd_fnc_expect was called inside code sent to a machine; skd_fnc_* functions run only in the check body. ...` |
| A wrong argument | `skd_fnc_step: argument 1 (title) must be a String, got SCALAR 5.` |
| A machine the session lacks | the message names the machines the session has. |

Fix the file and run the check again.
