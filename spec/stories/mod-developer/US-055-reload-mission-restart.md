---
type: User Story
title: "Reload a change that needs a mission restart"
description: "A change to a function that is final, such as a CBA `PREP` function, is reloaded by a mission restart within the running session, and the reload says why it needed one."
status: draft
id: US-055
role: mod-developer
kind: feature
requirements: [RLD-02, RLD-03, RST-02, NFR-PF-02, NFR-RL-04]
---

# US-055 Reload a change that needs a mission restart

As a mod developer, I want to have a change that cannot reload in place reloaded by a mission restart within the running session, told why, so that I still avoid a full game restart.

## Acceptance criteria

1. After the installed fixture file `fnc_version.sqf` of `@skd_fixture_cba` is changed from `"1"` to `"2"`, `shakedown reload` reports level `mission-restart`, the file with the reason that the function is final, and the time taken.
2. Then `skdfx_cba_fnc_version` returns `"2"` on every machine, the session is `ready`, the next check passes from the start state, and on a dedicated session the clients and headless clients have a start number one higher in the next result file.

## Next to

- [US-057](US-057-reload-fails.md) (system fail): the reload does not complete.
