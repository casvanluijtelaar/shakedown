---
type: User Story
title: "Inspecting a machine that stopped answering"
description: "Asking a machine that stopped answering returns, within its ceiling, that the machine is lost, instead of hanging the person or AI agent who asked."
status: draft
id: US-045
role: test-author
kind: system fail
requirements: [INS-01, OBS-03, P-07]
---

# US-045 Inspecting a machine that stopped answering

As a test author, I want to be told within seconds that the machine I ask has stopped answering, so that my diagnosis does not hang on the same stall it is trying to explain.

## Acceptance criteria

1. With `p2`'s process suspended, `shakedown inspect --machine p2 --json` returns within 20 s with `machine.state` `lost`, and `status` shows `p2` `lost` with its last answer time and the session `broken`.
2. `shakedown logs --machine p2` still returns `p2`'s last lines, and `inspect` of every other machine still answers.

## Next to

- [US-044](US-044-ask-running-game.md) (feature): ask a running game what is going on.
