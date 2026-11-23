---
type: Documentation
title: Sessions and the runner
description: What a session and the runner are, how to start, watch and stop a session, why only one runs at a time, how a session outlives the command or AI agent session that started it, what happens when the runner dies or cannot be reached, and how to end a runner that does not answer.
status: draft
---

# Sessions and the runner

A **session** is a running game in one setup: one singleplayer game, or a dedicated server with its game clients and headless clients, all on your machine. One session serves any number of runs, checks and commands without restarting the game; only a reload at level `game-restart`, a mission restart and a client joining start a process.

The **runner** is the background process that owns the session. The first command that needs it starts it; it outlives that command, and exits by itself 60 s after its last session stopped with no client connected. Every command except `list`, `guide`, `version` and `mcp` talks to it.

## Start, watch, stop

```
> shakedown session start --setup dedicated
session s-20261106-092156 ready (setup dedicated)
> shakedown status
runner: running (process 9184)
session: s-20261106-092156 of setup dedicated, ready
  server  server    ready      start 1 process 3116 profile C:\mods\mymod\.shakedown\sessions\s-20261106-092156\machines\server-1
          health: 47.1 frames a second, answers in 0.009 s, mission time 52.0 s, frame 2410
  p1      client    ready      start 1 process 3604 profile C:\mods\mymod\.shakedown\sessions\s-20261106-092156\machines\p1-1
          health: 58.3 frames a second, answers in 0.004 s, mission time 40.1 s, frame 2337
  p2      client    ready      start 1 process 7484 profile C:\mods\mymod\.shakedown\sessions\s-20261106-092156\machines\p2-1
          health: 57.9 frames a second, answers in 0.004 s, mission time 40.0 s, frame 2318
> shakedown session stop
session s-20261106-092156 stopped
```

- **`session start --setup <name>`** checks the environment, starts the game processes and returns when the session is `ready` or `failed`. It takes the game's own start time: 40 to 60 s in singleplayer, about 45 s for a dedicated server with two clients and a headless client. More mods take longer. A start that is refused or fails exits 4 and leaves no game process running ([Starting a session, and when it fails](starting-and-failed-starts.md)).
- **`status`** shows the runner, the session and each of its machines with its state, start number, process id, profile folder and health, the run in progress (`run: <id> <done> of <total> done, now <check>`) and, with `--json`, the last reload. It starts nothing and works at any time: with no runner, while a session starts in another command, and while a run goes on. It lists each machine from the moment its process is launched.
- **`session stop`** stops every process of the session and returns when none is left: on a dedicated session the clients and headless clients first, then the server, and it deletes the mission copy it put in the game's `MPMissions` folder. During a run it stops the run first. On a failed start that `status` still shows, it clears it and exits 0.

A machine's state is `launched` (its process runs and its extension has not connected), `connected` (the extension connected, the mission has not reported in), `ready` (its in-game agent answers and, on a dedicated session, the server sees it at client state 10), `stopped` (it ended when asked, such as a client a check made leave) or `lost` (it exited unasked or stopped answering). A session is `starting`, `ready`, `running` (a run is on), `reloading` (a reload or a reset is on), `broken` (a machine was lost, or a reset or reload failed), `failed` (its start failed), `stopping` or `stopped`.

The session's folder, `.shakedown\sessions\<session id>\` in the project, holds the generated mission, a dedicated session's `server.cfg`, each machine's profile folder `machines\<machine>-<start>` with its RPT and any crash dump, `audit.jsonl`, the log of each mod's rebuild, and `session.json` once it ends. Profile folders are kept after the session stops.

## One session at a time

One session runs on the machine at a time: the machine has one Steam session and one set of game ports. A second start is refused, exit 4:

```
A session runs already: s-20261106-092156 of setup dedicated. Stop it with `shakedown session stop`, or run against it without --setup.
```

## The command or the AI agent session ends

A session belongs to the runner, not to the command or the AI agent session that started it. `shakedown session start` returns once the session is `ready`, and the session keeps running. Any later command, in this terminal or another, works against it: `shakedown run` runs in it, and its result file names its session id.

The same holds for an AI agent. When a Claude Code session ends, Claude Code stops its `shakedown mcp` server; the runner and the game run on. A new Claude Code session's `status` tool shows the same session `ready`.

This holds however the host ends its MCP server: by closing its input, by closing a job object that holds it with kill-on-close, or by ending each process of its tree. The client starts the runner outside its own job object and process tree: through a short-lived relay process that leaves the client's job, or, when the client's job forbids that, through WMI. `runner.log` has a line for each start naming the way it went:

```
2026-11-09T09:59:32.863Z client 3248 started the runner by breakaway
2026-11-09T09:59:33.199Z relay 13328: started the runner by wmi
2026-11-09T09:59:33.598Z runner 12380 started on port 59670; its process was created at 2026-11-09T09:59:33.212Z
```

A session left running holds the game until it is stopped: end it with `shakedown session stop` (or the `session_stop` tool) when you are done.

## The runner dies

Every game process of a session is held by a job object that the runner owns with kill-on-close. When the runner ends, however it ends, a crash or `taskkill /F`, Windows ends every game process of its session at once, about 0.1 s after. Nothing waits for the game to answer.

What a job object cannot clean up, the next runner does. The runner keeps a run record, `%LOCALAPPDATA%\Shakedown\record.json`, of every session it started, each game process (its id and start time) and each file or folder it made outside the project, which is the mission copy in the game's `MPMissions` folder of a dedicated session. Every runner, when it starts and before it serves any command, ends each recorded process that still runs with its recorded start time, deletes each recorded file and folder, marks each recorded session stopped, and empties the record. `runner.log` has a line for each:

```
recovery: s-20261109-095933: deleted C:\Program Files (x86)\Steam\steamapps\common\Arma 3\MPMissions\shakedown_s-20261109-095933.VR
recovery: s-20261109-095933: stopped, reason runner-died
```

Between the death and the next start, `shakedown status` shows the runner `not-running` and no session, and starts nothing. The next command that needs a runner, such as `session start`, starts one, which recovers first.

Every session folder gets a `session.json` when the session ends:

```json
{
  "id": "s-20261109-095933",
  "state": "stopped",
  "reason": "runner-died",
  "stopped_at": "2026-11-09T10:00:43.237Z"
}
```

`reason` is `runner-died` for a session the recovery ended, and `null` for one stopped with `session stop` while it was `ready`. For a start that failed, `state` is `failed` and `reason` is the reason `session start` printed and `status` showed, with its `message`, `machine` and `rpt_tail`, redacted; a session that broke and was then stopped keeps its reason the same way ([The reason stays in session.json](starting-and-failed-starts.md#the-reason-stays-in-status-and-in-sessionjson)).

## A command cannot reach the runner

A runner can run and still not answer: suspended in a debugger, hung, or a runner of another Shakedown version left from before an install. A command then does not start a second runner beside it, which would lose the session. It waits up to 5 s for the runner's answer and then:

- `shakedown status` exits 0 and shows the runner `unreachable` with its process id, and no session:

  ```
  $ shakedown status --json
  {"runner": "unreachable", "runner_pid": 12380, "version": "0.2.0", "session": null, "run": null, "last_reload": null}
  ```

- every other command exits 4 with what to do:

  ```
  $ shakedown run
  The runner (process 12380) does not answer. Ending that process ends its session's game processes with it; the next command then starts a fresh runner.
  ```

  A runner of another version answers, and the message names both versions: `The runner (process 12380) runs version 0.1.8, and this command is version 0.2.0. ...`

- every MCP tool but `status`, `list_checks` and `guide` returns the error code `runner-unreachable` with the same message, and `status` returns `runner` `unreachable` ([AI agents and the MCP tools](agent-tools.md)).

The way out is one of two:

- **It is only slow or paused.** Resume it (end the debugger) and wait. Once it answers again, `status` shows the same session as before, every machine at the start number it had.
- **End it.** End the runner by the process id `status` gave:

  ```
  taskkill /PID 12380 /F
  ```

  Its game processes end with it, and the next command starts a fresh runner, which clears what the old one left. End it only by that id: the game processes are ended by the job object, not by name.

Never delete `%LOCALAPPDATA%\Shakedown\runner.json` while a runner holds a session: the next command would start a second runner, whose start-up recovery ends the first session's game.
