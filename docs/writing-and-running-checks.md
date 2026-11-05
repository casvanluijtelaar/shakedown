---
type: Documentation
title: Writing and running a check
description: How to write a check in SQF with steps, run code on p1, assert and keep evidence, how to list and run checks in a singleplayer session, and what the run leaves behind.
status: draft
---

# Writing and running a check

A check is one `.sqf` file under the project's checks folder. Its contract is [spec/contract/check-api.md](../spec/contract/check-api.md); this page shows the part that is built: a header, steps, code run on `p1`, assertions, evidence and an explicit failure, in a singleplayer session.

## A check

```sqf
// @check US-012 A hook thrown at a wall holds a player's weight
// @tags US-012 smoke

// Code sent to a machine sees only what it is given in args, as _this.
private _answer = ["ask p1 for a value", {
    ["p1", { (_this select 0) + 1 }, [1]] call skd_fnc_on
}] call skd_fnc_step;

["the answer is two", {
    ["compare it", {
        [_this select 0, "==", 2, "1 + 1 on p1"] call skd_fnc_expect;
    }, _this] call skd_fnc_step;
}, [_answer]] call skd_fnc_step;

["answer", _answer] call skd_fnc_evidence;
```

- **The header** is the first lines of the form `// @key value`. `@check` is required: the story or requirement ID the check proves, then a short sentence. `@tags`, `@setups`, `@timeout` (seconds, default 120) and `@reset` are optional.
- **The check id** is the file's path under the checks folder without `.sqf`: `checks/rope/US-012-hook.sqf` is `rope/US-012-hook`.
- **The body** is the rest of the file. It runs on `p1` in singleplayer, as plain SQF: no `#include`, no macro. Comments of both kinds may stand anywhere.
- **Keep long data out of string literals.** The game stands still while it compiles the body, and a long literal compiles slowly: about half a second for 100 KB.

## The functions

| Function | Does |
|---|---|
| `[title, code, args] call skd_fnc_step` | Runs `code` as a named step, with `args` as `_this`, and returns its value. Steps nest. The result records each step's path, state and its clocks at start and end. |
| `[machine, code, args, ceiling] call skd_fnc_on` | Runs `code` on a machine and returns its value. In singleplayer the machine is `"p1"`, which `"server"` also names. The code is compiled from its text there, so it does not see the check's local variables: give it what it needs in `args`. `ceiling` defaults to 10 s. |
| `[actual, op, expected, description] call skd_fnc_expect` | Asserts at once. `op` is one of `==`, `!=`, `<`, `<=`, `>`, `>=`, `~=` (expected `[target, tolerance]`), `in`, `contains`. |
| `[label, value] call skd_fnc_evidence` | Adds a value to the result, for a passing check as well as a failing one. |
| `[message] call skd_fnc_fail` | Ends the check `fail` / `explicit`. |

A function that fails does not return: the check ends there, and the reason is recorded against the innermost step.

## When the check itself is wrong

A broken check ends `fail` with reason `check-error` and a message that says what to fix. It is never `invalid`, and the next check of the run still runs.

| Mistake | Message |
|---|---|
| No `@check` line | `The check header is wrong, the header has no @check line. ...` |
| `// @timeout fast` | `... line 2: @timeout must be a whole number of seconds from 1 to 3600, got `fast`. ...` |
| A key given twice | `... line 3: @tags is given twice, first on line 2. Keep one. ...` |
| A syntax error | `The check body does not compile: Invalid number in expression, at line 4 of the check file. ...` |
| `skd_fnc_*` inside code given to `skd_fnc_on` | `skd_fnc_expect was called inside code sent to a machine; skd_fnc_* functions run only in the check body. ...` |
| A wrong argument | `skd_fnc_step: argument 1 (title) must be a String, got SCALAR 5.` |

A header mistake is found without the game. Fix the file and run the check again: the runner reads every check from disk at the start of each execution, so no reload is needed.

## Running checks

| Command | Does |
|---|---|
| `shakedown list [<id>...] [--tag <tag>]...` | Lists checks from the files, with no game. A check whose header is wrong is listed with what is wrong. |
| `shakedown run <id>... --setup sp` | Starts a session of setup `sp`, runs the checks, stops the session. |
| `shakedown session start --setup sp` | Starts a session and returns when it is ready. |
| `shakedown run <id>...` | Runs the checks against the running session, which stays up. |
| `shakedown session stop` | Stops the session and returns when its game process is gone. |

With no id and no tag, `run` runs every check; with tags, every check carrying any of them. Ids keep the order given; checks a tag selects run in id order.

A session start takes the game's own start time, 40 to 60 s on the test machine. A session that is kept up runs any number of runs without restarting the game.

`run` prints one line per check as it ends, then a summary with the path of the result file:

```
pass    smoke  (0.0 s)
1 passed, 0 failed, 0 invalid, 0 not run. Result: C:\...\e2e\.shakedown\runs\r-20261105-161317-1\result.json
```

Exit codes: 0 every check passed, 1 a check failed, 2 none failed and one was invalid, 3 the request is wrong (an unknown check or setup, a malformed project file), 4 it cannot be done now (no session, a session already running, the game or Steam missing), 5 the framework failed (see `runner.log`). Add `--json` to any command for one JSON document on stdout.

## What a run leaves behind

Each run writes `.shakedown\runs\<run id>\result.json` in the project, valid against [spec/contract/result.schema.json](../spec/contract/result.schema.json). It holds the run, the session with `p1`'s game version and its start number, the summary, and for each check its outcome, reason, steps with their clocks, and evidence. A session's folder, `.shakedown\sessions\<session id>\`, holds the generated mission and `p1`'s profile folder with its RPT.

Not built yet: the reset between checks, script errors in the result, log excerpts, and dedicated sessions. Until the reset comes, what one check changes in the game is still there for the next.
