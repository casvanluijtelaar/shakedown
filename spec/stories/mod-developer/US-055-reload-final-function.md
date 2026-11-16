---
type: User Story
title: "Reload a change to a final function by a mission restart"
description: "A change to a function that is final, such as a CBA `PREP` function, is reloaded in one step, the game made to let go of the file, the change made and the mission restarted, and the reload says why it needed a mission restart."
status: stable
id: US-055
role: mod-developer
kind: feature
requirements: [RLD-02, RLD-03, RLD-06, RLD-07, RST-02, FIX-03, NFR-PF-02, NFR-RL-04]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-055 Reload a change to a final function by a mission restart

As a mod developer, I want a change to a final function reloaded in one step and told why it needed a mission restart, so that I neither stop and start the session myself nor edit a file the game holds.

## Acceptance criteria

1. With `@skd_fixture_cba` watched in a staged copy of its sources and a `rebuild` that copies its `fnc_version.sqf` into the installed copy (`FIX-03`), after the staged file is changed from `"1"` to `"2"`, `shakedown reload` reports level `mission-restart`, the file with the reason that the game is made to let go of the file and a mission restart compiles it again, and that the mod's rebuild runs, and the time taken.
2. Then `skdfx_cba_fnc_version` returns `"2"` on every machine, the session is `ready`, the next check passes from the start state, and in the next result file the lead keeps its start number while every other machine has a start number one higher.

## Next to

- [US-057](US-057-reload-fails.md) (system fail): the reload does not complete.
