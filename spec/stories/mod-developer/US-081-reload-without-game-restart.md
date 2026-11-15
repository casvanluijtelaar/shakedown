---
type: User Story
title: "A changed function reaches a running session without a game restart"
description: "A mod developer changes one function of a mod built for file patching and reloads it, and every machine of a running session runs the new code after a mission restart instead of a game restart, within 30 s of the save in singleplayer."
status: draft
id: US-081
role: mod-developer
kind: feature
requirements: [RLD-01, RLD-02, RLD-03, RLD-06, NFR-PF-01, NFR-PF-02]
---

# US-081 A changed function reaches a running session without a game restart

As a mod developer, I want my change in the running game without waiting for a game start, so that the loop from an edit to a verdict stays under the half minute G2 sets wherever the game allows it.

## Acceptance criteria

1. With `@skd_fixture_cba` watched in a staged copy of its sources and a `rebuild` that copies its `fnc_version.sqf` into the installed copy (`FIX-03`), on a dedicated session with two clients, after the staged file is changed, `shakedown reload` reports level `mission-restart` with its reason, every machine reached, and the time taken; the server keeps its process id.
2. A check that calls the function on `server`, `p1` and `p2` right after sees the new value on all three, 10 of 10.
3. In singleplayer the same reload reports level `mission-restart`, `p1` keeps its process id, and from saving the file to the outcome of a check whose body takes under 10 s takes under 30 s.
4. On the dedicated session the reload takes no longer than `NFR-PF-02`'s target.

## Next to

- [US-082](US-082-game-holds-changed-file.md) (system fail): the game still holds the changed file.
- [US-054](US-054-reload-changed-function.md) (feature): reload a changed function.
