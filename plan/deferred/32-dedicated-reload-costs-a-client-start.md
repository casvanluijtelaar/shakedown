---
type: Deferred
title: "A reload of mod code on a dedicated session costs a client start"
description: "The mission restart that takes a changed .sqf on a dedicated session starts new clients, so the reload M14 builds takes about 55 s there, as long as a game restart, and G2's 30 s for mod code holds only in singleplayer."
id: DEF-32
state: open
milestone: null
status: draft
---

# A reload of mod code on a dedicated session costs a client start

## Summary

Spike `SPK-19` found that the framework can make every machine let go of a changed mod file, and that a mission restart then takes the change (research `31-spike-reload-release.md`). M14 builds that as the level `mission-restart`. In singleplayer the reload took about 11 s. On a dedicated session with two clients it took 54.4 to 57.7 s, while a session start in the same run took 49.8 s.

## Current behaviour

On a dedicated session the mission restart ends every client and headless client and starts new ones once the server's mission has restarted (`P-08` step 3). It does so because a client kept across a mission change was left in a mission it never loaded about one change in two (research `10-prior-harness.md`, `14-sessions.md`). So the reload of a changed `.sqf` pays a client start: about 16 s for the server's mission restart, then about 28 s for the new clients, plus 5 to 10 s to free the file.

## The gap

G2's loop of under 30 s from an edit to a verdict holds for mod code only in singleplayer. A mod developer whose checks need a dedicated session, as arma-grappling-hook's multiplayer checks do, waits about a game start for every change of mod code.

## Options

1. **Recompile in place.** On a freed file, `BIS_fnc_recompile` of an addon's `CfgFunctions` function mid-mission on every machine (`SPK-09`; research 27 saw it work in singleplayer). Every process is kept and no client starts. It does not reach a final function, such as a CBA `PREP` function, which CBA compiles with `compileFinal` at each mission start.
2. **Keep the clients across the mission restart.** Find a way for a client to follow the server's `#restart` into the new mission reliably, such as waiting for its role before the mission starts. That would cut the client start for every function, final ones included, but it is the case research 10 found unreliable.

## Recommendation

Leave it until arma-grappling-hook's use shows the dedicated reload's time matters more than the singleplayer loop M14 gives. Then run option 1 first, as `SPK-09`, if the mod's changed functions are mostly `CfgFunctions` functions, and option 2 if they are mostly `PREP` functions.

## Acceptance criteria

- A changed function of `@skd_fixture_cba` reaches `server`, `p1` and `p2` of a running dedicated session within 30 s of the reload order, 10 of 10, by a level `RLD-02` names.
- `NFR-PF-02` and `US-081` carry that time for a dedicated session.
