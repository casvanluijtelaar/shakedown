---
type: User Story
title: "A changed function reaches a running dedicated session without a game restart"
description: "A mod developer changes one function of a mod built for file patching and reloads it, and every machine of a running dedicated session runs the new code without a game restart."
status: draft
id: US-081
role: mod-developer
kind: feature
requirements: [RLD-01, RLD-02, RLD-03, NFR-PF-01]
---

# US-081 A changed function reaches a running dedicated session without a game restart

As a mod developer, I want my change in the running game without waiting for a game start, so that the loop from an edit to a verdict stays under the half minute G2 sets.

## Acceptance criteria

1. On a dedicated session with two clients, after a change to one `PREP` function of `@skd_fixture_cba`, `shakedown reload` reports a level below `game-restart` with its reason, and every machine keeps its process id.
2. A check that calls the function on `server`, `p1` and `p2` right after sees the new value on all three, 10 of 10.
3. From saving the file to the check's outcome takes under 30 s for a check whose body takes under 10 s.

## Next to

- [US-082](US-082-game-holds-changed-file.md) (system fail): the game still holds the changed file.
- [US-054](US-054-reload-changed-function.md) (feature): reload a changed function.
