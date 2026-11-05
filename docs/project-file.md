---
type: Documentation
title: The project file for a singleplayer setup
description: The keys of shakedown.toml a singleplayer setup needs, with an example, how relative paths and defaults work, and what a mistake in the file looks like.
status: draft
---

# The project file for a singleplayer setup

A project is a folder with a `shakedown.toml` file. Every command finds it from the current folder upward, or takes `--project <dir>`. The file is read fresh by every command. Its full contract is [spec/contract/cli.md](../spec/contract/cli.md); this page covers what a singleplayer setup needs.

## Example

```toml
game = 'C:\Program Files (x86)\Steam\steamapps\common\Arma 3'
checks = "checks"

[mods.cba]
path = 'C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\450814997'
patch = "cba_main"

[setups.sp]
mode = "singleplayer"
mods = ["cba"]
```

A project with no mod declares `mods = []` in its setup and no `[mods.*]` table; `e2e/shakedown.toml` in this repository is one.

## Keys

| Key | Needed | Meaning |
|---|---|---|
| `game` | yes | The Arma 3 folder, the one that holds `arma3_x64.exe`. |
| `checks` | no, default `checks` | The folder of the check files. |
| `mods.<name>.path` | yes, for each mod | The mod's folder, such as a Workshop folder or a `@mod` folder. |
| `mods.<name>.patch` | no | A `CfgPatches` class of the mod; the result file records its `versionStr` as the game loaded it. |
| `setups.<name>.mode` | yes | `"singleplayer"`. |
| `setups.<name>.mods` | yes | The names of the mods to load, in load order. The framework's own addon is loaded after them. |
| `setups.<name>.world` | no, default `"VR"` | The world the session mission is on. |
| `setups.<name>.spawn` | no, default `[100, 100]` | Where the player starts. |
| `setups.<name>.start_timeout` | no, default 180 | Seconds from the game's launch to the session being ready, 30 to 1800. |

A singleplayer setup always has one game client, `p1`, and no headless client; `clients` and `headless` are ignored for it. Single quotes keep backslashes in Windows paths as they are. A relative path is relative to the project folder. Mod and setup names are letters, digits and `-`, starting with a letter.

## Mistakes

An unknown key, a missing key, a value of the wrong type or out of range, or a setup that names an undeclared mod is refused with exit code 3, naming the key and its line:

```
shakedown.toml, line 12: setups.sp.start_timeout must be a whole number from 30 to 1800, got 5.
```

A setup name the file does not declare is refused with the setups it does declare.
