---
type: User Story
title: "A reload that does not complete"
description: "When a reload does not reach every machine, it fails naming the step and the machine, the session is broken, and the checks after it are invalid rather than run on mixed code."
status: stable
id: US-057
role: mod-developer
kind: system fail
requirements: [RLD-03, DM-05, P-08, NFR-RL-01, INS-04]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-057 A reload that does not complete

As a mod developer, I want to be told which machine did not take my change when a reload fails, so that I never read a verdict from a session running mixed code.

## Acceptance criteria

1. When `p2`'s process is ended during a `game-restart` reload of a changed `.sqf`, `reload` exits 4 with state `failed`, naming the step and `p2`; `status` shows the session `broken` with that reason.
2. A run asked after it is refused with exit code 4, saying the session is broken and naming `shakedown reload`, so no check reaches a verdict on mixed code; then a `game-restart` reload brings every machine back `ready` with a new start number.

## Next to

- [US-054](US-054-reload-changed-function.md) (feature): reload a changed function.
- [US-055](US-055-reload-final-function.md) (feature): reload a change to a final function.
