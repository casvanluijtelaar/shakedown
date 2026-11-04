---
type: User Story
title: "See the runner, the session and the active run at any time"
description: "`status` answers at once with the runner, the session with each machine's state and last answer time, the active run's progress and the last reload, and works where no game or runner runs."
status: stable
id: US-046
role: test-author
kind: feature
requirements: [INS-02, AGT-05, NFR-PT-02, INS-04]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-046 See the runner, the session and the active run at any time

As a test author, I want to see at any time whether a session runs, what state each machine is in and how far a run has got, so that I know what is going on before I ask the game anything.

## Acceptance criteria

1. During a run, `shakedown status --json` returns the session with each machine's state and `last_answer_at`, and the run with `done`, `total` and `current_check`; the MCP tool `status` returns the same.
2. With no runner running, `shakedown status` exits 0 saying no runner runs, and starts nothing.
3. On the Mac, with no game installed, `shakedown mcp` starts and its `status` tool answers `runner` `not-running`; `list_checks` and `guide` answer too.

## Next to

- [US-047](US-047-no-session.md) (mistake): a command needs a session and none runs.
