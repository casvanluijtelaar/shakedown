---
type: Research
title: "Spikes SPK-08 and SPK-01 across a mission restart: reloading by restart"
description: "What a changed mod file does in a running game on the test machine in M6 - the game holds every loose file it read until some time later that no wait or flush tried here controls, a released file reaches even a kept process through a mission restart, and SPK-08 holds once in singleplayer and once on a dedicated session but not as a promise - with what ACE3, HEMTT and Neefay's harness actually do to reload, CBA's cache shown off, the time of a mission restart in singleplayer and on a dedicated session, orders sent across mission restarts, none lost and none run twice, and the time of the game-restart reload chosen after it."
status: draft
generated: { by: "claude-code/developer", at: 2026-11-09T15:20:00+01:00 }
sources:
  - { id: probe, resource: "probes run by hand on the Windows test machine, 2026-11-09, Arma 3 2.22.154089 stable, CBA_A3 3.19.0 (workshop 450814997): the installed fixture file opened for writing and renamed while a session ran, the processes holding it listed through the Windows Restart Manager, the game started directly with and without -debug and with -maxFileCacheSize, and the fixtures' link in the game folder pointed at a changed copy before a mission restart", read: 2026-11-09 }
  - { id: ace, resource: "https://github.com/acemod/ACE3 at master, docs/wiki/development/setting-up-the-development-environment.md, sections 4.1.1, 4.2 and 7 (read with gh)", read: 2026-11-09 }
  - { id: hemtt, resource: "https://github.com/BrettMayson/HEMTT at tag v1.23.1 (the latest release, 2026-10-25), bin/src/modules/file_patching.rs; issues 669 and 1256 on the link hemtt dev makes", read: 2026-11-09 }
  - { id: neefay, resource: "https://github.com/Neefay/arma3-llm-harness at 21aac67e (no license: read for ideas only), README.md and host/src/arma_harness/harness.py, the reload method and the server config it writes", read: 2026-11-09 }
  - { id: web, resource: "web search of 2026-11-09 for reports of Arma 3 holding files read through file patching, and gh issue searches for file patching in acemod/ACE3, CBATeam/CBA_A3 and BrettMayson/HEMTT: no report of the hold, and none of editing a held file while the game runs", read: 2026-11-09 }
  - { id: round, resource: "the research round of 2026-11-09 on the Windows test machine, Arma 3 2.22.154089, CBA_A3 3.19.0, through the installed shakedown: share modes of the held fixture files probed with CreateFileW, loadFile of new, missing and re-pointed loose files and of other spellings of a path, the release watched while idle and after reading other files, the fixture files changed once released and a mission restart run in sp-cba and in ded-cba (two clients and a headless client), and BIS_fnc_recompile in singleplayer", read: 2026-11-09 }
  - { id: cba, resource: "https://github.com/CBATeam/CBA_A3 at tag v3.19.0, addons/xeh/fnc_compileFunction.sqf", read: 2026-11-09 }
  - { id: restart, resource: "crates/shakedown/tests/m06.rs, us_055_nfr_pf_02_ten_reloads_of_a_changed_function_on_a_dedicated_session and us_040_us_056_reload_a_check_a_new_file_and_config_in_singleplayer, runs of 2026-11-09 on the Windows test machine after the owner chose the game restart", read: 2026-11-09 }
  - { id: tests, resource: "crates/shakedown/tests/m06.rs, spk_08_the_game_holds_a_mod_file_it_read_and_a_stopped_session_frees_it, spk_01_nfr_rl_04_us_011_no_order_lost_or_run_twice_across_mission_restarts_in_singleplayer and spk_01_nfr_rl_04_us_011_mission_restarts_on_a_dedicated_session, runs of 2026-11-09 on the Windows test machine", read: 2026-11-09 }
---

# Spikes SPK-08 and SPK-01 across a mission restart: reloading by restart

M6 ran `SPK-08` and the mission-restart part of `SPK-01` (`spec/06-open-questions.md`) first, on the Windows test machine with Arma 3 2.22.154089 and CBA_A3 3.19.0, and a research round after it looked for a way for a changed mod file to reach a running session. `SPK-01` passed across mission restarts. `SPK-08` passes only when the game has let go of the changed file, which it does at a time nothing tried here controls: CBA's cache is off, and a mission restart takes a released file on every machine, but while the game holds a file nobody can change it.

## The game holds the loose files it read

The game opens every loose file it reads through file patching for reading and shares it for reading only: while it holds the file, an open for reading succeeds, and an open for writing or for deleting fails with Windows error 32, a sharing violation, so the file can be neither written nor renamed[^round]. With a session of `sp-cba` running, the installed `fnc_version.sqf` of `@skd_fixture_cba` (a CBA `PREP` function), `fn_version.sqf` of `@skd_fixture` (a `CfgFunctions` function), `XEH_PREP.hpp` and `XEH_preInit.sqf` were held this way; `config.cpp`, which the game reads from the PBO, was free[^probe][^tests][^round]. The Windows Restart Manager named every game process of a dedicated session as a holder: the server, both clients and the headless client[^probe].

- It is the game's doing, not the framework's. A game started directly, with no runner, held the file the same way, with `-debug` and without it; `-maxFileCacheSize=0` did not change it (the RPT shows the engine raised the value to 512 MB, `FileServer MaxCacheSize`); and a real folder in the game folder instead of the install's junction was held the same way[^probe].
- A file first read mid-mission is held too. `loadFile` of `\skd_spk08\a\v.sqf`, a file in a folder that no PBO covers, linked into the game folder for the probe, read it through file patching and held it[^round].
- A new file under a prefix that a PBO covers is not found: `loadFile` of the fixture's `fn_new.sqf` returned `""`[^round].
- While the game holds a file it serves the content it read. With the probe's link pointed at a copy holding other content, `loadFile` of the same path returned the first content, and so did the path in capitals, without its leading backslash and with a doubled backslash; the path with `..` or `/` was not found[^round]. In the build round, a mission restart in singleplayer with the fixture's link pointed at a changed copy still compiled `"1"`[^probe].

The hold does not last for the life of the process. The game lets go of a file at some point after it read it, and then reads it afresh[^round]:

| Session | Held since | What the machines did | Released |
|---|---|---|---|
| singleplayer `p1` | the game start | read 3000 other loose files (14.6 s) | yes: both fixture files, and the probe's file, which then gave the new content |
| singleplayer `p1` | a mission restart | about 7800 lookups of missing paths, then idle | yes, between 90 s and 140 s after the restart |
| singleplayer `p1` | a `loadFile` | idle for over 4 minutes | no |
| singleplayer `p1` | the same file | read 3090 other loose files (the order met its 60 s ceiling) | yes |
| dedicated: server, `p1`, `p2`, `hc1` | the session start | idle for 150 s | no |
| dedicated | the same | each machine read 3000 other loose files, one machine after the other (25 s to 59 s each) | yes, within 5.5 minutes of the start |
| dedicated | a mission restart | idle for 90 s, twice | no |
| dedicated | the same | each machine looked up 3000 missing paths (3 s to 8 s), then 150 s | no |
| dedicated | the same | each machine read 3000 other loose files, all at once (160 s), then 60 s, twice | no, 13 minutes after the restart |

The release follows reading many other files more often than not, but neither a time nor a count of other reads made it happen every time, and the round did not find what decides it.

## A released file reaches a running process through a mission restart

Once the game had let go of the fixture files, they could be written, and a mission restart brought the change to the processes that had read them[^round]:

- **Singleplayer `sp-cba`**: `fnc_version.sqf` and `fn_version.sqf` changed to `"2"`; the mission restart of a `@reset mission` check took 4.2 s; after it, `skdfx_cba_fnc_version` returned `"2"` on `p1`, the same process. `skdfx_fnc_version` still returned `"1"`, because the Functions Library copies an addon's function compiled at the game start; `["skdfx_fnc_version"] call BIS_fnc_recompile` then returned `true` and the function `"2"`, since the session mission sets `allowFunctionsRecompile = 1` and the function in `missionNamespace` is not final. That is `SPK-09`'s pass condition in singleplayer. With the file changed back to `"1"`, `BIS_fnc_recompile` took that the same way.
- **Dedicated `ded-cba`, two clients and a headless client**: `fnc_version.sqf` changed to `"2"`; the mission restart took 70 s from the order to the result; after it, `skdfx_cba_fnc_version` returned `"2"` on the server, which kept its process, and on `p1`, `p2` and `hc1`.

So `SPK-08`'s pass condition held once in singleplayer and once on a dedicated session: an edited `PREP` function live on every machine after a level-B reload, with CBA's cache off. It does not hold as a promise, because the file can be changed only once the game has let go of it, and after each mission restart the game holds it again. Of the ten reloads `NFR-PF-02` needs on the dedicated session, the first two found the file still held 90 s after the restart, and the next two still held after a flush by reading other files; the round stopped there at its bound, so the time of ten reloads is not measured, on the dedicated session or in singleplayer. The build round's ten mission restarts on the dedicated session (46.9 to 58.2 s, 48.6 s on average) are the restart's share of it.

## CBA's cache is off through the session mission

Every machine's RPT shows `[CBA] (xeh) INFO: ... CBA CACHE DISABLED? (Disable caching with cba_cache_disable.pbo): _compileType=compile` at each mission start, and `["compile"] call CBA_fnc_isRecompileEnabled` is `true`[^probe][^tests]. CBA 3.19.0's `CBA_fnc_isRecompileEnabled` reads `CfgSettings >> CBA >> caching` from the mission config as well as the game's, as research HR-4 expected, so `INT-CBA-02` holds and `cba_cache_disable.pbo` is not needed. With the cache off, `CBA_fnc_compileFunction` compiles the file again at each mission start with `compileFinal preprocessFileLineNumbers`, which reads through the game's file server and so gets the held content while the game holds the file[^cba].

## Approaches tried

| Approach | Test | Result | What it would change |
|---|---|---|---|
| Write the held file another way | share modes probed with `CreateFileW` | only reading is shared; write and delete fail with error 32 | nothing: no write gets past the hold |
| Point the path at other content | the probe's link re-pointed, the path spelled five other ways | the held content, or not found | nothing |
| Wait for the release | idle for 90 s to over 4 minutes, 4 times | not released | nothing |
| Lookups of missing paths | about 7800 in singleplayer, 3000 per machine on the dedicated session | released in singleplayer within 140 s, not on the dedicated session | nothing to rely on |
| Reading other loose files | 3000 or more per machine, 5 times | released 3 times in 5 | a reload step that makes each machine read files the framework ships, with no promise that it works |
| Mission restart on a released file | as above | the new content on every machine, kept processes included | `RLD-02` level `mission-restart` as specified |
| `BIS_fnc_recompile` on a released file | singleplayer | the new content mid-mission | `RLD-02` level `mission` for a `CfgFunctions` function, as specified |
| ACE3's layout: mod folder and file patching root as one folder, through two links | not run: the hour of game runs went to the release | | |
| Sending the file's text, as Neefay's harness does, and setting a final function at a mission restart before CBA compiles it | not run, for the same reason; it needs code in the framework's addon that runs before every mod's preInit, and a preprocessor for the mod's `#include` and macros | | |

## What follows for the spec

`SPK-08`'s fallback (load `cba_cache_disable.pbo`) does not help. A mission restart takes a changed file, but only once the game lets go of the file, and the round found no way to make it let go. What would follow:

- **Every `.sqf` change of a mod is `game-restart`.** Only a new process is sure to read a changed file, so `RLD-02`'s `mission` and `mission-restart` rows, `P-08`'s level A and level B as reloads, `US-054` and `US-055` change, and the time a mod developer waits is a game start (25 s in singleplayer, about 50 s for a dedicated session with two clients and a headless client on this machine). The file must be changed while no game process holds it, so the change and the reload are one step: the reload stops the processes, the file is changed, the processes start. `FIX-03` and the fixture contract, which let a test change an installed file while a session runs, change with it.
- **The levels stay, and a reload waits for the files.** A reload of level `mission` or `mission-restart` waits, with a ceiling, until no game process holds the changed files, and falls back to `game-restart` at the ceiling. A developer's editor cannot save a held file, so the framework would also stage the mod's files: the game reads a copy, and the reload copies the changed files over it once they are free. On this round's numbers the wait often reaches any useful ceiling, so most reloads would end as game restarts.
- **The framework stages the mod's files, and every reload is a game restart.** The developer's own files are never held, and the reload makes a new copy before it restarts the processes.

All three change what the spec promises beyond `m06.md`'s fallback, which the owner decides.

## Why the cited setups reload and the spike did not

Research `13-hot-reload.md` built the reload levels on three sources. Read again for what each actually does:

- **ACE3** sets up file patching as the spike did: a junction under the Arma 3 folder at the main prefix (`mklink /J "[Arma 3]\z\ace" ".hemttout/dev"`), a `hemtt dev` build, and `-filePatching` with `-debug`, `-showScriptErrors`, `-skipIntro` and `-noSplash`; its guide says file patching lets a developer "change the files in an addon while the game is running, requiring only a restart of the mission"[^ace]. The guide does not say how the file is written while the game holds it, and no ACE3, CBA_A3 or HEMTT issue reports the hold or a way round it[^web].
- **HEMTT** `hemtt dev` links each addon folder of `.hemttout/dev/addons` back to the project's addon folder, and links `<Arma 3>\<mainprefix>\<prefix>` to the build folder (to the project root since 1.19, issue 1256)[^hemtt]. The game therefore reads the developer's own files through two links; the fixtures' install reads a copy through one junction. Both are links the game follows to the same kind of loose file.
- **`Neefay/arma3-llm-harness`** does not reload through the game's files at all. Its server config sets `allowedFilePatching = 0`. Its `reload` reads the changed file of the mission's source folder on the host, preprocesses it there, sends the text to the game and runs `compile` on it, assigning the result to the function's variable; the staged copy of the mission sets `allowFunctionsRecompile` so mission functions can be assigned[^neefay]. It reloads mission functions, never an addon's, and never a final one.

Neefay's reload works because the game's file server never reads the changed file, and it does not reach a final function. ACE3's claim agrees with what this round found once a file is released: a mission restart takes the change. Whether an ACE3 developer's editor can save while the game holds the file, or whether the hold is new to a recent version of the game, the sources do not say.

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

## The game-restart reload, measured

After the owner chose a game restart for every `.sqf` change, made in one step with the mod's `rebuild` while no game process runs (`RLD-02`, `RLD-06`), the reload of a changed `PREP` function was timed on the setup of the M6 demo, a dedicated server with two clients and no headless client, loading CBA and both fixtures[^restart]:

- Ten reloads, each of a new value staged in a copy of `@skd_fixture_cba`'s sources and copied into the installed copy by the mod's `rebuild` (about 0.2 s), took 80.3 to 82.7 s by the reload result's `duration_s`, 81.8 s on average. After each, `skdfx_cba_fnc_version` returned the new value on the server, `p1` and `p2`, each a new start. The session's own start in that run took 82.4 s.
- Each reload is a session start: the processes ended within about 0.8 s, the server reached `ready` about 32 s after its launch, and the clients, launched after it, about 47 s after theirs.
- In singleplayer (`sp-cba`) the reload took 44.6 s, as long as that run's session start (45.2 s).

These starts are slower than the build round's (46 s for a dedicated session with two clients and a headless client, 25 s in singleplayer) on the same machine and the same game version; nothing in the reload adds to a start, so the difference is the machine's on that day.
