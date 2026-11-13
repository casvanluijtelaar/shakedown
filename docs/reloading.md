---
type: Documentation
title: Reloading and the mission restart
description: How reload classifies a changed file and applies its level, why every change of mod code is a game restart made in one step with the mod's rebuild, how to set a mod up for it, why a reload during a run is refused, how to rerun the checks a finding names, what reloads inside the mission and what partial means, how a check asks for a mission restart with @reset mission, how a broken session comes back, and what the reload result holds.
status: draft
---

# Reloading and the mission restart

A session is kept up across many runs. Two things bring new code or a fresh mission into it without `session stop` and `session start`: `shakedown reload` for a changed file, and `// @reset mission` in a check's header for a check that needs a freshly started mission.

## The levels

Every changed file is classified, and the highest level any file needs is the level applied (`RLD-02`):

| Change | Level | Reason the reload gives |
|---|---|---|
| A check file | `none` | Checks are read fresh for every execution. |
| Any `.sqf` of a watched mod, whatever it defines | `game-restart` | The game holds every mod file it has read and reads a changed one only in a new process, so the game restarts. |
| A new or deleted file | `game-restart` | File patching finds only files in the built PBO, so the mod must be rebuilt before this reload (or, when the mod has a `rebuild`, the game restarts after it). |
| `config.cpp` and any other file (`.hpp`, `.h`, `.inc`, `.ext`, string tables, models, textures, sounds) | `game-restart` | Config is read at game start; the others are not reloaded by file patching. |

When the mod has a `rebuild`, each `game-restart` reason ends with "The mod's rebuild runs while no game process runs."

**Why every change of mod code restarts the game.** Arma 3 opens each loose mod file it reads shared for reading only. While a game process holds the file, nothing can write or replace it, and every machine keeps reading the content it holds, across a mission restart, until it lets go of the file at a time that nothing the framework does controls (spike `SPK-08`, [research](../research/27-spikes-reload.md)). Only a stopped process is sure to let go of a file, and only a new one is sure to read the changed file. So the reload does it in one step: it stops every game process, makes the change while none runs, and starts them again.

## Setting a mod up for reload

The game must load a build of your files, not your files themselves, so the file you edit is never the one the game holds. Name three keys of the mod in `shakedown.toml` ([The project file](project-file.md)):

```toml
[mods.agh]
path = 'C:\dev\arma-grappling-hook\.hemttout\build'   # the build the game loads
patch = "agh_main"
source = 'C:\dev\arma-grappling-hook\addons'           # your files, which reload watches
prefix = 'z\agh\addons'
rebuild = 'cd /d C:\dev\arma-grappling-hook && hemtt build'
```

- `source` is the folder of your addon folders. `reload` compares every file under it with the snapshot taken when the session last became ready.
- `rebuild` is your own command that makes the build from your files, such as `hemtt build`. The reload runs it with `cmd /C` in the project folder after every game process has ended and before any starts again, so it can overwrite files the game held. It must end with exit code 0 within 120 s; its output goes to `rebuild-<mod>.log` in the session folder. It is read fresh at each reload, so adding it needs no new session.
- Do not load a mod through `hemtt dev` or `hemtt launch` links to watch it: those link the game folder back to your own files, which the game then holds, and your editor cannot save them while a session runs.

A mod with no `rebuild` is still reloaded: the processes stop and start, and they read whatever build is there. The framework builds nothing itself.

A file outside every watched folder and the tests folder is refused with exit code 3, naming the watched folders.

## Running a reload

```
shakedown reload                       # what changed under the watched folders since the session was last ready
shakedown reload checks\crate\open.sqf # the files named, relative to the current folder
shakedown reload --json                # the Reload shape below
```

The changed files are found by content: a file with new content, a new file and a deleted file. A file named on the command line is classified whether it changed or not, which is how a game restart is asked for without a change: name a `config.cpp`.

- **`none`** is applied at once: nothing in the game changes, and the next run reads the checks from disk.
- **`game-restart`** stops every process of the session, runs each setup mod's `rebuild` in the order of the setup's mods, and starts the processes again in the same session folder, each machine a new start with its start number one higher and its own profile folder (`machines\<name>-<start>`). Every machine captures a new baseline, and the runner takes a new snapshot of the watched files. On the test machine a dedicated session with two clients took 80 to 83 s, 82 s on average, over ten reloads with a rebuild that copies one file, as long as starting the session (`NFR-PF-02` sets the target at 90 s plus the rebuild); a singleplayer session took about 45 s.

A reload or a `reset` is refused while a run, another reload or a reset is in progress, exit code 4 (the tools: `session-busy`), naming what is in progress: for a reload, its id and level; for a run, its id, its progress and `shakedown run stop`. The run goes on untouched, and the change waits in your files until you ask again:

```
$ shakedown reload
Session s-20261111-194841 is running run r-20261111-194939-6 (0 of 1 checks done, now at slow), so it cannot reload now. Wait for the run to end, or stop it with `shakedown run stop`.
$ shakedown reset
Session s-20261111-194841 is busy with reload rl-20261111-195102-7 (level game-restart), so it cannot be reset now. Wait for it to be ready; `shakedown status` shows it.
```

The `reload` tool returns when the reload is done, or after 50 s with state `applying`; `shakedown status` then shows it as `last_reload`.

The result, as `--json` prints it and the `reload` tool returns it:

```json
{
  "id": "rl-20261109-140312-4",
  "state": "done",
  "level": "game-restart",
  "files": [{"path": "C:\\dev\\arma-grappling-hook\\addons\\main\\functions\\fnc_fire.sqf", "level": "game-restart", "reason": "The game holds every mod file it has read and reads a changed one only in a new process, so the game restarts. The mod's rebuild runs while no game process runs."}],
  "machines": ["server", "p1", "p2"],
  "reason": null,
  "started_at": "2026-11-09T14:03:12.104Z",
  "duration_s": 81.8
}
```

`machines` are the machines the reload reached. `duration_s` is the time from the reload order to every machine ready on the new code, the rebuild included.

## Rerunning the checks a finding names

Tag every check that proves a finding with its id, `// @tags m07-1`, and after the fix reload and rerun them by the tag:

```
shakedown reload
shakedown run --tag m07-1
```

With `@skd_fixture` alone, the time from the reload to the last of three short checks' outcomes on the test machine was about 29 s in singleplayer (the reload about 24 s) and about 50 s on a dedicated session with two clients (the reload about 44 s). A setup that loads more mods, such as CBA_A3, takes longer to start, and so to reload.

## What reloads inside the mission, and `partial`

No change of mod code reloads inside the running mission, neither by a recompile nor by a mission restart: the game holds the files it has read (`DEF-21`). A check reloads inside the mission without any reload at all, since it is read fresh for every execution. Everything else is the game restart above.

Every check execution in the result file has a `reload` field. It is `partial` only after a reload of mod code inside the mission, where handlers the mod registered by value keep the old code until a mission restart (`RLD-05`). Since no such reload exists, every execution records `full`: the checks ran on the code the game started with.

## A reload that does not complete

When a machine does not come back during a game restart, because its process exits or the start ceiling passes, the reload ends `failed`, exit code 4, with `reason` naming the level it was applying, the machine and why, and the session is `broken` with the same reason. A `rebuild` that exits with another code or passes its ceiling still lets every machine come back, on the files as they are, and then fails the reload the same way, naming the mod and its log, with the lead as the machine. A run asked on a broken session is refused, exit code 4 (the `run` tool: `session-broken`), naming `shakedown reload`.

`shakedown reload` on a broken session always restarts the game, whatever the files need, running the rebuilds again: it is the way back for a session broken by a lost machine, a failed reset or a failed reload. The other is `shakedown session stop`.

## A check that needs a fresh mission

The reset in place before every check restores what the framework can name ([The check API by example](check-api.md#the-reset-between-checks)). A check that depends on state it cannot name, such as a public variable an earlier check set or a mod's setting, asks for a mission restart instead:

```sqf
// @check US-011 A fresh mission clears what an earlier check set
// @reset mission
["every machine starts with no such variable", {
    ["all", { isNil "my_flag" }, "==", true, "my_flag is nil"] call skd_fnc_expectEach;
}] call skd_fnc_step;
```

- **Singleplayer**: `p1` ends the mission and the game loads the session mission again from its `-autotest` list, in about 2 s. `p1` keeps its process and its start number.
- **Dedicated**: the game clients and the headless clients are stopped, the server kicks any user it still lists and restarts the mission, and fresh client and headless client processes join it, each with its start number one higher. The server keeps its process. On the test machine this took about 48 s with two clients and a headless client, most of it the clients' own start: about as long as starting the session.

A mission restart does not reload a changed mod file: the processes it keeps still hold the files they read. Every machine captures a new baseline at the new mission start. The result file lists every machine start that took part in the run, so a run with a `@reset mission` check shows the old and the new start of each client.

An order sent to a machine while its mission restarts or its process is replaced, such as an `exec` from another window, waits until the machine is ready again, for at most the setup's start ceiling, then runs with its whole ceiling and is answered once.

When the mission restart does not bring a machine back, that check ends `invalid` / `reset-failed`, naming the machine, and the session is `broken`: the checks after it end `invalid` / `session-broken`, and the run exits 2. `shakedown reload` brings it back.
