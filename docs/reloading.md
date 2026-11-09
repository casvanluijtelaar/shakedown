---
type: Documentation
title: Reloading and the mission restart
description: How reload classifies a changed file and which level it applies, what this build applies and what it refuses, how a check asks for a mission restart with @reset mission, how a broken session comes back, and what the reload result holds.
status: draft
---

# Reloading and the mission restart

A session is kept up across many runs. Two things bring new code or a fresh mission into it without stopping it: `shakedown reload` for a changed mod file, and `// @reset mission` in a check's header for a check that needs a freshly started mission.

## The levels

Every changed file is classified, and the highest level any file needs is the level applied (`RLD-02`):

| Change | Level | Reason the reload gives |
|---|---|---|
| A check file | `none` | Checks are read fresh for every execution. |
| An `.sqf` that defines a `CfgFunctions` function that is not final on any machine | `mission` | Recompiled in place. |
| Any other `.sqf`, such as a CBA `PREP` function, which is final | `mission-restart` | Final code changes only at mission start, with CBA's cache off in the session mission. |
| A new or deleted file | `game-restart` | File patching finds only files in the built PBO, so the mod must be rebuilt before this reload. |
| `config.cpp` and any other file (`.hpp`, `.h`, `.inc`, `.ext`, string tables, models, textures, sounds) | `game-restart` | Config is read at game start; the others are not reloaded by file patching. |

The runner asks every machine which `CfgFunctions` functions an `.sqf` defines and whether each is final there, by the function's file under the mod's `prefix`.

## What this build applies

- **`none`** is applied at once: nothing in the game changes, and the next run reads the checks from disk.
- **`game-restart`** stops every process of the session and starts them again in the same session folder, each machine a new start with its start number one higher and its own profile folder (`machines\<name>-<start>`). Every machine captures a new baseline, and the runner takes a new snapshot of the watched files.
- **`mission`** and **`mission-restart`** are refused, exit code 4, and the session stays `ready`. Arma 3 keeps every mod file it has read open for the life of its process and serves later reads from that hold, so a file cannot be changed while the session runs and a process that read it never sees a change (spike `SPK-08`, [research](../research/27-spikes-reload.md)). To load such a change, stop the session, change the file, and start it again.

The mod must be loaded from a development build with a `source` folder in `shakedown.toml` ([The project file](project-file.md)): `reload` watches the files under each setup mod's `source`. A file outside every watched folder and the checks folder is refused with exit code 3, naming the watched folders.

## Running a reload

```
shakedown reload                       # what changed under the watched folders since the session was last ready
shakedown reload checks\crate\open.sqf # the files named, relative to the current folder
shakedown reload --json                # the Reload shape below
```

The changed files are found by comparing every file under the watched folders with the snapshot the runner took when the session last became ready, by content: a file with new content, a new file and a deleted file. A file named on the command line is classified whether it changed or not, which is how a game restart is asked for: name a `config.cpp`.

A reload is refused while a run, another reload or a reset is in progress, with what is in progress; for a run, its progress and `shakedown run stop`.

The result, as `--json` prints it and the `reload` tool returns it:

```json
{
  "id": "rl-20261109-140312-4",
  "state": "done",
  "level": "game-restart",
  "files": [{"path": "C:\\...\\skdfx\\addons\\main\\config.cpp", "level": "game-restart", "reason": "Config is read at game start, so the game restarts."}],
  "machines": ["server", "p1", "p2", "hc1"],
  "reason": null,
  "started_at": "2026-11-09T14:03:12.104Z",
  "duration_s": 47.2
}
```

`machines` are the machines the reload reached. `duration_s` is the time from the reload order to every machine ready on the new code. `shakedown status` shows the last reload as `last_reload`.

## A reload that does not complete

When a machine does not come back during a game restart, because its process exits or the start ceiling passes, the reload ends `failed`, exit code 4, with `reason` naming the level it was applying, the machine and why, and the session is `broken` with the same reason. Every check run on it is `invalid` / `session-broken`.

`shakedown reload` on a broken session always restarts the game, whatever the files need: it is the way back for a session broken by a lost machine, a failed reset or a failed reload. The other is `shakedown session stop`.

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

Every machine captures a new baseline at the new mission start. The result file lists every machine start that took part in the run, so a run with a `@reset mission` check shows the old and the new start of each client.

An order sent to a machine while its mission restarts, such as an `exec` from another window, waits until the machine is ready again, for at most the setup's start ceiling, then runs with its whole ceiling and is answered once.

When the mission restart does not bring a machine back, that check ends `invalid` / `reset-failed`, naming the machine, and the session is `broken`: the checks after it end `invalid` / `session-broken`, and the run exits 2. `shakedown reload` brings it back.
