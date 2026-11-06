---
type: Documentation
title: Writing and running a check
description: How to write a check in SQF - its file, header and body - how to list and run checks in a session, see it with status, reset it by hand, the reset between checks, and what the run leaves behind.
status: draft
---

# Writing and running a check

A check is one `.sqf` file under the project's checks folder. Its contract is [spec/contract/check-api.md](../spec/contract/check-api.md); this page covers the file and running it in a session, [The check API by example](check-api.md) shows every function a check can call, and [Dedicated sessions](dedicated-sessions.md) covers the server, clients and headless clients.

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

- **The header** is the first lines of the form `// @key value`. `@check` is required: the story or requirement ID the check proves, such as `US-012` or `NFR-PF-02`, then a short sentence. `@tags`, `@setups` (setups of the project file, separated by spaces), `@timeout` (seconds, default 120) and `@reset` are optional. A file saved with a UTF-8 byte order mark reads the same as one without.
- **The check id** is the file's path under the checks folder without `.sqf`: `checks/rope/US-012-hook.sqf` is `rope/US-012-hook`.
- **The body** is the rest of the file. It runs on the lead machine, `p1` in singleplayer and the server on a dedicated session, as plain SQF: no `#include`, no macro. Comments of both kinds may stand anywhere. The server has no player, so a check that runs in both names a player's unit with `["p1"] call skd_fnc_player`, not `player`.
- **Keep long data out of string literals.** The game stands still while it compiles the body, and a long literal compiles slowly: about half a second for 100 KB.

## The functions used above

| Function | Does |
|---|---|
| `[title, code, args] call skd_fnc_step` | Runs `code` as a named step, with `args` as `_this`, and returns its value. Steps nest. The result records each step's path, state and its clocks at start and end. |
| `[machine, code, args, ceiling] call skd_fnc_on` | Runs `code` on a machine and returns its value. In singleplayer the machine is `"p1"`, which `"server"` also names. The code is compiled from its text there, so it does not see the check's local variables: give it what it needs in `args`. `ceiling` defaults to 10 s. |
| `[actual, op, expected, description] call skd_fnc_expect` | Asserts at once. `op` is one of `==`, `!=`, `<`, `<=`, `>`, `>=`, `~=` (expected `[target, tolerance]`), `in`, `contains`. |
| `[label, value] call skd_fnc_evidence` | Adds a value to the result, for a passing check as well as a failing one. |
| `[message] call skd_fnc_fail` | Ends the check `fail` / `explicit`. |

A function that fails does not return: the check ends there, and the reason is recorded against the innermost step. Waits, polled assertions, the scene, actions, keybinds and cleanups are in [The check API by example](check-api.md).

## When the check itself is wrong

A broken check ends `fail` with reason `check-error` and a message that says what to fix. It is never `invalid`, and the next check of the run still runs.

| Mistake | Message |
|---|---|
| No `@check` line | `The check header is wrong, the header has no @check line. ...` |
| `// @check A hook holds` | `... line 1: @check must start with the story or requirement ID the check proves, such as US-012 or NFR-PF-02, then a short sentence; it starts with `A`.` |
| `// @timeout fast` | `... line 2: @timeout must be a whole number of seconds from 1 to 3600, got `fast`. ...` |
| A key given twice | `... line 3: @tags is given twice, first on line 2. Keep one. ...` |
| `// @setups nosuch` | `... line 2: @setups names the setup `nosuch`, which shakedown.toml does not declare; it declares: sp, dedicated. Fix the name. ...` |
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
| `shakedown session stop` | Stops the session and returns when its game processes are gone. |
| `shakedown status` | The runner, the session and each of its machines with its state, process id and profile folder, and the run in progress. Starts nothing. |
| `shakedown reset` | Resets the running session by hand, as before a check, and says how long it took. |

With no id and no tag, `run` runs every check; with tags, every check carrying any of them. Ids keep the order given, and an id given more than once runs that check once per time: `run a a b` runs `a`, `a`, `b`, and `run a b a` runs `a`, `b`, `a`. Checks a tag selects run after them in id order, each once. `list` lists each selected check once.

A session start takes the game's own start time, 40 to 60 s on the test machine. A session that is kept up runs any number of runs without restarting the game. A check whose `@setups` leaves out the session's setup is listed as not run. Before every check the session is reset to the state the mission started in, in under a second; [The check API by example](check-api.md#the-reset-between-checks) says what the reset restores and what it leaves.

`run` prints one line per check as it ends, then a summary with the path of the result file:

```
pass    smoke  (0.0 s)
1 passed, 0 failed, 0 invalid, 0 not run. Result: C:\...\e2e\.shakedown\runs\r-20261105-161317-1\result.json
```

Exit codes: 0 every check passed, 1 a check failed, 2 none failed and one was invalid, 3 the request is wrong (an unknown check or setup, a malformed project file), 4 it cannot be done now (no session, a session already running, the game or Steam missing), 5 the framework failed (see `runner.log`). Add `--json` to any command for one JSON document on stdout.

## What a run leaves behind

Each run writes `.shakedown\runs\<run id>\result.json` in the project, valid against [spec/contract/result.schema.json](../spec/contract/result.schema.json). It holds the run, the session with each machine's kind, game version, mod versions and start number, the summary, and for each check execution its outcome, reason, steps with their clocks, and evidence. A check run twice has two entries in `checks`, with `repeat` 1 and 2, and prints two lines, the second with `#2` right after the check id, as in `pass    sel/RUN-01-a #2  (0.0 s)`. A session's folder, `.shakedown\sessions\<session id>\`, holds the generated mission, a dedicated session's `server.cfg`, and each machine's profile folder `machines\<machine>-1` with its RPT.

What a check leaves in the game stays there until the next check's reset, so a person can look at it in the game window after a run. Not built yet: script errors in the result and log excerpts.
