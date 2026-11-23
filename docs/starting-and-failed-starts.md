---
type: Documentation
title: Starting a session, and when it fails
description: What session start checks before any game process starts and the message for every gap, how to pin a mod's version so a Workshop update stops the start, what makes a started session fail - Steam, a machine that exits, a client that hangs, the game's own files - where the reason of a failed start is kept, and what happens to a client that code sends to another server.
status: draft
---

# Starting a session, and when it fails

`shakedown session start --setup <name>` (and `run --setup` with no session running) either refuses a start before any game process runs, or starts the processes and ends `ready` or `failed`. A refused and a failed start both exit 4 and say why; neither leaves a game process running.

## Refused: the environment is checked first

Before it starts anything, the start checks the whole environment and names every gap it finds in one message, so you fix them all before the next try:

| Checked | The gap reads |
|---|---|
| The game's `arma3_x64.exe`, and for a dedicated setup `arma3server_x64.exe`, in the `game` folder | `the game executable C:\...\arma3server_x64.exe (check `game` in shakedown.toml)` |
| A running `steam.exe` of your Windows user | `a running Steam client of this user (start Steam and log in)` |
| An account logged in to that Steam: Steam's login file names an account and Steam's `ActiveUser` value is not 0 | `an account logged in to Steam: Steam runs, but no account is logged in (log in to Steam)` |
| 2 GB free on the drive of the project's `.shakedown` folder | `2 GB free on drive C:, which holds C:\project\.shakedown; it has 1.40 GB free (free space on it)` |
| Every mod folder the setup names | `the folder C:\mods\gone of mod gone` |
| The framework's addon, `@shakedown\addons\shakedown_main.pbo` and `@shakedown\shakedown_x64.dll`, next to `shakedown.exe` | `the framework's addon file C:\...\@shakedown\shakedown_x64.dll (install Shakedown again)` |
| For a dedicated setup, the five game ports from `base_port` free, read from the system's UDP table without binding them | `free ports 2462 to 2466 for the server; 2463 held by another program (stop it, or set base_port in shakedown.toml)` |
| For a dedicated setup, the link `<game>\@shakedown`, made when missing | `the framework's link C:\...\Arma 3\@shakedown in the game folder: a folder of that name that is not a link is there (move it away)` |

A start with a mod folder missing and a port taken:

```
$ shakedown session start --setup dedicated
The session cannot start; missing: the folder C:\mods\gone of mod gone; free ports 2462 to 2466 for the server; 2463 held by another program (stop it, or set base_port in shakedown.toml). Provide what is missing and start the session again.
```

The `session_start` tool returns the same message with the error code `precondition-failed` ([AI agents and the MCP tools](agent-tools.md)). A refused start still starts the runner, which stays up about 60 s with no session; a retry within that minute meets it, which is expected.

A second start while a session runs is refused too, naming it ([One session at a time](sessions-and-the-runner.md#one-session-at-a-time)).

## A mod in another version

Workshop mods update themselves, and a check that passed against one version of CBA or ACE says nothing about the next. To stop a start when a mod is not the version you checked against, name that version in the project file, beside the mod's `patch` ([The project file](project-file.md#pinning-a-version)):

```toml
[mods.ace]
path = 'C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\463939057'
patch = "ace_main"
version = "3.21.2.113"
```

`version` is the `versionStr` of the `patch` class, the exact string the result file records under `session.machines[].mods` for that mod, so the way to find it is to run once without the key and copy it from there. Once every machine is up, the start asks each for the versions it loaded; a machine that loaded another version, or does not load the class at all, fails the start before any check runs, and every process is stopped:

```
$ shakedown session start --setup sp-mymod
Session s-20261119-221502 failed: a mod was loaded in another version than the project names: mod mymod on p1 is version 0.0.6, and shakedown.toml wants version 9.9.9. Install the version wanted, or set mods.<name>.version in shakedown.toml to the version found. Every process it started was stopped. The last lines of p1's RPT:
...
```

A machine that does not load the class reads `mod mymod on p1 is in no version (its CfgPatches class mymod_main is not loaded)`. Shakedown does not download or pin a Workshop version: it only refuses a version you did not name. Without the key any version is taken, and the result file still records the one each machine loaded.

## Failed: the start began and did not finish

### Steam is not running or not logged in

A game process that exits with code 53 during the start, which the game does when it cannot reach a logged-in Steam client, fails the start with:

```
Steam is not running or not logged in (p1 exited with code 53)
```

Make sure Steam is logged in, not only running, and is the same Windows user's.

### A machine exits

A process that exits during the start fails it at once. The reason names the machine and the state it was in, followed by its last 20 RPT lines, and every process the start launched is stopped.

### The game cannot read one of its own files

A start can also fail on the machine itself: the game logs `Error 20 reading file` on one of its own base-game PBOs, such as `...\expansion\addons\rocks_f_exp.pbo`, and ends with `Application terminated intentionally` a few seconds after it starts. The failed start's reason holds the last RPT lines that name the file. It is a fault of the machine, not of the framework or the mod, and a second start works.

### A client that hangs

A start has a ceiling, 180 s from the first launch unless the setup's `start_timeout` says otherwise. A process that runs but never gets anywhere, hung or suspended, fails it at the ceiling: the reason names the machine that got least far, a machine still `launched`, whose extension never connected, before one that did, and lists every machine that was not `ready` with its state, followed by that machine's last 20 RPT lines, or, for a process that wrote none, such as one suspended at its launch, a line saying it wrote no RPT line. Every process the start launched is stopped.

```
$ shakedown session start --setup hung
Session s-20261119-223010 failed: the session was not ready within the start ceiling of 90 s: p2 was launched. Every process it started was stopped. p2 wrote no RPT line.
```

There is no earlier ceiling per machine: the time a client takes from its launch to its extension's connect is too close to the start ceiling for one to help. A start that hangs costs its whole `start_timeout`; a setup that starts quickly can set a shorter one, with room for the slowest start you have seen.

### What a failed start prints

With `--json`, `session start` prints the session with state `failed` and its `reason`: `message`, `machine` and `rpt_tail`, the machine's last 20 RPT lines, redacted. That output often passes 4 KB, so a program that runs `session start` in the background reads its stdout and stderr while it runs: the command writes them as it ends and cannot exit before they are read.

## The reason stays in status and in session.json

`status` shows a failed start, state `failed` with its reason, until the next `session start` or `session stop`; `session stop` clears it and exits 0, as nothing of it runs. A program that started the session in the background, such as an AI agent through `session_start`, follows the start with `status` alone.

The session folder keeps the reason after that: its `session.json` holds the same `reason` as `session start` printed, redacted, with the `message`, the `machine` and the `rpt_tail`:

```json
{
  "id": "s-20261119-223010",
  "state": "failed",
  "reason": {
    "message": "the session was not ready within the start ceiling of 90 s: p2 was launched",
    "machine": "p2",
    "rpt_tail": ["..."]
  },
  "stopped_at": "2026-11-19T22:31:52.410Z"
}
```

So a start that failed because the game could not read one of its own files can be told from one that failed on a mod even once the runner has exited: the RPT lines say which file. A session stopped from `ready` writes `reason: null` ([Sessions and the runner](sessions-and-the-runner.md#the-runner-dies)).

## A client sent to another server

Every server the framework starts is the session's own: `loopback = 1`, BattlEye off, named `shakedown <session id>`, and every client is told to connect to it alone. The one way left to reach another server is code a check sends: `connectToServer` acts only outside a mission, so such code first takes the client out of its mission, as the multiplayer pause menu's Abort does. A client that leaves its mission gets 90 s without a heartbeat to start its next one instead of the usual 10 s; when its agent starts in a mission whose server has another name, the runner ends that client's process at once. The check ends `fail` / `check-error`, as the author's mistake, and the reason reads:

```
p2 joined the server "other", which is not the session's own ("shakedown s-20261119-224400"), so the runner ended its process: a client may join only the session's own server. The session is broken: restart its game with `shakedown reload`, or stop it with `shakedown session stop`.
```

The session is broken with that reason; in the result file the machine is `lost` with reason `foreign-server`. `shakedown reload` restarts the game and brings every machine back ([Outcomes and a broken session](outcomes-and-broken-sessions.md)).
