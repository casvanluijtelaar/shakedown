---
type: Research
title: Hot reload of mod code and checks
description: What of a mod and of the checks can be reloaded into a running Arma 3 session without a game restart, what file patching needs on each machine, how CBA, ACE and HEMTT developers reload today, and what forces a restart.
status: stable
generated: { by: "claude-code/research", at: "2026-11-04T21:55:00+01:00" }
sources:
  - id: wiki-startup
    resource: "https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters"
    read: 2026-11-04
  - id: wiki-servercfg
    resource: "https://community.bistudio.com/wiki/Arma_3:_Server_Config_File"
    read: 2026-11-04
  - id: wiki-funclib
    resource: "https://community.bistudio.com/wiki/Arma_3:_Functions_Library (function attributes table)"
    read: 2026-11-04
  - id: wiki-recompile
    resource: "https://community.bistudio.com/wiki/BIS_fnc_recompile"
    read: 2026-11-04
  - id: wiki-compilefinal
    resource: "https://community.bistudio.com/wiki/compileFinal"
    read: 2026-11-04
  - id: wiki-compilescript
    resource: "https://community.bistudio.com/wiki/compileScript"
    read: 2026-11-04
  - id: wiki-mergeconfig
    resource: "https://community.bistudio.com/wiki/diag_mergeConfigFile"
    read: 2026-11-04
  - id: wiki-diagexe
    resource: "https://community.bistudio.com/wiki/Arma_3:_Diagnostics_Exe"
    read: 2026-11-04
  - id: wiki-freeext
    resource: "https://community.bistudio.com/wiki/freeExtension"
    read: 2026-11-04
  - id: forum-recompile
    resource: "https://forums.bohemia.net/forums/topic/182019-how-to-enable-true-dev-version/ and https://feedback.bistudio.com/T78735 (via web search summary only)"
    read: 2026-11-04
  - id: cba
    resource: "https://github.com/CBATeam/CBA_A3 at commit 8b15a04bd52ff7acc568a93cf6e90343e43a8a56 (release v3.19.0 is the latest): addons/xeh/fnc_compileFunction.sqf, addons/xeh/fnc_isRecompileEnabled.sqf, addons/xeh/fnc_preStart.sqf, addons/main/script_macros_common.hpp, optionals/cache_disable/config.cpp"
    read: 2026-11-04
  - id: ace
    resource: "https://github.com/acemod/ACE3 at commit 45c25247f7599687e85c5981f150efe37227393b (release v3.21.2 is the latest): docs/wiki/development/setting-up-the-development-environment.md, addons/main/script_macros.hpp, addons/main/script_debug.hpp, addons/common/XEH_preInit.sqf"
    read: 2026-11-04
  - id: hemtt
    resource: "https://github.com/BrettMayson/HEMTT at tag v1.23.1: bin/src/commands/dev.rs, bin/src/commands/launch/mod.rs, bin/src/commands/launch/launcher.rs (the book pages are generated from these doc comments)"
    read: 2026-11-04
---

# Hot reload of mod code and checks

`CR-03` asks to reload the mod's changed code and the checks into a running session without a restart. G2 wants a verdict within about 30 seconds of a change, with the game already running. This topic says what "without a restart" can mean in Arma 3: without restarting the **game** is often possible; without restarting the **mission** is possible only for some code.

## Three reload levels

| Level | What it costs | What it picks up |
|---|---|---|
| A. In mission | Seconds | Plain SQF variables, functions compiled without `compileFinal`, the checks |
| B. Mission restart | A mission load on every machine (time: **needs a spike**, see T5) | Every SQF function, including final ones, with file patching on |
| C. Game restart | A full start: 20 to 35 seconds for server, clients and headless client before a mission, per the PRD | Config, new files, the extension |

The framework picks the lowest level that a change needs, and says which level it used and why.

## File patching

File patching lets the game load unpacked files in place of the files packed in a PBO. It is off by default since 1.50 and turned on with `-filePatching`[^wiki-startup]. A server decides which clients may join with it on: `allowedFilePatching = 0` none, `1` headless clients only, `2` all clients[^wiki-servercfg]. `filePatchingExceptions[]` lets listed Steam ids join regardless, fully working since 2.10[^wiki-servercfg].

ACE's developer guide gives the rules that matter[^ace]:

- A file must exist in the built PBO for file patching to find it. A new file means rebuilding the PBO and restarting the game.
- Configs are not patched at run time, only at load time.
- Changing a file "requires only a restart of the mission".

HEMTT does the build. `hemtt dev` builds the mod into `.hemttout/dev` unbinarized, "with links back to the original addon folders", so edits to the source tree are what the game reads[^hemtt][^ace]. `hemtt launch` runs `hemtt dev` first, needs `mainprefix` set, checks that a link to the build exists in the Arma 3 folder under the main prefix, and starts the game with `-filePatching` by default (`file_patching = false` or `--no-filepatching` turns it off) plus `-skipIntro -noSplash -showScriptErrors -debug`[^hemtt]. ACE's guide makes the same link by hand with `mklink /J "[Arma 3]\z\ace" ".hemttout/dev"` and starts with `-mod=@CBA_A3;z\ace -filePatching`[^ace].

**Forces:**
- Every process of a reload-capable session starts with `-filePatching`, and the framework's server config sets `allowedFilePatching = 2`. `CR-01`, `CR-03`.
- The mod under test is loaded from a HEMTT dev build (or an equivalent linked unpacked tree), not from a packed release. The framework does not build mods (PRD scope), so it states this as a precondition and checks it before a run. `CR-06`.
- On one test machine every process reads the same Arma 3 folder, so one dev build reaches the server, all clients and the headless clients at once. A Linux server or a second machine needs its own copy or link: **needs a spike** when that setup comes up.
- A change that adds or removes a file, or touches config, is a game restart. `CR-03`.

## What can be reloaded

### Functions from `CfgFunctions`

Functions are compiled final by the Functions Library and are "protected against rewriting during the mission", except in missions previewed from the editor and in missions whose `Description.ext` has `allowFunctionsRecompile = 1`[^wiki-recompile]. `BIS_fnc_recompile` recompiles one function by name, or all functions with a mode number[^wiki-recompile]. The `recompile = 1` function attribute recompiles a function at mission start; functions defined in `Description.ext` are always compiled at mission (re)start[^wiki-funclib].

Whether `allowFunctionsRecompile` also lets an **addon's** functions be recompiled mid-mission is unclear: a 2015 forum thread says functions defined in a PBO stay final, and a Feedback Tracker report says a modified BIS function was not recompiled from `uiNamespace`[^forum-recompile]. That is **needs a spike** (HR-1).

**Forces:** the framework's test mission sets `allowFunctionsRecompile = 1`. The framework tries level A with `BIS_fnc_recompile` per changed function on every machine, and falls back to level B when HR-1 fails. `CR-03`.

### CBA `PREP` functions

`PREP` is the macro CBA mods use to define functions from `fnc_*.sqf` files in XEH preStart and preInit[^cba]. By default it calls `CBA_fnc_compileFunction`, which compiles the file once into `uiNamespace` with `compileScript [file, true]` (final), and copies that into `missionNamespace` at each mission start[^cba]. CBA's own documentation of the macro says the function "can not be overwritten without a mission restart"[^cba]. `compileFinal` code cannot be overwritten by assignment, `publicVariable` or any other command, and a copy of final code is final too[^wiki-compilefinal][^wiki-compilescript].

When recompiling is enabled, `CBA_fnc_compileFunction` instead compiles the file afresh at each mission start, still with `compileFinal`[^cba]. Recompiling is enabled when `CfgSettings >> CBA >> Caching >> compile` is `0` in the addon config or the mission config[^cba]. CBA ships `optionals/cache_disable` (`cba_cache_disable.pbo`), which sets `xeh`, `compile` and `functions` caching to `0`[^cba]. ACE's guide lists the three ways to disable the cache: load that PBO, put the same `CfgSettings` class in the test mission's `Description.ext`, or define `DISABLE_COMPILE_CACHE` in a component's `script_component.hpp`[^ace].

Only the last one gives level A. With `DISABLE_COMPILE_CACHE`, `PREP` becomes `compile preprocessFileLineNumbers`, not final, in both CBA's and ACE's macros[^cba][^ace]. ACE then collects each component's `PREP` block (`PREP_RECOMPILE_START` and `PREP_RECOMPILE_END`) into `ACE_PREP_RECOMPILE`, so `[] call ACE_PREP_RECOMPILE` recompiles them without a mission restart[^ace]. But that define is in the mod's source, and the framework must not patch the mod (PRD scope).

**Forces:**
- For a CBA mod as built, the reliable reload is level B: a mission restart with caching disabled. The framework's test mission carries the `CfgSettings >> CBA >> Caching` class with all three values at `0`, so no extra PBO is needed. `CR-03`, `CR-02`.
- Level A for a CBA mod works only if the mod's developer builds with `DISABLE_COMPILE_CACHE`. The framework can detect whether a function is final with `isFinal` (named in `compileFinal`'s see-also[^wiki-compilefinal]) and choose the level from that, never assuming.
- The level-B cost is the mission restart time on every machine, which decides whether G2's 30 seconds holds. That is T5's measurement. `CR-03`, G2.

### Code captured by handlers

ACE's debug header defines `LINKFUNC(x)` as `{call FUNC(x)}` when the cache is disabled, and as `FUNC(x)` otherwise[^ace]. The reason is that an event handler or per-frame handler registered with a function's code keeps that code; a recompiled function is a new value in the variable, and only a handler that looks the variable up by name at call time sees it.

**Forces:** after a level-A reload, handlers the mod registered at init still run the old code unless the mod registers them by name. The framework cannot know which, so a level-A reload is reported as partial, and a check that fails after one is rerun after a level-B reload before it counts. **needs a spike** (HR-3). `CR-03`, `CR-31`.

### Scripts, mission files and the checks

- A script run with `execVM` or compiled with `compileScript` or `compile preprocessFileLineNumbers` reads its file each time, so with file patching a changed script is used on its next run. `compileScript` is an alias for `compile (prefixHeader + preprocessFileLineNumbers path)`[^wiki-compilescript]. Whether the engine caches preprocessed files between calls is **needs a spike** (HR-2).
- Mission files are read at mission start; `Description.ext` functions are recompiled at every mission (re)start[^wiki-funclib]. A changed mission is level B.
- The checks belong to the framework. If the runner sends each check's source over the channel of `12-remote-execution.md` and the agent compiles it with plain `compile` before running it, the checks are reloaded on every run, with no file patching and no restart. The SQF check API itself should be compiled without `compileFinal` so the framework's own SQF can be reloaded the same way.

**Forces:** the runner owns check source and ships it per run. `CR-03`, `CR-40`, `CR-50`.

### Config, string tables, models

Config is read at game start[^ace]. `diag_mergeConfigFile` merges a config file into the running game, and new entities pick up the change without a mission restart[^wiki-mergeconfig]. It exists only in the diagnostics executable, which ships on the development branch and has multiplayer removed completely[^wiki-mergeconfig][^wiki-diagexe].

String tables, models (`.p3d`), textures and sounds were not covered by the sources read here. Treat them as level C: **unverified**, and a spike only if a mod's checks need it.

**Forces:** a config change is a game restart in every setup the framework supports. Config merge stays out of the framework: it is development branch only and singleplayer only. `CR-03`.

### The extension

`freeExtension` unloads an extension so its developer can replace it, but only on the development branch; on stable it does nothing[^wiki-freeext]. A changed extension is a game restart, which is why `12-remote-execution.md` keeps the framework's extension thin.

## What cannot be reloaded, and how the user is told

| Change | Level | Why |
|---|---|---|
| A check | A | The runner sends its source each run. |
| A non-final mod function | A | Recompile in place. |
| A `CfgFunctions` function | A if HR-1 passes, else B | Final by default. |
| A cached CBA `PREP` function | B | Final; recompiled only at mission start with caching off. |
| A script read at each run | A, if HR-2 passes | Read from the loose file. |
| Mission files | B | Read at mission start. |
| A new or deleted file | C, plus a rebuild | File patching only finds files that exist in the PBO. |
| Config (`config.cpp`, `.hpp` in config) | C | Read at game start; merge is development branch only. |
| String tables, models, textures, sounds | C (**unverified**) | Not covered. |
| The extension | C | `freeExtension` is development branch only. |

**Forces:** the reload command takes the list of changed files (from the mod developer, or by comparing file times in the dev build), classifies each by the table above, runs the highest level needed, and reports per file which level it used and why. A change that needs level C says so before it restarts, so `CR-03`'s "without a restart" is never claimed falsely. Requirements touched: `CR-03`, `CR-31` (a check run on half-reloaded code is reported with that fact), `CR-42` (the time of the reload is reported).

## How a reload reaches every machine

- Files: on one test machine every process reads the same linked dev build, so nothing is copied.
- Level A: the runner sends the recompile order to every machine over its own channel, or the server sends it with `remoteExec` to all (`0`). Each machine compiles its own copy; `publicVariable` of a final value would make it final on the receivers too[^wiki-compilefinal], so code is never broadcast as a value.
- Level B: the server restarts the mission and every connected client and headless client loads it again. How, and whether clients stay connected, is T5's question.
- The reload is done when every machine reports the new version of each function, for example a hash of `toString` of the function, which the agent returns per machine. `CR-22`.

## Needs a spike

| # | Question | Pass condition |
|---|---|---|
| HR-1 | `BIS_fnc_recompile` on an addon's `CfgFunctions` function mid-mission | With `-filePatching`, `allowFunctionsRecompile = 1` and an edited `.sqf` in the dev build, `["TAG_fnc_x"] call BIS_fnc_recompile` on the server and a client makes `TAG_fnc_x` return the new value on both, without a mission restart. |
| HR-2 | Loose scripts read fresh | An edited script run again with `execVM` and with `compileScript` gives the new behaviour on the next call. |
| HR-3 | Handlers keep old code | After a level-A recompile of a function the mod registered as an event handler by value, the handler still runs the old code; after level B it runs the new code. |
| HR-4 | CBA caching off through the mission | A test mission with `CfgSettings >> CBA >> Caching` at `0` and no `cba_cache_disable.pbo` picks up an edited `PREP` function after a mission restart, on a dedicated server, two clients and a headless client; the RPT shows CBA's "CBA CACHE DISABLED?" line. |
| HR-5 | Level-B time | The time from the reload order to every machine reporting the new function hash, on the `arma-grappling-hook` setup with two clients. Pass: under G2's 30 seconds (proposal). |
| HR-6 | File patching on a headless client | With `allowedFilePatching = 1`, a headless client started with `-filePatching` joins and reads the edited file. |

## Development and profiling branch only

- `diag_mergeConfigFile` and the diagnostics executable: development branch, no multiplayer[^wiki-mergeconfig][^wiki-diagexe].
- `freeExtension`: development branch[^wiki-freeext].

Neither is recommended: the first works only in singleplayer, the second is avoided by design.

[^wiki-startup]: Bohemia Interactive Community Wiki, Arma 3: Startup Parameters (`-filePatching`, `-noFilePatching`).
[^wiki-servercfg]: Bohemia Interactive Community Wiki, Arma 3: Server Config File (`allowedFilePatching`, `filePatchingExceptions`).
[^wiki-funclib]: Bohemia Interactive Community Wiki, Arma 3: Functions Library, function attributes.
[^wiki-recompile]: Bohemia Interactive Community Wiki, `BIS_fnc_recompile`.
[^wiki-compilefinal]: Bohemia Interactive Community Wiki, `compileFinal`.
[^wiki-compilescript]: Bohemia Interactive Community Wiki, `compileScript`.
[^wiki-mergeconfig]: Bohemia Interactive Community Wiki, `diag_mergeConfigFile`.
[^wiki-diagexe]: Bohemia Interactive Community Wiki, Arma 3: Diagnostics Exe.
[^wiki-freeext]: Bohemia Interactive Community Wiki, `freeExtension`.
[^forum-recompile]: Bohemia forum thread 182019 (2015) and Feedback Tracker T78735, as summarised by a web search; not read in full.
[^cba]: CBA_A3 repository at commit `8b15a04`.
[^ace]: ACE3 repository at commit `45c2524`.
[^hemtt]: HEMTT repository at tag `v1.23.1`.
