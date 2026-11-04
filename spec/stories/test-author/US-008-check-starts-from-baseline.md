---
type: User Story
title: "Every check starts from the session's start state"
description: "Before every check the framework resets the session, so a check sees the session's start state on every machine whatever the previous check created or changed."
status: draft
id: US-008
role: test-author
kind: feature
requirements: [RST-01, RST-02, SCN-04, NFR-RL-02, NFR-PF-04]
---

# US-008 Every check starts from the session's start state

As a test author, I want to have every check start from the same state, whatever the check before it did, so that one check cannot make the next one pass or fail.

## Acceptance criteria

1. A check that creates objects, units and vehicles with `skd_fnc_create` and with its own `createVehicle` on a client, moves and damages `p1`, puts it in a vehicle and changes the date and weather is followed in the same run by a check that asserts none of those objects exists by net id, `p1` is at its start position, direction and loadout with no damage and on foot, and the date and weather are the session's start values on every machine; the second check passes.
2. In a singleplayer session and in a dedicated session with two clients, 20 such pairs in one run all pass, and every machine in the result file keeps `start` 1.
3. `shakedown reset` on a dedicated session with two clients and one headless client, after such a check, reports state `done` with a duration under 2 s.

## Next to

- [US-009](US-009-previous-check-killed-player.md) (error): the previous check killed the player.
- [US-010](US-010-reset-cannot-complete.md) (system fail): the reset cannot complete.
