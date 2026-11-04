---
type: User Story
title: "The runner dies"
description: "When the runner process is killed, every game process it started ends within seconds, and the next start finds no process, port or file left from it."
status: draft
id: US-035
role: test-author
kind: system fail
requirements: [STP-01, STP-03, STP-04, NFR-RL-03]
---

# US-035 The runner dies

As a test author, I want to have a crash or kill of the framework itself take the game down with it and leave nothing behind, so that the next run starts clean.

## Acceptance criteria

1. After the runner is killed with `taskkill /F` during a run on a dedicated session, no `arma3_x64` or `arma3server_x64` process of that session runs within 10 s, in 5 of 5 tries.
2. The next `shakedown status` starts a new runner and shows no session; the next `session start` of the same setup succeeds, and no mission copy of the dead session is left in the game's `MPMissions` folder.

## Next to

- [US-007](US-007-session-outlives-client.md) (feature): the session outlives its client.
- [US-036](US-036-stop-session.md) (feature): stopping a session.
