---
type: User Story
title: "A command that needs a session when none runs"
description: "A command or tool that needs a running session, called when none runs, is refused with what to do next, and starts no game."
status: draft
id: US-047
role: test-author
kind: mistake
requirements: [AGT-02, NFR-US-01, SES-01]
---

# US-047 A command that needs a session when none runs

As a test author, I want to be told to start a session when I ask for something that needs one, so that I know the next command instead of waiting on a game that is not there.

## Acceptance criteria

1. With no session, `shakedown exec --machine p1 --code "1"`, `inspect`, `reset` and `run` without `--setup` exit 4 with a message saying no session runs and naming `shakedown session start --setup <name>` and the setups the project declares.
2. The MCP tools `exec`, `inspect`, `reset` and `run_start` without `setup` return `isError` with code `no-session` and the same next step.

## Next to

- [US-046](US-046-status-without-game.md) (feature): see the runner, the session and the run.
