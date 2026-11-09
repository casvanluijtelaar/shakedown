---
type: Documentation
title: Agent tools
description: How an AI agent drives Shakedown through the shakedown MCP server - the project's .mcp.json, every tool and its command line twin, the rules every tool keeps, the agent's workflow for a session and a run, the error codes, and the text the guide tool returns.
status: draft
---

# Agent tools

`shakedown mcp` is an MCP server on stdio that gives an AI agent every command of the command line as a tool (`AGT-01`). A tool asks the runner the same requests as its command and returns the same JSON that the command prints with `--json` (`NFR-US-03`), so whatever [Asking a running game](asking-a-running-game.md) and [Writing and running a check](writing-and-running-checks.md) say of a command holds for its tool. The contract is [spec/contract/mcp-tools.md](../spec/contract/mcp-tools.md).

## Setting it up

The project, the folder holding `shakedown.toml`, declares the server in a `.mcp.json` beside it (`INT-CC-01`):

```json
{
  "mcpServers": {
    "shakedown": { "command": "shakedown", "args": ["mcp"], "timeout": 70000 }
  }
}
```

`shakedown` must be on `PATH`: put the install folder there ([Installing Shakedown](install.md)). Claude Code asks once whether to trust the project's servers, then shows the tools as `mcp__shakedown__<tool>`. The server finds the project from the folder Claude Code starts it in. It starts and answers `status`, `list_checks` and `guide` on a machine with no game and no runner (`AGT-05`); every other tool reaches the runner, which it starts when none runs.

The demo project `e2e/grappling-hook` has such a `.mcp.json`: start Claude Code in that folder.

## The tools

| Tool | Command | Does |
|---|---|---|
| `status` | `shakedown status` | The runner, the session with its machines, the active run. Shows a failed start until the next start or stop. |
| `session_start` | `shakedown session start --setup <name>` | Starts a session; returns when it is `ready` or `failed`, or after 50 s with state `starting`. |
| `session_stop` | `shakedown session stop` | Stops the session, or clears a failed start. |
| `list_checks` | `shakedown list` | The checks a selection takes, from the files, with no runner. |
| `run_start` | `shakedown run` | Starts a run and returns its `run_id` at once. |
| `run_status` | - | A run's progress and outcomes so far; with `wait_s` it waits for the next check to end. |
| `run_result` | `shakedown run --json` | The result of a run that ended: the same JSON `run --json` prints. |
| `reset` | `shakedown reset` | Resets the session in place. |
| `exec` | `shakedown exec` | SQF on a machine or a group, its value as data and the script errors it raised. |
| `wait_until` | `shakedown wait` | Waits for a condition on a machine. |
| `expect` | `shakedown expect` | Tries an assertion live. |
| `inspect` | `shakedown inspect` | A compact summary of a machine. |
| `logs` | `shakedown logs` | A machine's log lines, redacted. |
| `errors` | `shakedown errors` | The script errors the session caught. |
| `guide` | `shakedown guide` | The text below. |

`run_stop` and `reload` come with the milestones that build their commands.

## What every tool keeps to

- **Structured results.** Each tool has an `inputSchema` and an `outputSchema`, and returns `structuredContent` and the same JSON as one text block (`AGT-02`). The shapes a tool shares with the result file, such as `ScriptError`, are those of [result.schema.json](../spec/contract/result.schema.json).
- **No call past 60 s** (`AGT-03`). `exec`, `wait_until` and `expect` take a `ceiling` of at most 50 s, default 10; `run_status` a `wait_s` of at most 50. A call that waits sends `notifications/progress` every 4 s when the request carries a progress token. A run never holds a call: `run_start` returns at once.
- **Cancelling stops the work** (`AGT-04`). When the client cancels `exec`, `wait_until`, `expect` or `inspect`, the server closes the call's connection to the runner, and the runner stops the code on the machine; `inspect` then lists no script of it. A session start and a run go on when their call is cancelled; stop them with `session_stop`.
- **Long results go to a file** (`AGT-07`). A result over 20,000 characters is written to `.shakedown/results/` in the project and the tool returns `{"result_path": <path>}`. `logs` keeps its own lines under the limit and names the file with all of them in `path`.
- **A failing check is a result.** `run_result` of a run whose checks failed is a successful call; their outcomes say `fail`.

## Mistakes the agent can fix

Such a mistake is a result with `isError` `true` and one text block, `<code>: <what is wrong>. <what to do next>.` (`AGT-02`, `NFR-US-01`):

```
unknown-machine: Session s-20261106-180721 has no machine `p9`; its machines are p1, and the groups all, clients and headless. Name one of those.
```

| Code | When |
|---|---|
| `bad-arguments` | An argument does not match the input schema, a ceiling or `wait_s` is above 50, or the operands of `expect` do not fit its operator. |
| `config-error` | `shakedown.toml` is missing or malformed; the message names the key and its line. |
| `unknown-check` | An id or a tag selects nothing; the message lists the checks or the tags that exist. |
| `unknown-machine` | The session has no such machine. |
| `unknown-setup` | The project has no such setup; the message lists those it declares. |
| `unknown-run` | No run has that id, or, from `run_result`, the run has not ended: call `run_status`. |
| `compile-error` | The SQF does not compile on the machine; the engine's message follows. |
| `no-session` | The tool needs a session and none runs; the message names the start command and the setups of the project. |
| `session-exists` | A session runs already, or one of another setup than the `setup` given to `run_start`. |
| `session-busy` | The session is starting or stopping, or busy with a run or a reset. |
| `session-broken` | The session is `broken`; the reason follows. |
| `precondition-failed` | Steam, the game, a mod folder or the framework's addon is missing; the message names it. |
| `runner-unreachable` | The runner does not answer or runs another version; its process id follows. Ending that process ends its session ([When a client or the runner ends](sessions-and-the-runner.md)). |

A failure of the framework itself is a JSON-RPC error, not a tool result.

## The agent's workflow

1. `guide` once, then `status`.
2. `session_start` with a setup. When it returns `starting`, call `status` until the session is `ready` or `failed`; a failed start keeps its `reason` there.
3. `run_start` with `ids` or `tags`, then `run_status` with `wait_s` 50 until its state is `finished`: each call returns as soon as a check ends, with every outcome so far.
4. `run_result` for the summary, each check's reason and failed step, and the path of the result file. `logs` with `check_id` and `run_id` gives a check's log lines.
5. To look into a failure: `inspect` the machine, `exec` code on it, `wait_until` and `expect` to try what the check does, `errors` for the script errors.
6. `session_stop` when done.

## The guide

`guide` and `shakedown guide` return this text (`AGT-06`):

```text
Shakedown runs automated checks of an Arma 3 mod inside the running game, in singleplayer or on a dedicated server with game clients and headless clients.

THE PROJECT
The project is the folder holding shakedown.toml. It names the game, the mods, and the setups: a setup is a mode (singleplayer or dedicated), its clients and headless clients, and the mods it loads. Checks are .sqf files under checks/; a check's id is its path there without .sqf.

START A SESSION
1. status: is a runner up, does a session run? It works with no game.
2. session_start {"setup": "<name>"}: starts the game processes. It returns when the session is ready or failed, or after 50 s with state starting: then call status until the session is ready or failed. A failed session carries reason.message and the last RPT lines of the machine at fault.
3. One session serves many runs; start it once. session_stop ends every process of it.
Machines are named server, p1..pN (game clients) and hc1..hcN (headless clients). In singleplayer p1 is the only machine and server names it too. Groups: all, clients, headless.

RUN CHECKS
1. list_checks {"tags": ["US-019"]} shows which checks a selection takes, without the game.
2. run_start {"ids": [...]} or {"tags": [...]} returns a run_id at once. ids run in the order given; an id given twice runs twice. With "setup" and no session, the run starts one and stops it at its end.
3. run_status {"run_id": "...", "wait_s": 50} returns as soon as a check ends, with each outcome so far. Call it again until state is finished.
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

WRITE A CHECK
A check is a file checks/<id>.sqf. Its header is comment lines:
  // @check US-019 The crate opens with the mod's action
  // @tags US-019 crate
  // @setups sp            (optional: only these setups)
  // @timeout 60           (optional, seconds, default 120)
The body is SQF run on the lead machine (server, or p1 in singleplayer), with the skd_fnc_* functions:
  ["open the crate", {
      private _crate = ["skdfx_Crate", AGLToASL [100, 105, 0]] call skd_fnc_create;
      ["p1", _crate, "Open crate"] call skd_fnc_useAction;
      ["server", {(_this select 0) getVariable ["skdfx_open", false]}, "crate open on the server", 10, [_crate]] call skd_fnc_waitUntil;
  }] call skd_fnc_step;
  ["p1", {damage player}, "<", 0.5, "p1 unhurt"] call skd_fnc_expectOn;
- ["p2", {code}, [args]] call skd_fnc_on runs code on a machine and returns its value; code sent to a machine sees only _this, not the check's local variables.
- skd_fnc_waitUntil [machine, condition, description, ceiling, args] waits with a ceiling; skd_fnc_tryWaitUntil returns false at the ceiling instead of failing.
- skd_fnc_expectOn [machine, value, op, expected, description, ceiling, args]; skd_fnc_expectEach for several machines; skd_fnc_expectAgree for machines that must read the same.
- skd_fnc_expectError [text, code] for an error the mod should raise; any other script error on any machine fails the check.
- skd_fnc_evidence [label, value] attaches a value to the result.
Every check starts from a reset session, so it needs no cleanup of what it created. Run it with run_start, read the outcome, fix, and run it again: the file is read fresh each time. The project's docs folder has the whole check API.
```
