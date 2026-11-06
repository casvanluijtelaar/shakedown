---
type: Documentation
title: Asking a running game
description: How to run code on a machine of a running session, wait for a condition, try an assertion live, inspect a machine and list the script errors it caught, from the command line, and what status shows of a start that failed.
status: draft
---

# Asking a running game

Between runs, and while one runs, a session's machines answer questions from outside any check. Every command here has `--json`, which prints exactly what its MCP tool returns ([Agent tools](agent-tools.md)). Their contract is [spec/contract/cli.md](../spec/contract/cli.md); each needs a running session and, with none, exits 4 naming `shakedown session start --setup <name>` and the setups the project declares.

Machines are named `server`, `p1` to `pN` and `hc1` to `hcN`; in singleplayer `p1` is the only machine and `server` names it too. `exec` also takes the groups `all`, `clients` and `headless`. A name the session does not have exits 3, naming the machines it has.

## Run code: `exec`

```
shakedown exec --machine server --code "sleep 1; 1 + 1" --json
```

```json
{
  "errors": [],
  "results": {
    "server": { "duration_s": 1.002, "state": "done", "value": 2 }
  }
}
```

The code runs scheduled on the machine, so it may `sleep` and `waitUntil` (`CTL-03`). `--args '[42, "a"]'` gives it a JSON array as `_this`; `--file <path>` reads the code from a file. The value comes back encoded as the result file encodes values: objects and groups as `{"$type": "object", "netId": ...}`, and an object given in `--args` that way arrives as the object (`DM-13`, `DM-14`).

- `--ceiling` is the in-game ceiling in seconds, default 10, at most 600. Code still running at it is stopped and its state is `ceiling`; a machine that has not answered 10 s after it is `lost`.
- A script error the code raises does not stop it (`CTL-05`): the value comes back, the error is listed in `errors` with its machine, text, file, line and time, and the command exits 0. Its `check_id` is `null`, as it was raised outside any check.
- Code that does not compile exits 3 with the engine's message, such as `The code does not compile on p1: Invalid number in expression (line 1).`
- A command stopped before the answer (Ctrl+C, or a cancelled tool call) stops the code on the machine.

## Wait for a condition: `wait`

```
shakedown wait --machine p1 --condition "!isNil 'skdfx_flag'" --ceiling 20 --json
```

The condition is evaluated on the machine every frame until it returns `true` or the ceiling passes (`OBS-01`). The answer is `{"met", "waited_s", "error"}`, and the command exits 0 whether the condition was met or not. A condition that raises a script error, or returns anything but a Boolean, ends the wait at that evaluation with `met` `false` and the cause in `error`:

```json
{ "error": "the condition raised a script error: Undefined variable in expression: _skdfx_undefined (no file, line 1)", "met": false, "waited_s": 0.0 }
```

`wait` takes one machine, not a group.

## Try an assertion: `expect`

```
shakedown expect --machine p1 --value "damage player" --op "<" --expected 0.5 --json
```

Reads the value every frame until `value op expected` holds or the ceiling passes, as `skd_fnc_expectOn` does in a check (`OBS-06`), and answers `{"passed", "last_value", "waited_s"}`. `--expected` is JSON. The operators are those of the check API: `== != < <= > >= ~= in contains`. Operands that do not fit the operator, such as `<` on a string, exit 3 naming what the operator needs.

## Look at a machine: `inspect`

```
shakedown inspect --machine p1
shakedown inspect --machine server --near 2:14 --radius 20 --json
```

A compact summary, read without stopping anything (`INS-01`): the machine as `status` lists it; the mission, its start count and the clocks (`time`, `tick`, `fps`, `frame`); the run, check and step running; the player unit (class, position ASL, direction, damage, alive, vehicle, animation) or `null` on a machine without one; up to 50 objects within `--radius` metres (default 50) of the player or of the object whose net id `--near` gives; up to 50 running scheduled scripts; and the last 10 script errors of that machine. Code `exec` runs is a script named `skd_exec <order>` while it runs; `inspect` leaves its own out.

## Script errors: `errors`

```
shakedown errors --machine p1
shakedown errors --run r-20261106-180800-1 --check fails --json
```

Every script error the session's machines caught, inside checks and outside, the oldest first, with its machine and time (`ERR-05`). `--machine`, `--run` and `--check` narrow the list. An error raised outside any check has `check_id` `null`.

## A start that failed stays in `status`

A `session start` that fails exits 4 with the reason and the last RPT lines of the machine at fault. `status` then shows that session, state `failed` with the same `reason`, until the next `session start` or `session stop`; `session stop` clears it and exits 0, as nothing of it runs (`INS-02`). A program that started the session in the background, such as an AI agent through `session_start`, follows the start with `status` alone.
