---
type: User Story
title: "A client that hangs in its start fails it, and the reason is kept"
description: "A game process that runs but never connects fails the start at the start ceiling, naming that machine, with every process stopped, and the reason of every failed start is kept in the session folder."
status: draft
id: US-087
role: test-author
kind: system fail
requirements: [SES-10, SES-04, INS-02]
---

# US-087 A client that hangs in its start fails it, and the reason is kept

As a test author, I want a hung start to fail naming the machine that hung and to leave its reason behind, so that the machine is free again and I do not lose why it failed.

## Acceptance criteria

1. When `p2`'s process is suspended right after its launch, `session start` ends `failed` at the start ceiling, naming `p2`, its state `launched` and its last RPT lines, and stops every process it started.
2. After that start, and after `session stop` clears it from `status`, the session folder's `session.json` holds the same reason.

## Next to

- [US-004](US-004-start-never-ready.md) (system fail): a start that never becomes ready.
