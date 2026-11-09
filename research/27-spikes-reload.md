---
type: Research
title: "Spikes SPK-08 and SPK-01 across a mission restart: reloading by restart"
description: "What a changed mod file does in a running game on the test machine in M6 - the game holds every loose file it read until its process ends, so no running process takes a change and SPK-08 fails - with what ACE3, HEMTT and Neefay's harness actually do to reload, CBA's cache shown off, the time of a mission restart in singleplayer and on a dedicated session, and orders sent across mission restarts, none lost and none run twice."
status: draft
generated: { by: "claude-code/developer", at: 2026-11-09T15:20:00+01:00 }
sources:
  - { id: probe, resource: "probes run by hand on the Windows test machine, 2026-11-09, Arma 3 2.22.154089 stable, CBA_A3 3.19.0 (workshop 450814997): the installed fixture file opened for writing and renamed while a session ran, the processes holding it listed through the Windows Restart Manager, the game started directly with and without -debug and with -maxFileCacheSize, and the fixtures' link in the game folder pointed at a changed copy before a mission restart", read: 2026-11-09 }
  - { id: ace, resource: "https://github.com/acemod/ACE3 at master, docs/wiki/development/setting-up-the-development-environment.md, sections 4.1.1, 4.2 and 7 (read with gh)", read: 2026-11-09 }
  - { id: hemtt, resource: "https://github.com/BrettMayson/HEMTT at tag v1.23.1 (the latest release, 2026-10-25), bin/src/modules/file_patching.rs; issues 669 and 1256 on the link hemtt dev makes", read: 2026-11-09 }
  - { id: neefay, resource: "https://github.com/Neefay/arma3-llm-harness at 21aac67e (no license: read for ideas only), README.md and host/src/arma_harness/harness.py, the reload method and the server config it writes", read: 2026-11-09 }
  - { id: web, resource: "web search of 2026-11-09 for reports of Arma 3 holding files read through file patching, and gh issue searches for file patching in acemod/ACE3, CBATeam/CBA_A3 and BrettMayson/HEMTT: no report of the hold, and none of editing a held file while the game runs", read: 2026-11-09 }
  - { id: tests, resource: "crates/shakedown/tests/m06.rs, spk_08_the_game_holds_every_mod_file_it_read_until_its_process_ends, spk_01_nfr_rl_04_us_011_no_order_lost_or_run_twice_across_mission_restarts_in_singleplayer and spk_01_nfr_rl_04_us_011_mission_restarts_on_a_dedicated_session, runs of 2026-11-09 on the Windows test machine", read: 2026-11-09 }
---

# Spikes SPK-08 and SPK-01 across a mission restart: reloading by restart

M6 ran `SPK-08` and the mission-restart part of `SPK-01` (`spec/06-open-questions.md`) first, on the Windows test machine with Arma 3 2.22.154089 and CBA_A3 3.19.0. `SPK-01` passed across mission restarts. `SPK-08` failed, and not on what its fallback covers: CBA's cache is off, but the game itself never lets a running process see a changed mod file.

## The game holds every mod file it read

With a session of `sp-cba` running, the installed `fnc_version.sqf` of `@skd_fixture_cba`, which the game read through file patching when CBA compiled the `PREP` function, could not be opened for writing (Windows error 32, a sharing violation) or renamed[^probe][^tests]. The Windows Restart Manager named every game process of the session as holding it: on a dedicated session the server, both clients and the headless client[^probe]. Every loose file the game read was held the same way: `fn_version.sqf` (a `CfgFunctions` function) and `XEH_preInit.sqf` too; `config.cpp`, which the game reads from the PBO and not as a loose file, was free. The hold starts when the game reads the file at its start, lasts across mission restarts, and ends only with the process; once the session stopped, the file could be written at once[^tests].

It is the game's doing, not the framework's:

- A game started directly, with no runner, held the file the same way, with `-debug` and without it[^probe].
- `-maxFileCacheSize=0` did not change it; the RPT shows the engine raised the value to 512 MB (`FileServer MaxCacheSize`)[^probe].
- A real folder in the game folder instead of the install's junction was held the same way[^probe].

A process also keeps reading the content it first read. With the fixture's link in the game folder pointed at a copy with the file changed to `"2"`, a mission restart in singleplayer still compiled `"1"`, and `loadFile` of the file's game path returned `"1"`, though CBA logged its cache off for that mission[^probe]. So a running process takes no change to a loose file by any path: not by editing it, which the hold forbids, and not by changing what its path points at.

## CBA's cache is off through the session mission

Every machine's RPT shows `[CBA] (xeh) INFO: ... CBA CACHE DISABLED? (Disable caching with cba_cache_disable.pbo): _compileType=compile` at each mission start, and `["compile"] call CBA_fnc_isRecompileEnabled` is `true`[^probe][^tests]. CBA 3.19.0's `CBA_fnc_isRecompileEnabled` reads `CfgSettings >> CBA >> caching` from the mission config as well as the game's, as research HR-4 expected, so `INT-CBA-02` holds and `cba_cache_disable.pbo` is not needed. That half of the pass condition holds; the other half, an edited `PREP` function live on every machine after a level-B reload, cannot: on a dedicated session the server keeps its process through level B, and in singleplayer `p1` does.

## What follows for the spec

`SPK-08`'s fallback (load `cba_cache_disable.pbo`) does not help. What would:

- **Every `.sqf` change of a mod is `game-restart`.** Only a new process reads a changed file, so `RLD-02`'s `mission` and `mission-restart` rows, `P-08`'s level A and level B as reloads, `US-054` and `US-055` change, and the time a mod developer waits is a game start (25 s in singleplayer, about 50 s for a dedicated session with two clients and a headless client on this machine). The file must also be changed while no game process holds it, so the change and the reload are one step: the reload stops the processes, the file is changed, the processes start. `FIX-03` and the fixture contract, which let a test change an installed file while a session runs, change with it.
- **The framework stages the mod's files.** The game would read a copy the framework makes, so the developer's own files are never held, and a reload makes a new copy before it restarts the processes. Level B still cannot work for a process that already read the file, so this gives a free source folder, not a cheaper level.

Both change what the spec promises beyond `m06.md`'s fallback, which the owner decides. `SPK-09` (`BIS_fnc_recompile` mid-mission) reads files through the same file server and would meet the same hold.

## Why the cited setups reload and the spike did not

Research `13-hot-reload.md` built the reload levels on three sources. Read again for what each actually does:

- **ACE3** sets up file patching as the spike did: a junction under the Arma 3 folder at the main prefix (`mklink /J "[Arma 3]\z\ace" ".hemttout/dev"`), a `hemtt dev` build, and `-filePatching` with `-debug`, `-showScriptErrors`, `-skipIntro` and `-noSplash`; its guide says file patching lets a developer "change the files in an addon while the game is running, requiring only a restart of the mission"[^ace]. The guide does not say how the file is written while the game holds it, and no ACE3, CBA_A3 or HEMTT issue reports the hold or a way round it[^web].
- **HEMTT** `hemtt dev` links each addon folder of `.hemttout/dev/addons` back to the project's addon folder, and links `<Arma 3>\<mainprefix>\<prefix>` to the build folder (to the project root since 1.19, issue 1256)[^hemtt]. The game therefore reads the developer's own files through two links; the fixtures' install reads a copy through one junction. Both are links the game follows to the same kind of loose file.
- **`Neefay/arma3-llm-harness`** does not reload through the game's files at all. Its server config sets `allowedFilePatching = 0`. Its `reload` reads the changed file of the mission's source folder on the host, preprocesses it there, sends the text to the game and runs `compile` on it, assigning the result to the function's variable; the staged copy of the mission sets `allowFunctionsRecompile` so mission functions can be assigned[^neefay]. It reloads mission functions, never an addon's, and never a final one.

So the only source that reloads in a running game is Neefay's, and it works because the game's file server never reads the changed file. Its way does not reach a final function: a `CfgFunctions` function or a CBA `PREP` function is final from the moment the game compiles it, so an assignment fails until the next mission start, when the game and CBA read the file again through the file server, which serves the held content. ACE3's claim and the spike's measurement disagree, and the sources give no reason.

The differences between ACE3's setup and the spike that remain to be run in the game, each on singleplayer first, are:

| Difference | ACE3 and HEMTT | The spike |
|---|---|---|
| Mod folder and file patching root | One folder: `-mod=z\ace`, the junction itself | The PBO from the install's `@skd_fixture_cba`, the loose tree through `<game>\skdfx_cba` |
| Links | Junction to the build, then a link per addon folder to the source | One junction to a copy |
| How the file is written | An editor's save, which may write a new file and swap it in | Opened for writing in place, and renamed |
| When the file was first read | ACE3's `PREP` at preInit with the cache off by `cba_cache_disable.pbo` or `DISABLE_COMPILE_CACHE` | CBA's preStart and each preInit, cache off through the session mission |

Two ways round the hold follow from the sources and need the game to test: Neefay's way, the framework reading the changed file itself and sending its text, with a final function replaced at a mission restart before the game or CBA compiles it again; and a file read for the first time, at a path the game has not opened, to see whether a file the game opens mid-mission is held as well.

## A mission restart, and orders across it

The mission restart itself works and is built for `@reset mission` (`RST-06`):

- **Singleplayer**: `endMission "END1"` loads the next `-autotest` entry; three mission restarts during one run each took under 3 s of the check's time[^tests].
- **Dedicated, two clients and a headless client, CBA and both fixtures**: the clients and the headless client are ended, the server kicks the users it still lists and sends `#restart`, and new client and headless client processes join. With nobody on it the server restarts the mission at once with `persistent = 1`; it reports `ready` with the next mission start count after about 16 s, and the fresh clients take about 28 s more, from launch to ready[^tests]. Ten mission restarts in one run, each the preparation of a `@reset mission` check whose body takes well under a second, gave check durations of 46.9 to 58.2 s, 48.6 s on average; nine of the ten were within 47 to 48.5 s[^tests]. A mission restart on a dedicated session costs a client's game start, so it is no cheaper than G2's 30 s allows, and it is about what a game restart of the same session costs (the session's start took 46 s).

`SPK-01`'s part across a mission restart: round trips of `1 + 1` sent without pause while the checks restarted the mission, each also counted in the machine's `uiNamespace`, which outlives a mission[^tests]:

| Machine | Mission restarts | Round trips | Lost | Run twice | 95th percentile | Slowest |
|---|---|---|---|---|---|---|
| singleplayer `p1` | 3 | 310 | 0 | 0 | 0.037 s | 2.4 s |
| dedicated `server` | 10 | 6025 | 0 | 0 | 0.125 s | 1.7 s |
| dedicated `p1`, across 11 processes | 10 | 602 | 0 | not countable: each start is a new process | 0.053 s | 56.1 s |

A first run with the hold bounded by each order's own ceiling (30 s plus the 10 s grace) lost the round trips to `p1` that waited longer than that for the new `p1`; the hold is therefore bounded by the setup's start ceiling, and the order then has its whole ceiling.

None was lost and none ran twice: every round trip answered `2`, and the count each kept process held equalled the round trips sent to it. That holds because the runner holds every new order to a machine whose mission restarts or whose process is replaced until it is `ready` again, and waits for the orders already sent there to be answered before it ends a mission or a process (`P-08`, `NFR-RL-04`). The slowest round trips are those held through a restart.
