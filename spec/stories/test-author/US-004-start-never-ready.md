---
type: User Story
title: "A start that never becomes ready ends failed and leaves nothing running"
description: "A session start in which a machine exits or does not become ready within the start ceiling ends failed, names the machine and its last log lines, and stops every process it started."
status: stable
id: US-004
role: test-author
kind: system fail
requirements: [SES-04, STP-02, NFR-US-01, INS-04]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-004 A start that never becomes ready ends failed and leaves nothing running

As a test author, I want to have a start that hangs or loses a process end at a ceiling, with the machine that was not ready and its last log lines, so that a stuck start never holds the machine and I know where to look.

## Acceptance criteria

1. When a game process of the session is ended during the start, `session start` returns with state `failed` and exit code 4, and the session's `reason` names that machine, its state and the last 20 lines of its RPT.
2. A setup with `start_timeout = 30` whose session is not `ready` within 30 s returns `failed` with a reason that names each machine that was not `ready` and its state.
3. After a `failed` start, no `arma3_x64` or `arma3server_x64` process of that session runs, and a new `session start` is accepted.

## Next to

- [US-001](US-001-singleplayer-session.md) (feature): start a singleplayer session.
- [US-002](US-002-dedicated-session.md) (feature): start a dedicated session.
