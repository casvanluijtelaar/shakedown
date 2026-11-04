---
type: User Story
title: "A second session start while one runs is refused"
description: "Starting a session while another one runs is refused, naming the running session and how to stop it, and leaves the running session untouched."
status: draft
id: US-005
role: test-author
kind: mistake
requirements: [SES-02, NFR-US-01]
---

# US-005 A second session start while one runs is refused

As a test author, I want to be stopped when I start a session while one already runs, so that I never fight another session over the one game install and its ports.

## Acceptance criteria

1. `session start` while a session runs exits 4 with a message naming the running session's id and setup and the command `shakedown session stop`; `session_start` returns `isError` with code `session-exists`.
2. After the refusal, `status` shows the running session still `ready`, with the same machines and start numbers as before.

## Next to

- [US-002](US-002-dedicated-session.md) (feature): start a dedicated session.
