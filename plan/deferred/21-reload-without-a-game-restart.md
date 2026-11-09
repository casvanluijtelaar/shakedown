---
type: Deferred
title: "A changed mod file reaches a running game only through a game restart"
description: "The game holds every loose mod file it has read, shared for reading only, until a time nothing tried controls, so every .sqf change reloads by a game restart; what makes the game let go of a file, and the two ways round the hold that research 27 did not run, are left to find."
id: DEF-21
state: open
milestone: null
status: stable
---

# A changed mod file reaches a running game only through a game restart

## Summary

Spike `SPK-08` found that Arma 3 2.22 opens every loose mod file it reads through file patching shared for reading only, so while a game process holds it the file can be neither written nor replaced, and every machine reads the content it holds, across a mission restart, until it lets go of the file. Once it lets go, a mission restart takes the change on every machine, kept processes included, and `BIS_fnc_recompile` takes it mid-mission in singleplayer (research `27-spikes-reload.md`). The round found no wait, flush or setting that makes the game let go every time.

## Current behaviour

Every change of mod code is level `game-restart` (`RLD-02`), made in one step: the reload stops every game process, runs the mod's `rebuild` while none holds a file, and starts them again (`P-08`, `RLD-06`). The time a mod developer waits is a game start, `NFR-PF-02`. A check's change still reloads at level none, under 30 s (`NFR-PF-01`). `@reset mission` keeps the mission restart (`RST-06`).

## The gap

G2's loop for mod code (PRD `CR-03`: "inside the mission, by a mission restart within the session, or by a game restart") has only its last level. A dedicated session with two clients waits about a game start for every change of mod code, instead of a recompile in place or the mission restart of `SPK-08`'s plan.

## Options

1. **Find what makes the game let go of a file.** The round saw a release follow reading thousands of other loose files more often than not (3 of 5 times), and once in singleplayer after a mission restart and lookups of missing paths, but never on a schedule. A probe of the game's file server cache (`FileServer MaxCacheSize` in the RPT), its handle count over time, and the engine commands that touch the file server would say whether a release can be forced. If it can, the levels `mission` and `mission-restart` of research `13-hot-reload.md` come back as `SPK-08` planned them.
2. **ACE3's layout.** ACE3's guide sets the mod folder and the file patching root as one folder, reached through two links (`hemtt dev` links `.hemttout/dev/addons` back to the project and the game folder to the build). Research 27 ran the fixtures through one junction to a copy and did not run this layout. If the game lets go of files read through it, or the developer's editor can save through it, the levels come back for mods built that way.
3. **Staging plus text injection at a mission restart.** As Neefay's harness does for mission functions, the framework would read the changed file on the host, preprocess it there (the mod's `#include` and macros), and send the text; the framework's addon would set the function before every mod's `preInit` compiles it at a mission restart, so the game's file server never reads the changed file. It needs a preprocessor in the framework and code that runs before CBA's `PREP`, and reaches a final function only at a mission restart.

## Recommendation

Option 1 first, as a bounded spike: it is the cheapest and, if it works, it restores the planned levels without new machinery. Option 2 next, since it costs one game run. Option 3 only if both fail and the game restart's time proves too slow in use.

## Acceptance criteria

- A spike in `research/` says, for each option it ran, whether a changed `.sqf` of `@skd_fixture_cba` reaches every machine of a dedicated session with two clients without a game restart, 10 of 10.
- If one does, `RLD-02`, `P-08`, `NFR-PF-01`, `NFR-PF-02`, `US-054`, `US-055` and `FIX-03` are edited to the level it allows, and M7 is planned against it.
