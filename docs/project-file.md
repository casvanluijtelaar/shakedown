---
type: Documentation
title: The project file
description: The keys of shakedown.toml for singleplayer and dedicated setups and for a mod that reload watches, with an example, how relative paths and defaults work, and what a mistake in the file looks like.
status: draft
---

# The project file

A project is a folder with a `shakedown.toml` file. Every command finds it from the current folder upward, or takes `--project <dir>`. The file is read fresh by every command. Its full contract is [spec/contract/cli.md](../spec/contract/cli.md); this page covers what a singleplayer and a dedicated setup need. [Dedicated sessions](dedicated-sessions.md) says how a dedicated session runs.

## Example

```toml
game = 'C:\Program Files (x86)\Steam\steamapps\common\Arma 3'
tests = "tests"

[mods.cba]
path = 'C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\450814997'
patch = "cba_main"

[setups.sp]
mode = "singleplayer"
mods = ["cba"]

[setups.dedicated]
mode = "dedicated"
clients = 2
headless = 1
mods = ["cba"]
```

A project with no mod declares `mods = []` in its setup and no `[mods.*]` table; `e2e/shakedown.toml` in this repository is one.

## Keys

| Key | Needed | Meaning |
|---|---|---|
| `game` | yes | The Arma 3 folder, the one that holds `arma3_x64.exe`. |
| `tests` | no, default `tests` | The folder of the check files. |
| `lib` | no, default `_lib` in the tests folder | The library folder: each `.sqf` file directly in it is compiled on every machine into `skd_lib_<file name>`, for helpers checks share ([Helpers from the library folder](check-api.md#helpers-from-the-library-folder)). No file under it is a check. A folder that does not exist is an empty library. |
| `base_port` | no, default 2402 | A dedicated server's game port, 1024 to 65530; it and the next four ports must be free. |
| `slow_step` | no, default 10 | Seconds a step may take before the result file and the run's lines flag it `slow`, a number above 0 such as `15` or `2.5` ([Every setup, repeats and slow steps](every-setup-and-repeats.md#slow-steps-and-slow-checks)). |
| `mods.<name>.path` | yes, for each mod | The mod's folder, such as a Workshop folder or a `@mod` folder. |
| `mods.<name>.patch` | no | A `CfgPatches` class of the mod; the result file records its `versionStr` as the game loaded it. |
| `mods.<name>.version` | no, needs `patch` | The `versionStr` of `patch` every machine must load, as the exact string the result file records. A start in which a machine loaded another version fails before any check runs ([When a start is refused or fails](starting-and-failed-starts.md#a-mod-in-another-version)). Without it any version is taken. |
| `mods.<name>.source` | no | The folder of your own files of the mod, the one holding its addon folders, which `reload` watches ([Reloading](reloading.md)). |
| `mods.<name>.prefix` | with `source` | The game path of `source`, such as `'z\agh\addons'`. |
| `mods.<name>.rebuild` | no | A command line that makes the build the game loads from your files, such as `hemtt build`. A `game-restart` reload runs it with `cmd /C` in the project folder, after every game process has ended and before any starts again, and a `mission-restart` reload once no game process holds a changed file, while the game still holds the mod's PBOs, so a rebuild that serves both writes only the loose files that changed ([Reloading](reloading.md#setting-a-mod-up-for-reload)); it must end within 120 s with exit code 0. It is read fresh at each reload. |
| `setups.<name>.mode` | yes | `"singleplayer"` or `"dedicated"`. |
| `setups.<name>.clients` | no, default 1 | A dedicated setup's game clients, `p1` to `pN`, 1 to 8. |
| `setups.<name>.headless` | no, default 0 | A dedicated setup's headless clients, `hc1` to `hcN`, 0 to 8. |
| `setups.<name>.late` | no, default 0 | A dedicated setup's late game clients, 0 to 8, named after its clients: `clients = 2` and `late = 1` make `p3` late. The session does not start them; a check joins one with `skd_fnc_join` ([Dedicated sessions](dedicated-sessions.md#joining-late-leaving-and-joining-again)). |
| `setups.<name>.mods` | yes | The names of the mods to load, in load order. The framework's own addon is loaded after them. |
| `setups.<name>.world` | no, default `"VR"` | The world the session mission is on. |
| `setups.<name>.spawn` | no, default `[100, 100]` | Where `p1` starts; `pN` starts 10 m further east per N. |
| `setups.<name>.start_timeout` | no, default 180 | Seconds from the first launch to the session being ready, 30 to 1800. A dedicated start of a server, two clients and a headless client takes about 45 s on the test machine. |

A singleplayer setup always has one game client, `p1`, and no headless client; `clients`, `headless` and `late` are ignored for it. Single quotes keep backslashes in Windows paths as they are. A relative path is relative to the project folder, and may climb out of it: with the project in `acceptance\shakedown\` of the mod's repository, `path = '..\..\.hemttout\build'` names the repository's build. Shakedown resolves `.` and `..` before it starts the game, so a committed `shakedown.toml` needs no absolute path to the mod. Mod and setup names are letters, digits and `-`, starting with a letter.

## Mistakes

An unknown key, a missing key, a value of the wrong type or out of range, or a setup that names an undeclared mod is refused with exit code 3, naming the key and its line:

```
shakedown.toml, line 12: setups.sp.start_timeout must be a whole number from 30 to 1800, got 5.
```

A file that names the check folder with `checks` is refused the same way, as an unknown key; the key is `tests`:

```
shakedown.toml, line 2: unknown field `checks`, expected one of `game`, `tests`, `lib`, `base_port`, `slow_step`, `mods`, `setups`. Fix the key as contract/cli.md describes the project file.
```

A `version` without a `patch` is refused the same way, since the version is that class's `versionStr`:

```
shakedown.toml, line 8: mods.ace.patch is required with version: the version is the versionStr of that CfgPatches class.
```

A setup name the file does not declare is refused with the setups it does declare.
