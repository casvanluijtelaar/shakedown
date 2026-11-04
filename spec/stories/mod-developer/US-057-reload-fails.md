---
type: User Story
title: "A reload that does not complete"
description: "When a reload does not reach every machine, it fails naming the step and the machine, the session is broken, and the checks after it are invalid rather than run on mixed code."
status: draft
id: US-057
role: mod-developer
kind: system fail
requirements: [RLD-03, DM-05, P-08, NFR-RL-01]
---

# US-057 A reload that does not complete

As a mod developer, I want to be told which machine did not take my change when a reload fails, so that I never read a verdict from a session running mixed code.

## Acceptance criteria

1. When `p2`'s process is ended during a `mission-restart` reload, `reload` exits 4 with state `failed`, naming the step and `p2`; `status` shows the session `broken` with that reason.
2. A check run after it ends `invalid` / `session-broken`, and a `game-restart` reload brings every machine back `ready` with a new start number.

## Next to

- [US-054](US-054-reload-inside-mission.md) (feature): reload inside the mission.
- [US-055](US-055-reload-mission-restart.md) (feature): reload by a mission restart.
