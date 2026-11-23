---
type: Deferred
title: "A changed mod file reaches a running game only through a game restart"
description: "The game holds every loose mod file it has read, shared for reading only, until it has read a few hundred MiB of other loose files, so every .sqf change reloads by a game restart; spike SPK-19 found that the framework can make every machine let go of the file and a mission restart then takes the change, which M14 builds."
id: DEF-21
state: done
milestone: M14
status: stable
---

# A changed mod file reaches a running game only through a game restart

## Summary

Spike `SPK-08` found that Arma 3 2.22 opens every loose mod file it reads through file patching shared for reading only. While a game process holds the file, it can be neither written nor replaced, and every machine reads the content it holds, across a mission restart, until it lets go of the file. Once it lets go, a mission restart takes the change on every machine, kept processes included (research `27-spikes-reload.md`). Spike `SPK-19` found what makes it let go: reading between 256 and 512 MiB of other loose files through file patching. The game keeps files by path, so one 8 MiB file reached through 80 junctions frees it. A changed `PREP` function of `@skd_fixture_cba` reached every machine through a mission restart, 10 of 10 on a dedicated session with two clients and 10 of 10 in singleplayer (research `31-spike-reload-release.md`).

## Current behaviour

Every change of mod code is level `game-restart` (`RLD-02`), made in one step: the reload stops every game process, runs the mod's `rebuild` while none holds a file, and starts them again (`P-08`, `RLD-06`). The time a mod developer waits is a game start, `NFR-PF-02`. A check's change still reloads at level none, under 30 s (`NFR-PF-01`). `@reset mission` keeps the mission restart (`RST-06`).

## The gap

G2's loop for mod code (PRD `CR-03`: "inside the mission, by a mission restart within the session, or by a game restart") has only its last level. Each change of mod code costs a game start: about 27 s in singleplayer and 50 to 82 s on a dedicated session with two clients on the test machine.

## Options

1. **Make the game let go of the file, then restart the mission.** `SPK-19` ran it and it passed. Every machine reads the framework's release files (640 MiB through 80 junction paths, 5 to 10 s), the runner checks the changed files are free, runs the mod's `rebuild`, and restarts the mission. In singleplayer a reload took 9.7 to 14.3 s. On a dedicated session it took 54.4 to 57.7 s, as long as a game restart, because the mission restart starts new clients. It keeps the server's process.
2. **ACE3's layout**, the mod folder and the file patching root as one folder reached through two links. Not run, since option 1 passed.
3. **Staging and text injection at a mission restart.** The framework would read and preprocess the changed file on the host and set the function before every mod's `preInit` compiles it. Option 1 made this unneeded: it existed only to get past the hold, and it needs the same mission restart for a final function, so it would cost the same client start on a dedicated session.

## Recommendation

Option 1, built by M14. It reaches the level `mission-restart` that `CR-03` asks for, in both modes, with no new machinery beyond a folder of junctions and one step before the mission restart. The dedicated session's remaining distance from 30 s is `DEF-32`.

## Acceptance criteria

- A spike in `research/` says, for each option it ran, whether a changed `.sqf` of `@skd_fixture_cba` reaches every machine of a dedicated session with two clients without a game restart, 10 of 10. Done: research `31-spike-reload-release.md`, option 1, 10 of 10.
- `RLD-02`, `P-08`, `NFR-PF-01`, `NFR-PF-02`, `US-054`, `US-055` and `FIX-03` are edited to the level `mission-restart`, and M14 builds against it.
