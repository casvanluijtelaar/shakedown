---
type: Specification
title: MCP tools
description: The contract for the shakedown MCP server - how it is started, the rules every tool follows, and each tool with its input, its structured output and its errors.
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# MCP tools

`shakedown mcp` is an MCP server over stdio (`AGT-01`). Claude Code starts it from the project's `.mcp.json` (`INT-CC-01`); the server is named `shakedown`, so Claude Code shows its tools as `mcp__shakedown__<tool>`. The command line's `--json` output for a command is the structured content of its twin tool ([cli.md](cli.md)).

## Rules for every tool

- Every tool has an `inputSchema` and an `outputSchema` (JSON Schema 2020-12) and returns `structuredContent` and the same JSON as one text block (`AGT-02`).
- A tool that changes nothing in the game is marked `readOnlyHint: true`: `status`, `list_checks`, `run_status`, `run_result`, `inspect`, `logs`, `errors`, `guide`. `exec` is marked `destructiveHint: true`.
- No call lasts longer than 60 s (`AGT-03`). A tool that waits takes a `ceiling` of at most 50 s and sends `notifications/progress` at least every 5 s when the request carries a progress token.
- A cancelled call stops the work it started in the game (`AGT-04`).
- A result over 20,000 characters is written to a file in the project's `.shakedown/results/` and the result is `{"result_path": path}` instead; every `outputSchema` allows that alternative (`AGT-07`). `logs` keeps its lines under the limit itself and names the file with all of them in `path`.
- A check that fails is a successful call that returns its outcome. `isError: true` is returned only for what the AI agent can fix. Its text is `<code>: <what is wrong>. <what to do next>.`, with one of these codes:

  | Code | When |
  |---|---|
  | `bad-arguments` | An input does not match the schema, or values conflict. |
  | `config-error` | `shakedown.toml` is malformed. |
  | `unknown-check` | A check id or tag selects nothing. |
  | `unknown-machine` | The session has no such machine. |
  | `unknown-setup` | The project has no such setup. |
  | `unknown-run` | No run has that id. |
  | `compile-error` | The SQF does not compile on the target; the engine's message follows. |
  | `no-session` | The tool needs a session and none runs. |
  | `session-exists` | A session already runs. |
  | `session-busy` | The session is running a run, a reload or a reset. |
  | `session-broken` | The session is `broken`; the reason follows. |
  | `precondition-failed` | A precondition of `SES-03` failed; what is missing follows. |
  | `runner-unreachable` | The runner `runner.json` names runs but does not answer, or runs another version; its process id, and its version when it answered, follow (`P-01`). |

## Shared shapes

- **Machine**: `{"name": "p1", "kind": "client", "start": 1, "state": "ready", "pid": 10432, "process_started_at": "<time>", "profile": "<path>", "last_answer_at": "<time>" or null, "health": {"flag": null, "since": null, "heartbeat": Clocks or null}}` (`INS-04`). `kind` is `server`, `client` or `headless`; `state` as the machine status machine of `01-domain-model.md`. `pid` is the operating-system process id of the machine's game process and `process_started_at` that process's start time, which together name it even after the id is reused; `profile` is the absolute path of its profile folder, which holds its RPT and any crash dump. `health.flag` is `null`, `mission-time` or `frame` for the clock that stopped (`HLT-02`), `since` the time it was raised, and `heartbeat` the clocks of the last heartbeat. A machine is listed from the moment its process is launched, in every session state.
- **Session**: `{"id", "setup", "mode", "state", "started_at", "folder", "machines": [Machine], "reason": {"message", "machine", "rpt_tail": [string]} or null}`; `state` as the session status machine. `reason` is set when the state is `failed` or `broken`: what happened, the machine it concerns, and that machine's last 20 RPT lines, redacted (`SES-04`, `P-07`, `P-09`).
- **Outcome line**: `{"check_id", "repeat", "outcome", "reason": {"kind", "message"} | null, "duration_s"}`.
- **Value**: an SQF value encoded as `DM-13` says.
- **Clocks**: `{"utc": Time, "tick", "time", "frame"}`, the clocks of `OBS-05`, the shape of the same name in [result.schema.json](result.schema.json).
- **Time**: an ISO 8601 UTC string with milliseconds, such as `"2026-11-04T21:37:06.123Z"`.

## Tools

| Tool | Input | Output |
|---|---|---|
| `status` | `{}` | `{"runner": "running" or "not-running" or "unreachable", "runner_pid": integer or null, "version", "session": Session or null, "run": {"id", "state", "done", "total", "current_check"} or null, "last_reload": Reload or null}`. `unreachable` comes with the runner's `runner_pid` and `session` `null` (`P-01`). A start that failed is the `session`, with state `failed` and its `reason`, until the next `session_start` or `session_stop` (`INS-02`). Works with no runner and no game (`AGT-05`). |
| `session_start` | `{"setup": string}` | Session. Returns when the session is `ready` or `failed`, or after 50 s with state `starting`; follow it with `status`. |
| `session_stop` | `{}` | Session, with state `stopped`. On a failed start that `status` still shows, that session with state `failed`, which it clears. |
| `list_checks` | `{"ids"?: [string], "tags"?: [string]}` | `{"checks": [{"id", "name", "tags", "setups", "timeout_s", "reset", "error"}]}`, each selected check once. `setups` is `null` for a check that runs in every setup. `error` is `null`, or, for a check whose header is wrong, what is wrong with it, naming the line and the key; `name`, `setups`, `timeout_s` and `reset` are then `null` and `tags` empty. Works with no runner. |
| `run_start` | `{"ids"?: [string], "tags"?: [string], "setup"?: string}` | `{"run_id", "state": "running" or "starting-session", "total"}`. `ids` run in the order given, an id given more than once once per time it is given (`RUN-03`); `total` counts the executions. Returns at once. With `setup` and no session, the run starts one and stops it at its end (`RUN-04`). |
| `run_status` | `{"run_id": string, "wait_s"?: number}` | `{"run_id", "state": "starting-session" or "running" or "finished" or "stopped", "done", "total", "current_check", "outcomes": [Outcome line]}`. With `wait_s` (at most 50) it returns as soon as a check ends or the run ends, or at `wait_s`. |
| `run_result` | `{"run_id": string}` | `{"run_id", "state", "summary": {"total", "pass", "fail", "invalid", "not_run"}, "outcomes": [Outcome line with "failed_step" and "evidence_path"], "result_file": path}`. The full result is the file. `unknown-run` while the run has not ended says to call `run_status`. |
| `run_stop` | `{"run_id": string}` | As `run_status`, with state `stopped`. |
| `reset` | `{}` | `{"state": "done" or "failed", "duration_s", "mismatches": [string]}`. |
| `reload` | `{"files"?: [string]}` | Reload: `{"id", "state": "applying" or "done" or "failed", "level": "none" or "game-restart", "files": [{"path", "level", "reason"}], "machines": [machine name], "reason": {"step", "machine", "message"} or null, "started_at", "duration_s"}` (`RLD-03`). The input's `files` are paths relative to the project, under a watched `source` or the checks folder; the output's are absolute. `machines` are the machines the reload reached, once it is done; `reason` says, for a reload that `failed`, the level it was applying, the machine that did not come back and why, and the session is then `broken`. On a `broken` session the level applied is `game-restart` (`RLD-01`). Returns when done, or after 50 s with state `applying`; `status` then shows it as `last_reload`. |
| `exec` | `{"machine": string, "code": string, "args"?: array, "ceiling"?: number}` | `{"results": {machine name: {"state": "done" or "ceiling" or "lost", "value": Value, "duration_s"}}, "errors": [ScriptError]}`. `machine` may be a group name. Script errors the code caused are returned, not raised, with `check_id` `null` outside a check. `ceiling` defaults to 10. `lost` is a machine that did not answer within the ceiling plus the grace. |
| `wait_until` | `{"machine", "condition": string, "description"?: string, "args"?: array, "ceiling"?: number}` | `{"met": boolean, "waited_s", "error": string or null}`. `error` says why a wait ended early with `met` `false`: the script error its condition raised, or the type it returned instead of a Boolean (`OBS-01`). `machine` is one machine, not a group; `ceiling` defaults to 10. |
| `expect` | `{"machine", "value": string, "op", "expected": Value, "args"?: array, "ceiling"?: number}` | `{"passed": boolean, "last_value": Value, "waited_s"}`. Operands that do not fit `op` are `bad-arguments`, naming what the operator needs. |
| `inspect` | `{"machine", "near"?: netId, "radius"?: number}`, `radius` default 50 | `{"machine": Machine, "mission", "mission_start", "time", "tick", "fps", "frame", "run_id", "check_id", "step_path", "player": {"class", "position", "direction", "damage", "alive", "vehicle", "animation"} or null, "nearby": [{"class", "net_id", "position", "distance"}], "scripts": [{"name", "file", "line"}], "errors": [ScriptError]}` (`INS-01`). A machine that does not answer within 10 s plus the grace returns `machine` with state `lost` and every other field `null`. |
| `logs` | `{"machine", "run_id"?, "check_id"?, "grep"?: regex, "last"?: integer}` | `{"lines": [string], "truncated": boolean, "path": path or null}`, redacted (`LOG-03`). |
| `errors` | `{"machine"?, "run_id"?, "check_id"?}` | `{"errors": [ScriptError]}`: the running session's script errors, inside checks and outside, the oldest first (`ERR-05`). Needs a session. |
| `guide` | `{}` | `{"text": string}`: how to start a session, run checks, read a result and write a check, in under 8,000 characters (`AGT-06`). |

`ScriptError` is the shape of the same name in [result.schema.json](result.schema.json).
