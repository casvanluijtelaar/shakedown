---
type: Documentation
title: Reloading and the mission restart
description: How reload classifies a changed file and applies its level, how a changed .sqf of a mod reaches every machine by a mission restart once the game is made to let go of it, when the reload falls back to a game restart and why, how to set a mod up for it, why a reload during a run is refused, how to rerun the checks a finding names, what partial means, how a check asks for a mission restart with @reset mission, how a broken session comes back, and what the reload result holds.
status: draft
---

# Reloading and the mission restart

A session is kept up across many runs. Two things bring new code or a fresh mission into it without `session stop` and `session start`: `shakedown reload` for a changed file, and `// @reset mission` in a check's header for a check that needs a freshly started mission.

## The levels

Every changed file is classified, and the highest level any file needs is the level applied (`RLD-02`):

| Change | Level | Reason the reload gives |
|---|---|---|
| A check file | `none` | Checks are read fresh for every execution. |
| A file of the library folder | `none` | Every reload reads the library folder again and has every machine compile it. |
| A changed `.sqf` of a watched mod that was there when the session was last ready, whatever it defines | `mission-restart` | The game is made to let go of the file by reading the framework's release files, and a mission restart compiles it again. |
| A new or deleted file | `game-restart` | File patching finds only files in the built PBO, so the mod must be rebuilt before this reload (or, when the mod has a `rebuild`, the game restarts after it). |
| `config.cpp` and any other file (`.hpp`, `.h`, `.inc`, `.ext`, string tables, models, textures, sounds) | `game-restart` | Config is read at game start; the others are not reloaded by file patching. |

When the mod has a `rebuild`, each `game-restart` reason ends with "The mod's rebuild runs while no game process runs.", and each `mission-restart` reason with "The mod's rebuild runs once no game process holds the file." One `config.cpp` among the files makes the whole reload a game restart.

**Why a changed `.sqf` needs a mission restart, and how the game lets go of it.** Arma 3 opens each loose mod file it reads shared for reading only. While a game process holds the file, nothing can write or replace it, and every machine keeps reading the content it holds, across a mission restart (spike `SPK-08`, [research](../research/27-spikes-reload.md)). A game process lets go of the loose files it holds once it has read a few hundred MiB of other loose files through file patching (spike `SPK-19`, [research](../research/31-spike-reload-release.md)). So a reload at level `mission-restart` does this, in one step:

1. Every machine reads the release files: one 8 MiB file reached through 80 directory junctions, `skd_release\l00` to `l79` in the game folder, 640 MiB in all, since the game keeps loose files by path. The runner makes them when the first such reload needs them, which takes up to a minute once, and keeps them. All machines read at once, with a ceiling of 60 s; the read takes 5 to 10 s.
2. The runner opens for writing the file the game reads for each changed file: the same path below the mod's `prefix` in the game folder. If one is still held, every machine reads the release files once more.
3. With every changed file free, it runs each setup mod's `rebuild`.
4. It restarts the mission, as `@reset mission` does (below): in singleplayer `p1` keeps its process and its start number; on a dedicated session the server keeps its process and the clients and headless clients start again.
5. After the mission restart every machine recompiles, with `BIS_fnc_recompile`, each `CfgFunctions` function whose file changed. The Functions Library compiles an addon's `CfgFunctions` function once per game process and copies it into every new mission, so without that a kept process would run the old function. A CBA `PREP` function needs no such step: the session mission turns CBA's function cache off, so CBA compiles it from its file at every mission start. The recompile is repeated after every later mission restart, such as that of a `@reset mission` check, until the next game restart.

**When the game does not let go: a game restart.** If a changed file is still held after the second read, the reload applies `game-restart` instead: it stops every game process, runs the rebuilds and starts them again. That file's entry in the result has level `game-restart` and a reason that names the file and the machine whose process holds it, found through the Windows Restart Manager:

```
"level": "game-restart",
"reason": "C:\\...\\Arma 3\\skdfx_cba\\addons\\main\\functions\\fnc_version.sqf is still held by p1's process after two reads of the framework's release files: the game did not let go of it, so the game restarts."
```

New and deleted files and config always restart the game: file patching finds only files in the built PBO, and config is read at game start.

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
- `rebuild` is your own command that makes the build from your files, such as `hemtt build`, or copies changed files into it. The reload runs it with `cmd /C` in the project folder: at level `game-restart` after every game process has ended and before any starts again, so it can overwrite any file the game held; at level `mission-restart` once no game process holds a changed file, while the game still holds its PBOs and every other file it read. A rebuild that serves both levels writes only the loose files that changed, such as `robocopy` does, or `fc /b <from> <to> >nul || copy /y <from> <to>` for one file; a `hemtt build` that rewrites a held PBO fails a `mission-restart` reload. It must end with exit code 0 within 120 s; its output goes to `rebuild-<mod>.log` in the session folder. It is read fresh at each reload, so adding it needs no new session.
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
- **`mission-restart`** frees the changed files, runs the rebuilds and restarts the mission, as above. On the test machine, with `@skd_fixture_cba`, a singleplayer session took 8 to 10 s from the reload order to `p1` ready on the new code, and the time from saving the file to a short check's outcome was 10 to 12 s. A dedicated session with two clients takes about as long as a game restart, since its clients start again (`NFR-PF-02` sets the targets).
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
  "level": "mission-restart",
  "files": [{"path": "C:\\dev\\arma-grappling-hook\\addons\\climb\\functions\\fn_letgo.sqf", "level": "mission-restart", "reason": "The game is made to let go of the file by reading the framework's release files, and a mission restart compiles it again. The mod's rebuild runs once no game process holds the file."}],
  "machines": ["server", "p1", "p2"],
  "reason": null,
  "started_at": "2026-11-09T14:03:12.104Z",
  "duration_s": 9.7
}
```

`level` is the level applied; each file's `level` is the level it needed, or `game-restart` for a file the game did not let go of. `machines` are the machines the reload reached. `duration_s` is the time from the reload order to every machine ready on the new code, the rebuild included.

## Rerunning the checks a finding names

Tag every check that proves a finding with its id, `// @tags m07-1`, and after the fix reload and rerun them by the tag:

```
shakedown reload
shakedown run --tag m07-1
```

With `@skd_fixture` alone and a changed `CfgFunctions` function, the time from the reload to the last of three short checks' outcomes on the test machine was about 13 s in singleplayer (the reload about 8 s) and about 57 s on a dedicated session with two clients (the reload about 50 s, most of it the clients' new start).

## What reloads inside the mission, and `partial`

No change of mod code reloads inside the running mission: a changed `.sqf` takes a mission restart (`DEF-32`). A check reloads inside the mission without any reload at all, since it is read fresh for every execution.

Every check execution in the result file has a `reload` field. It is `partial` after a reload that had a machine recompile a `CfgFunctions` function, until the next game restart, because code that took the function by value at the mission start, such as an event handler a mod added in its `postInit` with the function as its code, keeps the old code on a machine whose process the mission restart kept (`RLD-05`). Otherwise it is `full`. A check that sees `partial` and fails where it should not is worth rerunning after `shakedown reload` of a `config.cpp`, which restarts the game.

## A reload that does not complete

When a machine does not come back during a mission restart or a game restart, or does not answer the read of the release files, because its process exits or the start ceiling passes, the reload ends `failed`, exit code 4, with `reason` naming the level it was applying, the machine and why, and the session is `broken` with the same reason. A `rebuild` that exits with another code or passes its ceiling still lets every machine come back, on the files as they are, and then fails the reload the same way, naming the mod and its log, with the lead as the machine. A run asked on a broken session is refused, exit code 4 (the `run` tool: `session-broken`), naming `shakedown reload`.

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

A mission restart of `@reset mission` does not reload a changed mod file: the processes it keeps still hold the files they read. `shakedown reload` does, as above. Every machine captures a new baseline at the new mission start. The result file lists every machine start that took part in the run, so a run with a `@reset mission` check shows the old and the new start of each client.

An order sent to a machine while its mission restarts or its process is replaced, such as an `exec` from another window, waits until the machine is ready again, for at most the setup's start ceiling, then runs with its whole ceiling and is answered once.

When the mission restart does not bring a machine back, that check ends `invalid` / `reset-failed`, naming the machine, and the session is `broken`: the checks after it end `invalid` / `session-broken`, and the run exits 2. `shakedown reload` brings it back.
