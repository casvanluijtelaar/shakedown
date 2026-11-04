---
type: User Story
title: "The action cannot be used"
description: "When the action a check asks for is missing, its condition is false or the player is out of its radius, the check fails at the ceiling saying which of the three it was, with the facts."
status: draft
id: US-020
role: test-author
kind: error
requirements: [ACT-01, NFR-US-01]
---

# US-020 The action cannot be used

As a test author, I want to be told why a player could not use an action, so that I can tell a wrong title from a player standing too far away or a mod that hides the action.

## Acceptance criteria

1. `["p1", _crate, "Open Crate"] call skd_fnc_useAction` ends `fail` / `assertion` with a message saying there is no action of that title and listing the titles on the target, including `Open crate`.
2. With `p1` placed 10 m from the crate, the call ends `fail` / `assertion` with a message giving the distance and the action's radius of 3 m; on an already opened crate, the message says the action's condition is false.
3. `["server", _crate, "Open crate"] call skd_fnc_useAction` on a dedicated session ends `fail` / `check-error` saying `server` has no player.

## Next to

- [US-019](US-019-use-mod-action.md) (feature): make a player use a mod's action.
