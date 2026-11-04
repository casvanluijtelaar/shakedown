---
type: User Story
title: "Start a singleplayer session once and run many checks against it"
description: "A test author starts one singleplayer session and runs many checks and runs against it without the game restarting."
status: draft
id: US-001
role: test-author
kind: feature
requirements: [SES-01, SES-05, SES-09, CFG-01]
---

# US-001 Start a singleplayer session once and run many checks against it

As a test author, I want to start a singleplayer session once and run many checks against it, so that a check's verdict costs seconds instead of a game start.

## Acceptance criteria

1. `shakedown session start --setup sp` returns with exit code 0 once the session is `ready`, and `status` shows the session `ready` with one machine, `p1`, `ready`.
2. Two runs of three checks each, one after the other against that session, write two result files in which `session.id` is the same and `p1` has `start` 1 in both.
3. The MCP tool `session_start` with `{"setup": "sp"}` gives the same session shape as `shakedown session start --setup sp --json`.

## Next to

- [US-003](US-003-start-refused-missing-environment.md) (system fail): the start is refused because Steam, the game or a mod is missing.
- [US-004](US-004-start-never-ready.md) (system fail): the start never becomes ready.
