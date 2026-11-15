---
type: Research
title: "Spike SPK-19: making the game let go of a changed file"
description: "What makes Arma 3 let go of a loose mod file it holds - reading between 256 and 512 MiB of other loose files through file patching, which the framework can make every machine do with one 8 MiB file reached through many junctions - and that a changed .sqf of @skd_fixture_cba then reaches every machine of a dedicated session with two clients through a mission restart, 10 of 10, and of a singleplayer session, 10 of 10, with the time each reload takes against a game start, measured on the test machine beside M11."
status: draft
generated: { by: "claude-code/developer", at: 2026-11-15T07:45:00-08:00 }
sources:
  - { id: round, resource: "the research round of 2026-11-15 on the Windows test machine, Arma 3 2.22 stable, CBA_A3 3.19.0 (workshop 450814997), through the installed shakedown 0.1.1 (main at 810e2f7), setups ded2-cba (dedicated, two clients) and sp-cba, loading CBA and both fixtures: every machine made to loadFile, with one exec to all machines, distinct 8 MiB loose files and one 8 MiB file through many junctions under <game>\\skd_spk19, the hold tested by opening the installed fnc_version.sqf for writing, the file changed and the mission restarted by a @reset mission check, skdfx_cba_fnc_version called on every machine and each machine's process id read from status after each reload", read: 2026-11-15 }
  - { id: rpt, resource: "the RPT of every machine of the round's sessions, the line FileServer MaxCacheSize", read: 2026-11-15 }
  - { id: cba, resource: "https://github.com/CBATeam/CBA_A3 at tag v3.19.0, addons/xeh/fnc_compileFunction.sqf (read with gh)", read: 2026-11-15 }
  - { id: r27, resource: "research/27-spikes-reload.md: the hold, the release table, the approaches tried and the mission restart's time in M6", read: 2026-11-15 }
  - { id: r10, resource: "research/10-prior-harness.md and research/14-sessions.md: clients kept across a mission change", read: 2026-11-15 }
---

# Spike SPK-19: making the game let go of a changed file

`SPK-19` (`spec/06-open-questions.md`) asks whether a changed `.sqf` of `@skd_fixture_cba` can reach every machine of a dedicated session with two clients without a game restart, 10 of 10, by a way the framework can bring about every time. It ran as a research round beside M11 on the Windows test machine, about 17 minutes of game runs. **It passed** by `DEF-21`'s first option: the game can be made to let go of a file. Every machine reads a few hundred MiB of other loose files through file patching, the held file is then free to write, and a mission restart takes the change on every machine. ACE3's layout, `DEF-21`'s second option, was not run, since the first passed.

## What makes the game let go

Research 27 found that the game holds every loose file it read, shared for reading only, and lets go of it at a time no wait or flush it tried controlled. Reading thousands of other loose files freed it 3 times in 5[^r27]. This round measured bytes instead of file counts. After a session start of `ded2-cba` (server, `p1`, `p2`), every machine held `fnc_version.sqf`. Then each machine read distinct 8 MiB loose files with `loadFile`, all three at once, one step after the other[^round]:

| Read by each machine, in total | Time of the step | Held after |
|---|---|---|
| 256 MiB (32 files) | 3.1 s | yes |
| 512 MiB (64 files) | 3.1 s | no |
| 800 MiB (100 files) | 3.6 s | no |

After a mission restart the kept server and the new clients held the file again. One 8 MiB file read through 50 junctions (400 MiB) left it held. Through 100 junctions (800 MiB) it was freed. Each step took about 5 s[^round].

- **The game caches loose files by path.** The same file reached through another junction counts as another file, so the framework needs one 8 MiB file and a folder of junctions to it, not hundreds of MiB on disk.
- **The bytes decide, not the count.** The release comes between 256 and 512 MiB of other loose files read. Research 27's reads of thousands of small files only sometimes reached that, which is why its release looked random[^r27].
- **The size is not the logged cache size.** Every machine's RPT logs `FileServer MaxCacheSize = 2048 MB`[^rpt], so the limit that frees the file is lower than that line. The round did not look for the exact number. It used 640 MiB, 25 % above the size that freed the file in every trial.

## The pass condition, 10 of 10

Each of ten reloads ran in this order[^round]:

1. Every machine reads one 8 MiB file through 80 junctions (640 MiB), with one `exec` to all machines.
2. The installed `fnc_version.sqf` is checked free by opening it for writing.
3. The file is written with a new value, `"v1"` to `"v10"`.
4. A `@reset mission` check restarts the mission as `P-08` does.
5. `skdfx_cba_fnc_version` is called on every machine.

Up to three reads were allowed before giving up. Every reload needed one.

| Setup | Reloads | New value on every machine | Read (step 1) | Mission restart (step 4) | Both | Session start in the same run |
|---|---|---|---|---|---|---|
| `ded2-cba`: server, `p1`, `p2` | 10 | 10 of 10 | 7.5 to 9.9 s, 8.1 s on average | 46.9 to 48.2 s, 47.4 s | 54.4 to 57.7 s, 55.4 s | 49.8 s (55.0 s in the first run) |
| `sp-cba`: `p1` | 10 | 10 of 10 | 5.3 to 9.6 s, 6.1 s | 4.2 to 4.8 s, 4.6 s | 9.7 to 14.3 s, 10.7 s | 26.5 s |

The times are wall-clock times of the command line calls, the `run` of the restart check included. On the dedicated session the server kept its process through all ten reloads. Each mission restart started new client processes, as `P-08` step 3 does, so the read matters there only for the kept server and for freeing the file to write. In singleplayer `p1` kept its process through all ten.

## What the times mean

- **Singleplayer gains.** A changed `PREP` function is live in about 11 s, against a game start of about 27 s. The loop from a save to a verdict fits G2's 30 s for a check whose body takes a few seconds.
- **A dedicated session does not gain.** The reload takes about 55 s, as long as a session start, which research 27 found is as long as a `game-restart` reload[^r27]. The mission restart of a dedicated session ends the clients and starts new ones (`P-08` step 3), because a client kept across a mission change was left in a mission it never loaded about one change in two[^r10]. So the reload pays a client start either way. What it keeps is the server's process and its state outside the mission.
- **A final function needs the mission restart.** With CBA's recompile on, `CBA_fnc_compileFunction` compiles a `PREP` function with `compileFinal` at each mission start[^cba]. So no command can set the function in place mid-mission, and the lowest level that reaches it is the mission restart. A `CfgFunctions` function that is not final could be recompiled in place on a freed file with `BIS_fnc_recompile` (research 27, singleplayer). That would keep every process of a dedicated session, but it is `SPK-09`'s question and this round did not run it.

## What follows for the spec

- **A changed `.sqf` of a watched mod has the level `mission-restart`** (`CR-03`: reload at the lowest level), in singleplayer and on a dedicated session. The reload frees the files first by making every machine read the framework's release files, checks that no game process holds a changed file, runs the mod's `rebuild` while none does, and restarts the mission. A machine still holding a file after a second read makes the reload a `game-restart`, naming the file and the machine (`US-082`).
- **The release files** are one 8 MiB file and 80 junctions to its folder, in a folder of the game folder that no PBO covers. A file under a prefix a PBO covers is not found through file patching[^r27].
- **New and deleted files, config and the other kinds stay `game-restart`.** Nothing in this round changes why they need it.
- **Text injection (`DEF-21` option 3) is not needed.** It existed to get past the hold. It also needs a mission restart for a final function, so on a dedicated session it costs the same client start.
- **G2's 30 s on a dedicated session stays out of reach for mod code.** The reload keeps the server but costs a client start, about 55 s on this machine. Getting under 30 s needs clients kept across a mission restart, or an in-place recompile of functions that are not final (`SPK-09`).

## Left out

- `DEF-21` option 2, ACE3's layout: not run, since option 1 passed.
- The exact threshold between 256 and 512 MiB, and whether it depends on the machine's memory or a start parameter. The release files' 640 MiB is set from the trials, and the reload's check that the file is free, with its fallback to a game restart, covers a machine where it falls short.
- Memory and frame time after the read: not measured. The checks and mission restarts that followed it ran normally.
- A headless client: the pass condition's setup has none. It reads files through the same engine.
- `BIS_fnc_recompile` in place on a dedicated session (`SPK-09`).
