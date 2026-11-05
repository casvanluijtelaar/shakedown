---
type: User Story
title: "The session outlives the command or the AI agent session that started it"
description: "A session started from the command line or from an AI agent's MCP server keeps running when that client ends, and the next client finds it."
status: stable
id: US-007
role: test-author
kind: feature
requirements: [SES-06, SES-08]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-007 The session outlives the command or the AI agent session that started it

As a test author, I want to have the game session keep running when my command or my AI agent's session ends, so that the next command or the next agent session picks it up instead of starting the game again.

## Acceptance criteria

1. After `session_start` through the MCP server and the end of the Claude Code session that ran it, a new Claude Code session's `status` returns the same session id with state `ready`.
2. After `shakedown session start` returns, a later `shakedown run` in another terminal runs against that session, and its result file names the same session id.

## Next to

- [US-035](US-035-runner-dies.md) (system fail): the runner itself dies.
- [US-060](US-060-runner-unreachable.md) (system fail): the session runs, but a new client cannot reach its runner.
