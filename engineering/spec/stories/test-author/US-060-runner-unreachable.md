---
type: User Story
title: "A new client cannot reach the runner that holds the session"
description: "When the runner that holds a session runs but does not answer, or runs another version, a new command or tool says so with the runner's process id and the way out, and never starts a second runner beside it."
status: stable
id: US-060
role: test-author
kind: system fail
requirements: [P-01, INS-02, SES-06, AGT-02, NFR-US-01]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# US-060 A new client cannot reach the runner that holds the session

As a test author, I want to be told plainly when my session is still up but my command cannot reach the runner that holds it, so that I neither lose the session to a second runner nor wait on a command that never answers.

## Acceptance criteria

1. With a session `ready` and the runner's process suspended by the `runner_pid` that `status` gave, `shakedown status --json` exits 0 within 10 s with `runner` `unreachable`, that `runner_pid` and `session` `null`, and `shakedown run` exits 4 within 10 s with a message naming the runner's process id and saying that ending that process ends its session with it.
2. While the runner is suspended, no second `shakedown runner` process starts; the MCP tools `run_start` and `exec` return `isError` with code `runner-unreachable`, and the tool `status` returns `runner` `unreachable`.
3. Once the runner's process is resumed, the next `shakedown status` shows the same session id `ready`, with every machine at the start number it had.
4. A stand-in process that `runner.json` names, which answers a client's hello with another version, is refused the same way, and the message names both versions.

## Next to

- [US-007](US-007-session-outlives-client.md) (feature): the session outlives the client that started it.
