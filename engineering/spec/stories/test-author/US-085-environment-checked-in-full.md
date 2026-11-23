---
type: User Story
title: "The environment is checked in full before a start, every gap at once"
description: "Before any game process starts, the start checks the game, Steam running and logged in, every mod folder, the framework's addon, the ports and free disk, and names every gap it found at once."
status: draft
id: US-085
role: test-author
kind: feature
requirements: [ENV-01, SES-03, INT-STM-01]
---

# US-085 The environment is checked in full before a start, every gap at once

As a test author, I want every missing piece of my environment named in one go, so that I fix them all before the first start instead of one per failed attempt.

## Acceptance criteria

1. With Steam running but logged out, a mod folder missing and a port taken, `session start` exits 4 before any game process starts and names all three gaps.
2. With the drive of `.shakedown/` under 2 GB free, the start is refused naming the drive and the free space.
3. `session_start` through MCP returns the same gaps.

## Next to

- [US-086](US-086-mod-in-wrong-version.md) (error): a mod loaded in another version than the project names.
- [US-003](US-003-start-refused-missing-environment.md) (error): a start refused when Steam, the game or a mod is missing.
