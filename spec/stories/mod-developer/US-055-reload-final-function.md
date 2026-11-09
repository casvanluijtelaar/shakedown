---
type: User Story
title: "Reload a change to a final function by a game restart"
description: "A change to a function that is final, such as a CBA `PREP` function, is reloaded in one step, the processes stopped, the change made and the processes started, and the reload says why it needed a game restart."
status: stable
id: US-055
role: mod-developer
kind: feature
requirements: [RLD-02, RLD-03, RLD-06, RST-02, FIX-03, NFR-PF-02, NFR-RL-04]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-055 Reload a change to a final function by a game restart

As a mod developer, I want a change to a final function reloaded in one step and told why it needed a game restart, so that I neither stop and start the session myself nor edit a file the game holds.

## Acceptance criteria

1. With `@skd_fixture_cba` watched in a staged copy of its sources and a `rebuild` that copies its `fnc_version.sqf` into the installed copy (`FIX-03`), after the staged file is changed from `"1"` to `"2"`, `shakedown reload` reports level `game-restart`, the file with the reason that the game reads a changed file only in a new process and that the mod's rebuild runs, and the time taken.
2. Then `skdfx_cba_fnc_version` returns `"2"` on every machine, the session is `ready`, the next check passes from the start state, and every machine has a start number one higher in the next result file.

## Next to

- [US-057](US-057-reload-fails.md) (system fail): the reload does not complete.
