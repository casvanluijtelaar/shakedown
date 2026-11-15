---
type: Research
title: "Spike SPK-19: making the game let go of a changed file"
description: "What makes Arma 3 let go of a loose mod file it holds - reading a few hundred MiB of other loose files through file patching, which the framework can make every machine do - and whether a changed .sqf of @skd_fixture_cba then reaches every machine of a dedicated session with two clients without a game restart, 10 of 10, with the time it takes, measured on the test machine beside M11."
status: draft
generated: { by: "claude-code/developer", at: 2026-11-15T07:31:00-08:00 }
sources:
  - { id: round, resource: "the research round of 2026-11-15 on the Windows test machine, Arma 3 2.22 stable, CBA_A3 3.19.0 (workshop 450814997), through the installed shakedown 0.1.1 (main at 810e2f7): a dedicated session with two clients loading CBA and both fixtures, every machine made to loadFile distinct 8 MiB loose files, and one 8 MiB file through many junctions, under <game>\\skd_spk19, the hold tested by opening the installed fnc_version.sqf for writing, the file changed and the mission restarted with a @reset mission check, and skdfx_cba_fnc_version called on every machine", read: 2026-11-15 }
  - { id: rpt, resource: "the RPT of every machine of the round's sessions, the line FileServer MaxCacheSize", read: 2026-11-15 }
  - { id: r27, resource: "research/27-spikes-reload.md, the hold, the release table and the approaches tried in M6", read: 2026-11-15 }
---

# Spike SPK-19: making the game let go of a changed file

Draft: the 10 of 10 loop is running.

## What makes the game let go

After a session start of `ded2-cba` (server, `p1`, `p2`; CBA and both fixtures), every machine held `fnc_version.sqf` of `@skd_fixture_cba`, as research 27 found[^r27]. Each machine then read distinct 8 MiB loose files with `loadFile`, all three at once[^round]:

| Read by each machine, in total | Time | Held after |
|---|---|---|
| 256 MiB (32 files) | 3.1 s | yes |
| 512 MiB (64 files) | 3.1 s more | no |

After a mission restart, which made the new clients and the kept server hold the file again, one 8 MiB file read through 50 junctions (400 MiB) left it held, and through 100 junctions (800 MiB) freed it, in 5.2 s and 5.1 s[^round]. The game keeps a file by its path: the same file through another path counts as another file. Every machine's RPT logs `FileServer MaxCacheSize = 2048 MB`[^rpt], so the release comes well before that size; what decides it is the bytes read through file patching, between 256 and 512 MiB, not the count of files, which is why research 27's reads of 3000 small files freed the file only some of the time[^r27].
