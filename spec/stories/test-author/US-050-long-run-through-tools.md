---
type: User Story
title: "An AI agent follows a long run without a tool call timing out"
description: "A run started through MCP returns a handle at once and is followed with calls that each stay under a minute, so a long suite never hits the agent's tool timeout."
status: draft
id: US-050
role: test-author
kind: feature
requirements: [AGT-03, RUN-04]
---

# US-050 An AI agent follows a long run without a tool call timing out

As a test author, I want to start a long run from my agent and follow it check by check, so that no tool call times out and the agent sees each verdict as it comes.

## Acceptance criteria

1. `run_start` returns a `run_id` within 5 s, and `run_status` with `wait_s` 50 returns as soon as a check ends with its outcome line, and never later than 60 s after it was called.
2. A tool that waits, called with a progress token, sends a progress notification at least every 5 s; `session_start` that is not `ready` within 50 s returns state `starting` and `status` later shows it `ready`.

## Next to

- [US-051](US-051-cancelled-tool-call.md) (system fail): the agent's call is cancelled.
