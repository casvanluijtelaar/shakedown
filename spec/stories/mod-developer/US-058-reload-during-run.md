---
type: User Story
title: "A reload asked while a run is active"
description: "A reload or a reset asked while a run, a reload or a reset is in progress is refused, saying what is in progress, and the run goes on untouched."
status: stable
id: US-058
role: mod-developer
kind: mistake
requirements: [RLD-01, RST-03, NFR-US-01]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-058 A reload asked while a run is active

As a mod developer, I want to be told that a run is in progress when I ask for a reload in the middle of it, so that I never change the code under a check that is running.

## Acceptance criteria

1. `shakedown reload` during a run exits 4 with a message naming the active run and its progress and saying to wait for it or stop it with `shakedown run stop`; the MCP tool `reload` returns `isError` with code `session-busy`.
2. The run's checks end with the same outcomes as in a run with no reload asked, and no machine's start number changes.

## Next to

- [US-054](US-054-reload-inside-mission.md) (feature): reload inside the mission.
