---
type: Deferred
title: "The server can log Object not found for a dead player's body the reset deletes while it still falls"
description: "On a dedicated session the client a dead player's body is local to deletes it in the reset while the body's ragdoll still moves, and about once in 40 kills the server then logs Server: Object n:nnn not found with a message number that varies, naming that body."
id: DEF-25
state: open
milestone: null
status: stable
---

# The server can log Object not found for a dead player's body the reset deletes while it still falls

## Summary

`RST-05` has the reset delete the body a dead player left on a dedicated session. That body is local to the client whose player died, so under `DEF-09`'s rule that client deletes it. About once in 40 kills the server then logs `Server: Object n:nnn not found (message Type_nnn)` for that body, in the reset or just after it. No check's outcome changes. This is the line `DEF-09` removed for the objects a check creates; for a body it remains.

## Current behaviour

Measured with an installed framework at `85a7194` (finding `m09-3`), with a check that kills `p2` with `setDamage 1`, waits for its respawn and ends, and an `EntityRespawned`-recorded body with `Deleted` and `Local` handlers on every machine:

- The body stays local to `p2` until the reset deletes it, and `p2` alone deletes it; no locality change was seen. The first body of a session is the mission's own unit, made by the server (`2:nn`); each later one is the unit the respawn made on `p2` (`p2`'s own id).
- When the reset deletes it, 1.5 to 2 s after the kill, the body's ragdoll is still awake: the server's `Deleted` handler saw `isAwake` true in 38 of the 39 deletions of the 40-kill session below.
- With the fixture alone: 0 lines over 26 kills in one session and 6 first kills in six sessions. With CBA, the fixture and arma-grappling-hook, as the acceptance `ded` setup loads: 1 line over 40 kills in one session, `Server: Object 6:49 not found (message Type_467)`, logged in the reset of the check that killed `p2` and made the body `6:49`, just before the server's own `Deleted` handler for `6:49` ran. The acceptance run of `m09-3` logged `Object 2:89 not found (message Type_143)` in the same place, for a body the server made.

The likeliest cause, not proven: the owning client deletes a body whose ragdoll still sends updates, and an update sent before the deletion reaches the server after it. The message number changes from line to line (`Type_143`, `Type_467`), as an update's would, and the rate rises with the mods that add network traffic. `DEF-09`'s line (`Type_189`) came from a machine that did not own the object; this one comes from the owner.

## The gap

Noise in the server's RPT that a person or an AI agent reading it after a failure has to recognise as harmless, after any check in which a player died on a dedicated session.

## Options

1. Let the body come to rest before deleting it: the reset hides the body and stops its simulation (`hideObjectGlobal`, `enableSimulationGlobal false` on the server) in `free`, and deletes it in `verify` once a frame has passed on its owner. Costs a frame or two of the reset.
2. Delete the body on the death, not in the reset: the `EntityRespawned` handler on the owning client deletes the old body once its ragdoll is at rest. Takes the body out of the check, which some checks may want to look at.
3. Leave it, and say in `docs/` that this line after a death is harmless. Conceals nothing, fixes nothing.

## Recommendation

The first, measured as `DEF-09` was: it keeps the deletion in the reset and on the owner, where `DEF-09` put it. It belongs with the reset's work on the death of a player (`RST-05`, `DEF-14`, `DEF-22`), not with M9, whose stories do not cover the reset of a dead player.

## Acceptance criteria

1. On a dedicated session with two clients and a headless client loading CBA, the fixture and arma-grappling-hook, 100 checks that each kill `p2` leave no `Object ... not found` line in the server's RPT.
2. The reset after each of them is `done`, and its duration rises by no more than 0.1 s.
3. No code in the framework exists only to drop this line.
