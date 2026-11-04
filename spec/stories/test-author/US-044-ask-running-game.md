---
type: User Story
title: "Ask a running game what is going on"
description: "A person or an AI agent asks a running machine for its state, the player and the objects around it, its running scripts and its last errors, at any time, including during a check and after one failed, without stopping anything."
status: stable
id: US-044
role: test-author
kind: feature
requirements: [INS-01, INS-03, RST-07, ERR-05, CTL-01]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-044 Ask a running game what is going on

As a test author, I want to ask the running game for the state of the player and the objects around it when a check fails or stalls, so that I find the cause without a restart.

## Acceptance criteria

1. After a check fails, `shakedown inspect --machine p1 --json` returns `p1`'s player with position, direction, damage and vehicle, and the objects the check created within 50 m with their class, net id and distance, because the state stays until the next reset; the MCP tool `inspect` returns the same.
2. `inspect`, `exec`, `logs` and `errors` called while a check runs answer within 10 s, the inspect names the check and step running, and the check's outcome is the same as in a run without them.
3. `shakedown errors` returns script errors raised outside any check, such as from an `exec`, with their machine and time; and on a `broken` session, `inspect`, `logs` and `errors` still answer for every machine that answers.

## Next to

- [US-045](US-045-inspect-stalled-machine.md) (system fail): the machine asked has stopped answering.
