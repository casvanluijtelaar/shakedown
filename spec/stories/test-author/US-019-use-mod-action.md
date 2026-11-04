---
type: User Story
title: "Make a player use a mod's action and check the result on every machine"
description: "A test author makes the player of a named machine use an action the mod added to an object, through the mod's real path, and asserts the outcome on the server and on every client."
status: draft
id: US-019
role: test-author
kind: feature
requirements: [ACT-01, OBS-06, CTL-01]
---

# US-019 Make a player use a mod's action and check the result on every machine

As a test author, I want to make a player open a crate through the mod's own action and assert that the item lands in the player's inventory on the server and on both clients, so that the mod is tested the way a player uses it.

## Acceptance criteria

1. On a dedicated session with two clients, a check that creates the fixture's `skdfx_Crate` 2 m in front of `p1` and calls `["p1", _crate, "Open crate"] call skd_fnc_useAction` gets `true`, and then `skd_fnc_expectEach` on `["server", "p1", "p2"]` that `p1` holds one more `FirstAidKit` passes.
2. The same check passes in a singleplayer session, and the action's title is matched after its structured-text tags are removed.

## Next to

- [US-020](US-020-action-cannot-be-used.md) (error): the action cannot be used.
