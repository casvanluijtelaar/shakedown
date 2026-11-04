---
type: User Story
title: "A machine stops answering during a check"
description: "When a machine stops answering in the middle of a check, the check is invalid naming the machine, not failed, and the following checks are invalid because the session is broken."
status: draft
id: US-015
role: test-author
kind: system fail
requirements: [OBS-03, DM-07, P-07, NFR-RL-01, INS-04]
---

# US-015 A machine stops answering during a check

As a test author, I want to have a machine that stops answering reported as the environment's fault, naming it, so that a stall never reads as a defect in the mod.

## Acceptance criteria

1. When a client's process is suspended while a check waits on it, that check ends `invalid` with reason kind `machine-lost`, `reason.machine` naming the client, and a message that says it stopped answering and what it was asked; it ends within the wait's ceiling plus 10 s.
2. Each following check of the run ends `invalid` / `session-broken`, the run exits 2, and the result file lists that machine with `end_state` `lost` and `lost.reason` `no-answer`.
3. `status` shows the session `broken` with the same reason, and the game processes are still running for inspection.

## Next to

- [US-013](US-013-run-code-on-machine.md) (feature): run code on a machine.
- [US-041](US-041-one-outcome-with-reason.md) (feature): every check gets one outcome.
