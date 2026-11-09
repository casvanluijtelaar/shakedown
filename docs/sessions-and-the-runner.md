---
type: Documentation
title: When a client or the runner ends
description: What happens to a session when the command or the AI agent session that started it ends, when the runner itself dies, and when a command cannot reach the runner, how to end a runner that cannot be reached, and how the runner keeps control to the local user and records every piece of code sent to the game.
status: draft
---

# When a client or the runner ends

A session belongs to the runner, the background process that the first command needing it starts ([Installing Shakedown](install.md)), not to the command or the AI agent session that started it (`SES-06`). This page says what happens to a session when each of them ends, and what the runner lets in.

## The command or the AI agent session ends

`shakedown session start` returns once the session is `ready`, and the session keeps running. Any later command, in this terminal or another, works against it: `shakedown run` runs in it, and its result file names its session id.

The same holds for an AI agent. When a Claude Code session ends, Claude Code stops its `shakedown mcp` server; the runner and the game run on. A new Claude Code session's `status` tool shows the same session `ready`:

```
$ shakedown status
runner: running (process 12380)
session: s-20261109-095933 of setup dedicated, ready
...
```

This holds however the host ends its MCP server: by closing its input, by closing a job object that holds it with kill-on-close, or by ending each process of its tree. The client starts the runner outside its own job object and process tree (`SES-08`, `INT-WIN-02`): through a short-lived relay process that leaves the client's job, or, when the client's job forbids that, through WMI. `runner.log` has a line for each start naming the way it went:

```
2026-11-09T09:59:32.863Z client 3248 started the runner by breakaway
2026-11-09T09:59:33.199Z relay 13328: started the runner by wmi
2026-11-09T09:59:33.598Z runner 12380 started on port 59670; its process was created at 2026-11-09T09:59:33.212Z
```

A session left running holds the game until it is stopped: end it with `shakedown session stop` (or the `session_stop` tool) when you are done. The runner exits by itself 60 s after its last session stopped with no client connected.

## The runner dies

Every game process of a session is held by a job object that the runner owns with kill-on-close. When the runner ends, however it ends, a crash or `taskkill /F`, Windows ends every game process of its session at once (`STP-01`); on the test machine they were gone 0.1 s after the kill. Nothing waits for the game to answer.

What a job object cannot clean up, the next runner does. The runner keeps a run record, `%LOCALAPPDATA%\Shakedown\record.json`, of every session it started, each game process (its id and start time) and each file or folder it made outside the project, which is the mission copy in the game's `MPMissions` folder of a dedicated session (`STP-03`). Every runner, when it starts and before it serves any command, runs the recovery of `P-12`: it ends each recorded process that still runs with its recorded start time, deletes each recorded file and folder, marks each recorded session stopped, and empties the record. `runner.log` has a line for each:

```
recovery: s-20261109-095933: deleted C:\Program Files (x86)\Steam\steamapps\common\Arma 3\MPMissions\shakedown_s-20261109-095933.VR
recovery: s-20261109-095933: stopped, reason runner-died
```

Between the death and the next start, `shakedown status` shows the runner `not-running` and no session, and starts nothing. The next command that needs a runner, such as `session start`, starts one, which recovers first.

Every session folder, `.shakedown\sessions\<session id>\` in the project, gets a `session.json` when the session ends:

```json
{
  "id": "s-20261109-095933",
  "state": "stopped",
  "reason": "runner-died",
  "stopped_at": "2026-11-09T10:00:43.237Z"
}
```

`reason` is `runner-died` for a session the recovery ended, and `null` for one stopped with `session stop`; `state` is `failed` for a start that failed.

## A command cannot reach the runner

A runner can run and still not answer: suspended in a debugger, hung, or a runner of another Shakedown version left from before an install. A command then does not start a second runner beside it, which would lose the session. It waits up to 5 s for the runner's answer (`P-01`) and then:

- `shakedown status` exits 0 and shows the runner `unreachable` with its process id, and no session:

  ```
  $ shakedown status --json
  {"runner": "unreachable", "runner_pid": 12380, "version": "0.0.6", "session": null, "run": null, "last_reload": null}
  ```

- every other command exits 4 with what to do:

  ```
  $ shakedown run
  The runner (process 12380) does not answer. Ending that process ends its session's game processes with it; the next command then starts a fresh runner.
  ```

  A runner of another version answers, and the message names both versions: `The runner (process 12380) runs version 0.0.5, and this command is version 0.0.6. ...`

- every MCP tool but `status`, `list_checks` and `guide` returns `isError` with the code `runner-unreachable` and the same message, and `status` returns `runner` `unreachable` ([Agent tools](agent-tools.md)).

The way out is one of two:

- **It is only slow or paused.** Resume it (end the debugger) and wait. Once it answers again, `status` shows the same session as before, every machine at the start number it had.
- **End it.** End the runner by the process id `status` gave:

  ```
  taskkill /PID 12380 /F
  ```

  Its game processes end with it, and the next command starts a fresh runner, which clears what the old one left. End it only by that id: the game processes are ended by the job object, not by name.

## Only the local user controls the session

The runner and the extension in each game process talk only over `127.0.0.1` (`SEC-01`, `NFR-SC-01`); `netstat -ano` shows every socket of the runner, and every connection of a game process to it, on `127.0.0.1`.

Every client presents the runner token from `runner.json` on each connection, and every game process's extension presents its session token, which the runner passes to the process in its environment only, never on a command line (`SEC-02`). Neither token appears on any process's command line, in the result file, in a log the framework writes or in any command or tool output (`NFR-SC-03`). Both have a fixed form, `skd_r_` or `skd_s_` followed by 64 lowercase hexadecimal digits (`SEC-06`), so a search for `skd_[rs]_[0-9a-f]{64}` finds a leak; `runner.json` is the one file that holds the runner token. Its access list is protected, so it takes nothing from its folder, and grants the current user alone: no other account or group, the Administrators group included, can read it (`SEC-02`, `INT-WIN-03`).

An extension's hello is also checked against the process at the other end of its connection, read from the system's TCP table: only the process the runner started for that machine of that session is let in, so a program that copied the session token still cannot pose as a game process (`SEC-03`). Anything else is closed without a reply, runs no code, and adds a line to `runner.log` (`NFR-SC-02`):

```
refused client connection from pid 13940: bad-token
refused extension connection from pid 13940: bad-token
refused extension connection from pid 13940: unknown-process
refused client connection from pid 13940: not-hello
```

## Every piece of code sent to the game is recorded

Each session folder holds `audit.jsonl`, one JSON object per line for every `exec` order sent to a machine (`SEC-04`): its `time`, its `source`, its `target` machine and its `code`. The source is `cli` for `shakedown exec`, `mcp` for the `exec` tool, and the check id for a check: the check's body, sent to the lead, and each piece of code the body sends to another machine with `skd_fnc_on`, a wait or a polled assertion. Code the body runs on the lead itself is part of the body's line. An `exec` to a group has one line per machine.

```
{"code":"6 * 7","source":"cli","target":"server","time":"2026-11-09T10:12:04.511Z"}
{"code":"6 * 8","source":"mcp","target":"hc1","time":"2026-11-09T10:12:05.093Z"}
{"code":"private _v = [\"p1\", { 40 + 2 }] call skd_fnc_on; ...","source":"US-052-on-p1","target":"server","time":"2026-11-09T10:11:58.020Z"}
{"code":"40 + 2","source":"US-052-on-p1","target":"p1","time":"2026-11-09T10:11:58.090Z"}
```

The code is redacted like everything else the runner stores, so a Steam ID or the Steam account's names in it read `<steamid>` and `<steamname>` ([Script errors, logs and evidence](script-errors-and-logs.md)). `runner.log` holds process ids, ports, session ids and paths, and is redacted the same way.
