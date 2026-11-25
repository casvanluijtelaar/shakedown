---
type: Documentation
title: The project file
description: Every key of shakedown.toml with examples - the game, the tests and library folders, base_port and slow_step, mods with CBA, ACE, pinned versions and the keys reload watches, and singleplayer and dedicated setups with clients, headless clients and late joiners - how relative paths and defaults work, and what a mistake in the file looks like.
status: draft
---

# The project file

A project is a folder with a `shakedown.toml` file. Every command finds it by looking in the current folder and then upward, or takes `--project <dir>`. The file is read fresh by every command, so an edit takes effect at the next command; a running session keeps the setup it was started with until it is stopped.

## A complete example

```toml
game = 'C:\Program Files (x86)\Steam\steamapps\common\Arma 3'   # required
tests = "tests"        # default "tests"
lib = "tests/_lib"     # default "_lib" in the tests folder
base_port = 2402       # default 2402
slow_step = 10         # default 10 seconds

[mods.cba]
path = 'C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\450814997'
patch = "cba_main"
version = "3.19.0.260808"    # optional: refuse a start on any other version

[mods.ace]
path = 'C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\463939057'
patch = "ace_main"

[mods.mymod]
path = '..\.hemttout\build'  # the build the game loads, relative to the project folder
patch = "mymod_main"
source = '..\addons'         # makes the mod watched by reload
prefix = 'z\mymod\addons'    # the game path of source
rebuild = "hemtt build"      # run by a reload while the game lets go of the files

[setups.sp]
mode = "singleplayer"
mods = ["cba", "mymod"]

[setups.sp-ace]
mode = "singleplayer"
mods = ["cba", "ace", "mymod"]

[setups.dedicated]
mode = "dedicated"
clients = 2            # p1 and p2
headless = 1           # hc1
late = 1               # p3, which a check makes join
mods = ["cba", "mymod"]
world = "VR"
spawn = [100, 100]
start_timeout = 180
```

The smallest file names the game and one setup:

```toml
game = 'C:\Program Files (x86)\Steam\steamapps\common\Arma 3'

[setups.sp]
mode = "singleplayer"
mods = []
```

A project with no mod declares `mods = []` in its setup and no `[mods.*]` table.

## Top-level keys

| Key | Needed | Meaning |
|---|---|---|
| `game` | yes | The Arma 3 folder, the one that holds `arma3_x64.exe` (and `arma3server_x64.exe`, for a dedicated setup). |
| `tests` | no, default `tests` | The folder of the check files ([Writing checks](writing-checks.md)). A file that has a `checks` key is refused as an unknown key, naming `tests` among the keys it expects. |
| `lib` | no, default `_lib` in the tests folder | The library folder: each `.sqf` file directly in it is compiled on every machine into `skd_lib_<file name>`, for helpers checks share ([Shared helpers](writing-checks.md#shared-helpers-the-library-folder)). No file under it is a check. A folder that does not exist is an empty library. It is read when a session starts and at each `reload`. |
| `base_port` | no, default 2402 | A dedicated server's game port, 1024 to 65530; it and the next four ports must be free when a dedicated session starts. Singleplayer uses no port. |
| `slow_step` | no, default 10 | Seconds a step may take before the result file and the run's lines flag it `slow`: a number above 0, a whole number such as `15` or a decimal such as `2.5` ([Slow steps](every-setup-and-repeats.md#slow-steps-and-slow-checks)). |

## Mods

Each `[mods.<name>]` table declares one mod. A setup loads it by that name.

| Key | Needed | Meaning |
|---|---|---|
| `path` | yes | The mod's folder as `-mod` takes it: a Workshop folder, a `@mod` folder or a build output. |
| `patch` | no | A `CfgPatches` class of the mod. The result file records its `versionStr` as each machine loaded it, under `session.machines[].mods`. |
| `version` | no, needs `patch` | The `versionStr` of `patch` every machine must load, as the exact string the result file records. A start in which any machine loaded another version, or does not load `patch` at all, fails before any check runs ([A mod in another version](starting-and-failed-starts.md#a-mod-in-another-version)). Without it any version is taken. |
| `source` | no | The folder of your own files of the mod, the one holding its addon folders, which `reload` watches ([Reloading changed mod files](reloading.md)). |
| `prefix` | with `source` | The game path of `source`, such as `'z\mymod\addons'`. |
| `rebuild` | no | A command line that makes the build the game loads from your files, such as `hemtt build`, or copies the changed files into it. A `game-restart` reload runs it with `cmd /C` in the project folder, after every game process has ended and before any starts again; a `mission-restart` reload runs it once no game process holds a changed file, while the game still holds the mod's PBOs and every other file it read, so a rebuild that serves both writes only the loose files that changed ([Setting a mod up for reload](reloading.md#setting-a-mod-up-for-reload)). It must end within 120 s with exit code 0. It is read fresh at each reload, so adding it needs no new session. |

### CBA

Declare CBA_A3 like any mod and list it before the mods that need it. A check that presses a CBA keybind needs CBA on that machine; a check that presses no key needs no CBA, and Shakedown itself does not need it.

```toml
[mods.cba]
path = 'C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\450814997'
patch = "cba_main"
```

### ACE

ACE3 needs CBA. Give it `patch = "ace_main"` so the result file records the ACE version each machine loaded. With ACE loaded, a check can use ACE interaction actions ([ACE interaction actions](ace-interaction.md)), and the reset between checks gives every player ACE's full heal ([Scenes, resets and cleanups](scenes-resets-and-cleanups.md)).

```toml
[mods.ace]
path = 'C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\463939057'
patch = "ace_main"
version = "3.21.2.113"   # optional: stop the start when the Workshop has updated ACE

[setups.sp-ace]
mode = "singleplayer"
mods = ["cba", "ace", "mymod"]
```

### Pinning a version

Workshop mods update themselves. To make a start refuse a version you have not checked against, run once without `version`, copy the version from the result file's `session.machines[].mods`, and write it down:

```json
"mods": [{ "mod": "cba", "patch": "cba_main", "version": "3.19.0.260811" }]
```

```toml
[mods.cba]
path = '...'
patch = "cba_main"
version = "3.19.0.260811"
```

Shakedown does not download or pin a Workshop version; it only refuses a version you did not name.

## Setups

Each `[setups.<name>]` table is one way to run the checks: a mode, the machines and the mods. A check runs in every setup unless its `@setups` header names some ([Writing checks](writing-checks.md#the-header)).

| Key | Needed | Meaning |
|---|---|---|
| `mode` | yes | `"singleplayer"` or `"dedicated"`. |
| `mods` | yes | The names of the mods to load, in load order. The framework's own addon is loaded after them. |
| `clients` | no, default 1 | A dedicated setup's game clients, `p1` to `pN`, 1 to 8. |
| `headless` | no, default 0 | A dedicated setup's headless clients, `hc1` to `hcN`, 0 to 8. |
| `late` | no, default 0 | A dedicated setup's late game clients, 0 to 8, named after its clients: `clients = 2` and `late = 1` make `p3` late. The session does not start them; a check joins one with `skd_fnc_join` ([Joining late, leaving and joining again](dedicated-sessions.md#joining-late-leaving-and-joining-again)). |
| `world` | no, default `"VR"` | The world the session mission is on: a class name in `CfgWorlds`, such as `"Altis"` or `"Stratis"`. |
| `spawn` | no, default `[100, 100]` | Where `p1` starts on the world; `pN` starts 10 m further east per N. On a dedicated session the clients can start in each other's places ([Scenes, resets and cleanups](scenes-resets-and-cleanups.md)). |
| `start_timeout` | no, default 180 | Seconds from the first launch to the session being ready, 30 to 1800. A dedicated start of a server, two clients and a headless client takes about 45 s, so a value below 60 cannot hold it. A start that hangs costs this whole time before it fails ([A client that hangs](starting-and-failed-starts.md#a-client-that-hangs)). |

A singleplayer setup always has one game client, `p1`, and no headless client; `clients`, `headless` and `late` are ignored for it.

## Names, paths and quoting

- Mod and setup names are letters, digits and `-`, starting with a letter: `sp`, `dedicated-2`, `sp-ace`.
- Single quotes keep backslashes in Windows paths as they are. In double quotes, write `\\`.
- A relative path is relative to the project folder, and may climb out of it. With the project in `tests\shakedown\` of the mod's repository, `path = '..\..\.hemttout\build'` names the repository's build. Shakedown resolves `.` and `..` before it starts the game, which refuses a `-mod` folder whose path holds `..`, so a committed `shakedown.toml` needs no absolute path to the mod.

## Mistakes

An unknown key, a missing key, a value of the wrong type or out of range, or a setup that names an undeclared mod is refused with exit code 3, naming the key and its line. Nothing starts. `list` and `run` refuse the same way.

```
shakedown.toml, line 12: setups.sp.start_timeout must be a whole number from 30 to 1800, got 5.
```

A file that names the check folder with `checks` is refused as an unknown key; the key is `tests`:

```
shakedown.toml, line 2: unknown field `checks`, expected one of `game`, `tests`, `lib`, `base_port`, `slow_step`, `mods`, `setups`. Fix the key as contract/cli.md describes the project file.
```

A `version` without a `patch` is refused, since the version is that class's `versionStr`:

```
shakedown.toml, line 8: mods.ace.patch is required with version: the version is the versionStr of that CfgPatches class.
```

A `slow_step` that is a number but not above 0, such as `-1` or `0`:

```
shakedown.toml, line 3: slow_step must be a number of seconds above 0, got -1.
```

A `slow_step` that is not a number, such as the string `"10"`, gets the message for a key of the wrong type:

```
shakedown.toml, line 3: key `slow_step`: invalid type: string "10", expected f64. Fix the key as contract/cli.md describes the project file.
```

A tests folder that does not exist:

```
The tests folder C:\mods\mymod\tests does not exist. Create it, or set `tests` in shakedown.toml.
```

A setup name the file does not declare, given to `--setup`, is refused with the setups it does declare. No `shakedown.toml` in the folder or any folder above it is refused too; give `--project <dir>` or run the command inside the project. Through the [MCP tools](agent-tools.md) each of these is the error code `config-error` (or `unknown-setup` for a setup name).
