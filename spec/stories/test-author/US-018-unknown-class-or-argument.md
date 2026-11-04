---
type: User Story
title: "An unknown class name or a wrong argument to the check API"
description: "A check that creates an unknown class or calls a check API function with an argument of the wrong type or out of range ends as the author's mistake, naming the function, the argument and what it expected."
status: draft
id: US-018
role: test-author
kind: mistake
requirements: [SCN-01, DM-04, API-03]
---

# US-018 An unknown class name or a wrong argument to the check API

As a test author, I want to be told which function and which argument I got wrong, so that a typo in a class name is not mistaken for a defect in the mod.

## Acceptance criteria

1. `["B_Soldier_Typo", _pos] call skd_fnc_create` ends the check `fail` / `check-error` with a message naming the class and saying it is not in `CfgVehicles`.
2. `["p1", "here", 0] call skd_fnc_place` ends the check `fail` / `check-error` naming `skd_fnc_place`, the argument `position`, and the type it expects.
3. In both cases the failed step's path is in `evidence.failed_step`, and the next check of the run executes normally.

## Next to

- [US-017](US-017-build-a-scene.md) (feature): build a scene.
