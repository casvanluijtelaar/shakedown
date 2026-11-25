---
type: Documentation
title: AI agents and the MCP tools
description: How an AI agent drives Shakedown through the shakedown MCP server - the project's .mcp.json, every tool with its input and its command line twin, the rules every tool keeps, the error codes, the agent's workflow for a session and a run, and the text the guide tool returns.
status: draft
---

# AI agents and the MCP tools

`shakedown mcp` is an MCP server on stdio that gives an AI agent, such as Claude Code, every command of the command line as a tool. A tool asks the runner the same requests as its command and returns the same JSON that the command prints with `--json`, so whatever these docs say of a command holds for its tool.

With it an agent can start and stop sessions, list and run checks, read results, script errors and each machine's logs, run SQF on any machine and inspect it, reload changed files and rerun, and write checks: the `guide` tool tells it how.

## Setting it up

The project, the folder holding `shakedown.toml`, declares the server in a `.mcp.json` beside it:

```json
{
  "mcpServers": {
    "shakedown": { "command": "shakedown", "args": ["mcp"], "timeout": 70000 }
  }
}
```

`shakedown` must be on `PATH`: put the install folder there ([Installing, upgrading and removing](install.md)). Start Claude Code in that folder. It asks once whether to trust the project's servers, then shows the tools as `mcp__shakedown__<tool>`. The server finds the project from the folder it is started in. It starts and answers `status`, `list_checks` and `guide` on a machine with no game and no runner, macOS and Linux included; every other tool reaches the runner, which it starts when none runs.

The session belongs to the runner, not to the agent: when the agent's session ends, the game runs on, and the next agent session's `status` finds it ([Sessions and the runner](sessions-and-the-runner.md)). Have the agent call `session_stop` when it is done.

## The tools

| Tool | Input | Command | Does |
|---|---|---|---|
| `status` | `{}` | `shakedown status` | The runner, the session with its machines and their health, the active run, the last reload. Shows a failed start until the next start or stop. |
| `session_start` | `{"setup"}` | `shakedown session start --setup <name>` | Starts a session; returns when it is `ready` or `failed`, or after 50 s with state `starting`. |
| `session_stop` | `{}` | `shakedown session stop` | Stops the session, or clears a failed start. |
| `list_checks` | `{"ids"?, "tags"?}` | `shakedown list` | The checks a selection takes, from the files, with no runner: each with `id`, `name`, `tags`, `setups`, `timeout_s`, `reset` and `error`. |
| `run_start` | `{"ids"?, "tags"?, "setup"?, "each_setup"?, "repeat"?}` | `shakedown run` | Starts a run and returns its `run_id`, its state and `total` at once. `setup` with no session starts one and stops it at the end; `repeat` runs each check that many times in a row; `each_setup` runs the selection in every setup a selected check names ([Every setup, repeats and slow steps](every-setup-and-repeats.md)). |
| `run_status` | `{"run_id", "wait_s"?}` | - | A run's state, progress, current check and outcomes so far; with `wait_s` (at most 50) it returns as soon as a check ends or the run ends. |
| `run_result` | `{"run_id"}` | `shakedown run --json` | The result of a run that ended: the summary, each outcome with its reason, failed step and evidence path, the checks not run, repeats, `handed_over` and `result_file`. |
| `run_stop` | `{"run_id"}` | `shakedown run stop` | Stops a run and returns its status, state `stopped`, once its result file is written ([Running a suite unattended](unattended-runs.md)). |
| `reset` | `{}` | `shakedown reset` | Resets the session in place: `{"state", "duration_s", "mismatches"}`. |
| `reload` | `{"files"?}` | `shakedown reload` | Reloads changed files, or the `files` given relative to the project, at the level they need; on a broken session restarts the game ([Reloading changed mod files](reloading.md)). Returns when done, or after 50 s with state `applying`. |
| `exec` | `{"machine", "code", "args"?, "ceiling"?}` | `shakedown exec` | SQF on a machine or a group, its value as data and the script errors it raised. |
| `wait_until` | `{"machine", "condition", "description"?, "args"?, "ceiling"?}` | `shakedown wait` | Waits for a condition on one machine: `{"met", "waited_s", "error"}`. |
| `expect` | `{"machine", "value", "op", "expected", "args"?, "ceiling"?}` | `shakedown expect` | Tries an assertion live: `{"passed", "last_value", "waited_s"}`. |
| `inspect` | `{"machine", "near"?, "radius"?}` | `shakedown inspect` | A compact summary of a machine ([Asking a running game](asking-a-running-game.md)). |
| `logs` | `{"machine", "run_id"?, "check_id"?, "grep"?, "last"?}` | `shakedown logs` | A machine's log lines, redacted. |
| `errors` | `{"machine"?, "run_id"?, "check_id"?}` | `shakedown errors` | The script errors the session caught. |
| `guide` | `{}` | `shakedown guide` | The text at the end of this page. |

`status`, `list_checks`, `run_status`, `run_result`, `inspect`, `logs`, `errors` and `guide` are marked read-only; `exec` is marked destructive, so a client can ask before it runs.

## What every tool keeps to

- **Structured results.** Each tool has an `inputSchema` and an `outputSchema`, and returns `structuredContent` and the same JSON as one text block. The shapes a tool shares with the result file, such as `ScriptError`, are those of the result file ([Results](results.md)).
- **No call past 60 s.** `exec`, `wait_until` and `expect` take a `ceiling` of at most 50 s, default 10; `run_status` a `wait_s` of at most 50. A call that waits sends `notifications/progress` every 4 s when the request carries a progress token. A run never holds a call: `run_start` returns at once. An order sent to a machine whose mission restarts waits for the machine before its own ceiling starts, so on a dedicated session such a call can last longer than 60 s.
- **Cancelling stops the work.** When the client cancels `exec`, `wait_until`, `expect` or `inspect`, the server closes the call's connection to the runner, and the runner stops the code on the machine; `inspect` then lists no script of it. A session start or a reset goes on when its call is cancelled; stop a session with `session_stop`. A run never holds a call; stop it with `run_stop`.
- **Long results go to a file.** A result over 20,000 characters is written to `.shakedown\results\` in the project and the tool returns `{"result_path": <path>}`. With no project, the file goes to a `shakedown-results` folder in the system's temp folder. `logs` keeps its own lines under the limit and names the file with all of them in `path`.
- **A failing check is a result.** `run_result` of a run whose checks failed is a successful call; their outcomes say `fail`.

## Mistakes the agent can fix

Such a mistake is a result with `isError` `true` and one text block, `<code>: <what is wrong>. <what to do next>.`:

```
unknown-machine: Session s-20261106-180721 has no machine `p9`; its machines are p1, and the groups all, clients and headless. Name one of those.
```

| Code | When |
|---|---|
| `bad-arguments` | An argument does not match the input schema, a ceiling or `wait_s` is above 50, values conflict (such as `each_setup` with `setup`), or the operands of `expect` do not fit its operator. |
| `config-error` | `shakedown.toml` is missing or malformed; the message names the key and its line. |
| `unknown-check` | An id or a tag selects nothing; the message lists the checks or the tags that exist. |
| `unknown-machine` | The session has no such machine. |
| `unknown-setup` | The project has no such setup; the message lists those it declares. |
| `unknown-run` | No run has that id, or, from `run_result`, the run has not ended: call `run_status`. |
| `compile-error` | The SQF does not compile on the machine; the engine's message follows. |
| `no-session` | The tool needs a session and none runs; the message names the start command and the setups of the project. |
| `session-exists` | A session runs already, or one of another setup than the `setup` given to `run_start`, or one runs while `each_setup` asks to start its own. |
| `session-busy` | The session is starting or stopping, or busy with a run, a reload or a reset. |
| `session-broken` | The session is broken; the reason follows. |
| `precondition-failed` | Steam, the game, a mod folder, free disk, a port or the framework's addon is missing; every gap found follows, in one message. |
| `runner-unreachable` | The runner does not answer or runs another version; its process id follows. Ending that process ends its session ([Sessions and the runner](sessions-and-the-runner.md#a-command-cannot-reach-the-runner)). |

A failure of the framework itself is a JSON-RPC error, not a tool result.

## The agent's workflow

1. `guide` once, then `status`.
2. `session_start` with a setup. When it returns `starting`, call `status` until the session is `ready` or `failed`; a failed start keeps its `reason` there.
3. `run_start` with `ids` or `tags`, then `run_status` with `wait_s` 50 until its state is `finished` or `stopped`: each call returns as soon as a check ends, with every outcome so far. `run_stop` stops a run that should not go on.
4. `run_result` for the summary, each check's reason and failed step, and the path of the result file. `logs` with `check_id` and `run_id` gives a check's log lines.
5. To look into a failure: `inspect` the machine, `exec` code on it, `wait_until` and `expect` to try what the check does, `errors` for the script errors.
6. To try a fix: edit the check and run it again, or edit the mod, `reload`, and run again.
7. `session_stop` when done.

## The guide

`guide` and `shakedown guide` return this text:

```text
Shakedown runs automated checks of an Arma 3 mod inside the running game, in singleplayer or on a dedicated server with game clients and headless clients.

THE PROJECT
The project is the folder holding shakedown.toml. It names the game, the mods, and the setups: a setup is a mode (singleplayer or dedicated), its clients and headless clients, and the mods it loads. Checks are .sqf files under the tests folder (shakedown.toml's tests, default tests/); a check's id is its path there without .sqf. Helpers shared by checks go in the library folder (tests/_lib/ by default, or shakedown.toml's lib): each file kit.sqf there is compiled on every machine into skd_lib_kit, which a check or code sent to a machine runs with call skd_lib_kit; a changed library file reaches the game with reload.

START A SESSION
1. status: is a runner up, does a session run? It works with no game.
2. session_start {"setup": "<name>"}: starts the game processes. It returns when the session is ready or failed, or after 50 s with state starting: then call status until the session is ready or failed. A failed session carries reason.message and the last RPT lines of the machine at fault.
3. One session serves many runs; start it once. session_stop ends every process of it.
Machines are named server, p1..pN (game clients) and hc1..hcN (headless clients). In singleplayer p1 is the only machine and server names it too. Groups: all, clients, headless.

RUN CHECKS
1. list_checks {"tags": ["US-019"]} shows which checks a selection takes, without the game.
2. run_start {"ids": [...]} or {"tags": [...]} returns a run_id at once. ids run in the order given; an id given twice runs twice. With "setup" and no session, the run starts one and stops it at its end, unless a check hands over to a manual tester (skd_fnc_handOver): then run_result has handed_over and the session stays up. "repeat": n runs each check n times in a row, and run_result then has repeats: each check's summary, pass, fail, flaky (both came up) or invalid, with the count of each outcome. With "each_setup": true and no session, it runs the selection in every setup a selected check names, one session after the other: run_id is then a handle for all of them, run_status names the setup it is in, and run_result has a runs entry per setup.
3. run_status {"run_id": "...", "wait_s": 50} returns as soon as a check ends, with each outcome so far. Call it again until state is finished, or stopped: a run stops at its runner ceiling (the sum of its checks' @timeout plus 60 s) or by run_stop {"run_id": "..."}; the check it was on is then invalid with reason run-stopped and the rest are in not_run.
4. run_result {"run_id": "..."}: the summary, each check's outcome, reason and failed step, and result_file, the full result as JSON.

READ A RESULT
Each check execution ends pass, fail or invalid. fail is the mod's or the check's fault, with reason.kind: assertion, wait-ceiling, script-error, expected-error-missing, check-timeout, explicit, or check-error (the check itself is wrong: fix it). invalid means the framework could not judge (a machine lost, a reset failed): run it again. The result file holds per check its steps, errors (script errors with machine, step, file and line), evidence.failed_step, evidence.observed (the last value each machine read) and evidence.logs (each machine's log lines for the check). A check that fails is a normal result, never a tool error.

ASK THE RUNNING GAME
- exec {"machine": "p1", "code": "getPosASL player"}: runs SQF and returns the value as data, and any script errors it raised. args become _this. ceiling at most 50 s.
- wait_until {"machine": "server", "condition": "!isNil 'my_flag'", "ceiling": 20}: polls every frame; never sleep a fixed time.
- expect {"machine": "p1", "value": "damage player", "op": "<", "expected": 0.5}: an assertion tried live.
- inspect {"machine": "p1"}: the player, nearby objects, running scripts, the last errors, the check running.
- logs {"machine": "p1", "check_id": "...", "grep": "Error", "last": 50}: a machine's log lines.
- errors {"machine": "p1"}: script errors caught in the session, in checks and outside.
- reset: puts the session back to its start state; every check starts with one anyway.
- reload {"files": ["tests/x.sqf"]}: reloads changed files, or with none what changed under the mods' watched source folders, and says the level each needed and why. On a broken session it restarts the game, which brings every machine back.

WRITE A CHECK
A check is a file tests/<id>.sqf. Its header is comment lines:
  // @check US-019 The crate opens with the mod's action
  // @tags US-019 crate
  // @setups sp            (optional: only these setups)
  // @timeout 60           (optional, seconds, default 120)
  // @reset mission        (optional: start in a restarted mission, for state a reset cannot name)
The body is SQF run on the lead machine (server, or p1 in singleplayer), with the skd_fnc_* functions:
  ["open the crate", {
      private _crate = ["mymod_Crate", AGLToASL [100, 105, 0]] call skd_fnc_create;
      ["p1", _crate, "Open crate"] call skd_fnc_useAction;
      ["server", {(_this select 0) getVariable ["mymod_open", false]}, "crate open on the server", 10, [_crate]] call skd_fnc_waitUntil;
  }] call skd_fnc_step;
  ["p1", {damage player}, "<", 0.5, "p1 unhurt"] call skd_fnc_expectOn;
- ["p2", {code}, [args]] call skd_fnc_on runs code on a machine and returns its value; code sent to a machine sees only _this, not the check's local variables.
- skd_fnc_waitUntil [machine, condition, description, ceiling, args] waits with a ceiling; skd_fnc_tryWaitUntil returns false at the ceiling instead of failing.
- skd_fnc_expectOn [machine, value, op, expected, description, ceiling, args]; skd_fnc_expectEach for several machines; skd_fnc_expectAgree for machines that must read the same.
- skd_fnc_expectError [text, code] for an error the mod should raise; any other script error on any machine fails the check.
- skd_fnc_evidence [label, value] attaches a value to the result.
- ["p1", _crate, ["ACE_MainActions", "myAction"]] call skd_fnc_useAceAction uses a mod's ACE interaction action by its path of action names; with "p1" as the target, p1's own menu under ACE_SelfActions. Needs ACE in the setup.
- On a dedicated session, ["p3"] call skd_fnc_join starts a late client (setup key late) or a client that left, and ["p2"] call skd_fnc_leave makes a client leave; the next check's reset brings back the clients the session started with.
- ["all", "my_event"] call skd_fnc_record records a CBA event (or [machines, "EntityKilled", "mission"] a mission event) on those machines until the check ends; ["p2", "my_event", 10] call skd_fnc_waitEvent waits for one and returns its arguments; ["p2", "my_event"] call skd_fnc_events lists them with their clocks; the result file lists them under the check.
Every check starts from a reset session, so it needs no cleanup of what it created. Run it with run_start, read the outcome, fix, and run it again: the file is read fresh each time. The project's docs folder has the whole check API.
```
