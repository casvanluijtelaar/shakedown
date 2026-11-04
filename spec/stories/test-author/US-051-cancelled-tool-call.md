---
type: User Story
title: "A cancelled tool call stops the work it started in the game"
description: "When the AI agent's client cancels a tool call, the work it started in the game stops, so a dropped call does not leave code running in the session."
status: draft
id: US-051
role: test-author
kind: system fail
requirements: [AGT-04, INS-02]
---

# US-051 A cancelled tool call stops the work it started in the game

As a test author, I want to have a cancelled call stop what it started in the game, so that a call my agent gave up on does not keep changing the session behind it.

## Acceptance criteria

1. An `exec` of code that waits forever, cancelled by the client after 5 s, leaves no script of it in `inspect`'s `scripts` of that machine within 10 s, and the session stays `ready`.
2. A `wait_until` cancelled by the client stops polling on its machine, and the next tool call is answered normally.

## Next to

- [US-050](US-050-long-run-through-tools.md) (feature): an AI agent follows a long run.
