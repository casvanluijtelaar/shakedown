---
type: Documentation
title: Asking a running game
description: How to run SQF on a machine or a group of a running session, wait for a condition, try an assertion live, inspect a machine, read its logs and list the script errors it caught, from the command line, with the output of each and what it does when it fails.
status: draft
---

# Asking a running game

Between runs, and while one runs, a session's machines answer questions from outside any check: to try a line of SQF before you put it in a check, to look at what a failed check left, or to look into a broken session. Every command here has `--json`, which prints exactly what its MCP tool returns ([AI agents and the MCP tools](agent-tools.md)). Each needs a running session and, with none, exits 4:

```
No session runs. Start one with `shakedown session start --setup <name>`; the project declares the setups: sp, dedicated.
```

Machines are named `server`, `p1` to `pN` and `hc1` to `hcN`; in singleplayer `p1` is the only machine and `server` names it too. `exec` also takes the groups `all`, `clients` and `headless`. A name the session does not have exits 3, naming the machines it has. These commands work while a run is active and while the session is broken.

## Run code: `exec`

```
> shakedown exec --machine p1 --code "getPosASL player"
p1  done  [100,100,0.00143862]  (0.004 s)
> shakedown exec --machine all --code "profileName"
server  done  "server"  (0.021 s)
p1  done  "p1"  (0.006 s)
p2  done  "p2"  (0.006 s)
```

```
> shakedown exec --machine server --code "sleep 1; 1 + 1" --json
{
  "errors": [],
  "results": {
    "server": { "duration_s": 1.002, "state": "done", "value": 2 }
  }
}
```

`shakedown exec --machine <m> (--code <sqf> | --file <path>) [--args <json>] [--ceiling <s>]`:

- The code runs scheduled on the machine, so it may `sleep` and `waitUntil`.
- `--args '[42, "a"]'` gives it a JSON array as `_this`. `--file <path>` reads the code from a file.
- The value comes back encoded as the result file encodes values: objects and groups as `{"$type": "object", "netId": ..., "class": ..., "alive": ...}`, and an object given in `--args` that way arrives as the object ([Values from the game](results.md#values-from-the-game)):

  ```
  shakedown exec --machine p1 --code "typeOf (_this select 0)" --args "[{\"$type\": \"object\", \"netId\": \"2:14\"}]"
  ```

- `--ceiling` is the in-game ceiling in seconds, default 10, above 0 and at most 600. Code still running at it is stopped and its state is `ceiling`; a machine that has not answered 10 s after it is `lost`, and its session broken ([Outcomes and a broken session](outcomes-and-broken-sessions.md)).
- A script error the code raises does not stop it: the value comes back, the error is listed in `errors` with its machine, text, file, line and time, and the command exits 0. Its `check_id` is `null`, as it was raised outside any check. Without `--json` it prints `error on p1 at <time>: <text> (no file, line 1) outside any check`.
- Code that does not compile exits 3 with the engine's message, such as `The code does not compile on p1: Invalid number in expression (line 1).`; the message can give the engine's error twice.
- A command stopped before the answer (Ctrl+C, or a cancelled tool call) stops the code on the machine.
- An order sent to a machine while its mission restarts or its process is replaced waits until the machine is ready again, for at most the setup's start ceiling, then runs with its whole ceiling and is answered once.

Every `exec` is recorded in the session's `audit.jsonl` ([Security and privacy](security-and-privacy.md)).

## Wait for a condition: `wait`

```
> shakedown wait --machine p1 --condition "!isNil 'mymod_ready'" --ceiling 20
met after 0.4 s
```

`shakedown wait --machine <m> --condition <sqf> [--description <text>] [--args <json>] [--ceiling <s>]` evaluates the condition on the machine every frame until it returns `true` or the ceiling passes, default 10 s. The answer is `{"met", "waited_s", "error"}`, and the command exits 0 whether the condition was met or not: it prints `met after <s> s`, `not met within its ceiling of <s> s`, or `not met: <error> (after <s> s)`. A condition that raises a script error, or returns anything but a Boolean, ends the wait at that evaluation with `met` `false` and the cause in `error`:

```json
{ "error": "the condition raised a script error: Undefined variable in expression: _undefined (no file, line 1)", "met": false, "waited_s": 0.0 }
```

`wait` takes one machine, not a group.

## Try an assertion: `expect`

```
> shakedown expect --machine p1 --value "damage player" --op "<" --expected 0.5
held: last read 0 after 0 s
```

`shakedown expect --machine <m> --value <sqf> --op <op> --expected <json> [--args <json>] [--ceiling <s>]` reads the value every frame until `value op expected` holds or the ceiling passes, as `skd_fnc_expectOn` does in a check, and answers `{"passed", "last_value", "waited_s"}`; it prints `held` or `did not hold` with the last value read, and exits 0 either way. `--expected` is JSON, so a string is written `'"OFF"'` and a tolerance `'[[100, 100, 0], 0.5]'`. The operators are those of the check API: `==`, `!=`, `<`, `<=`, `>`, `>=`, `~=`, `in` and `contains` ([Waits and assertions](waits-and-assertions.md#the-operators)). Operands that do not fit the operator, such as `<` on a string, exit 3 naming what the operator needs.

## Look at a machine: `inspect`

```
> shakedown inspect --machine p1
p1 (client) ready, start 1
mission shakedown_s-20261106-092156 (start 1), time 312.4 s, tick 401.2 s, 58 fps, frame 18110
player: {"alive":true,"animation":"amovpercmstpsraswrfldnon","class":"B_Soldier_F","damage":0,"direction":0,"position":[100,104,0], ...}
  near: Land_WoodenBox_F 2:14 at 1.02 m
```

`shakedown inspect --machine <m> [--near <netId>] [--radius <m>]` gives a compact summary, read without stopping anything:

- the machine as `status` lists it;
- the mission, its start count and the clocks: `time`, `tick`, `fps` and `frame`;
- the run, check and step running: the step is the innermost step running in the check on the lead and, on another machine, the step whose wait, poll or code runs there, else `[]`;
- the player unit (class, position ASL, direction, damage, alive, vehicle, animation), or `null` on a machine without one;
- up to 50 objects within `--radius` metres (default 50) of the player, or of the object whose net id `--near` gives, each with its class, net id, position and distance;
- up to 50 running scheduled scripts; code `exec` runs is a script named `skd_exec <order>` while it runs, and `inspect` leaves its own out;
- the last 10 script errors of that machine.

```
shakedown inspect --machine server --near 2:14 --radius 20 --json
```

A machine that is `lost` is not asked: `inspect` returns it at once with `state` `lost` and every other field `null`, and one that stops answering while `inspect` waits is returned the same way within 20 s and is then `lost`, its session broken.

## Script errors: `errors`

```
> shakedown errors --machine p1
error on p1 at 2026-11-06T18:08:12.004Z: Undefined variable in expression: _x (no file, line 1) outside any check
error on p1 at 2026-11-06T18:09:40.517Z: Generic error in expression (mymod\addons\main\functions\fn_open.sqf, line 12) in check crate/US-019-open
```

`shakedown errors [--machine <m>] [--run <id>] [--check <id>]` lists every script error the session's machines caught, inside checks and outside, the oldest first, with its machine and time. `--machine`, `--run` and `--check` narrow the list. An error raised outside any check has `check_id` `null`. With none it prints `no script errors`.

## Log lines: `logs`

`shakedown logs --machine <m> [--run <id>] [--check <id>] [--grep <regex>] [--last <n>]` prints a machine's log lines, redacted, from its running RPT, a run or a check ([Reading logs](script-errors-and-logs.md#reading-logs-shakedown-logs)):

```
shakedown logs --machine p2 --check crate/US-019-open --grep "Error" --last 20
```

## A start that failed stays in `status`

A `session start` that fails exits 4 with the reason and the last RPT lines of the machine at fault. `status` then shows that session, state `failed` with the same `reason`, until the next `session start` or `session stop`; `session stop` clears it and exits 0, as nothing of it runs ([Starting a session, and when it fails](starting-and-failed-starts.md)).
