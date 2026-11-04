---
type: User Story
title: "The previous check killed the player"
description: "A check that ends with a player dead is followed by a check in which that player is alive at the start position, in singleplayer and on a dedicated session."
status: draft
id: US-009
role: test-author
kind: error
requirements: [RST-05, RST-01]
---

# US-009 The previous check killed the player

As a test author, I want to have a player that died in one check be alive at the start of the next, so that a check of a deadly situation does not break every check after it.

## Acceptance criteria

1. After a check that sets `p1`'s damage to 1 in a singleplayer session, the next check's `["p1", {alive player}, "==", true, "p1 alive"] call skd_fnc_expectOn` passes and the session stays `ready`.
2. On a dedicated session with two clients, after a check that kills `p2`, the next check sees `p2` alive at its start position on the server, and `p2` keeps `start` 1 in the result file.

## Next to

- [US-008](US-008-check-starts-from-baseline.md) (feature): every check starts from the session's start state.
