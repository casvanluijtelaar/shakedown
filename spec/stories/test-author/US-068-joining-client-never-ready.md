---
type: User Story
title: "A joining client never becomes ready"
description: "When a client a check makes join does not become ready within its ceiling, the check is invalid naming the machine and its last log lines, the client's process is stopped, and the session goes on without it."
status: draft
id: US-068
role: test-author
kind: system fail
requirements: [MCH-01, DM-05, SES-04, STP-02]
---

# US-068 A joining client never becomes ready

As a test author, I want a join that hangs to be reported as the environment's failure, so that a client that could not join never reads as a defect of the mod.

## Acceptance criteria

1. When the late client's process is suspended right after its launch, `["p3", 30] call skd_fnc_join` ends the check `invalid` / `machine-lost` within 30 s plus the runner's grace, naming `p3` and holding its last 20 RPT lines.
2. No `arma3_x64` process of `p3` runs after the check, `status` shows the session `ready` with `server`, `p1` and `p2`, and the next check runs with its own outcome.
3. When the late client's process exits during its join, the check ends `invalid` / `machine-lost` at once, naming `p3` and its exit code.

## Next to

- [US-066](US-066-client-joins-late.md) (feature): a client joins late.
- [US-015](US-015-machine-stops-answering.md) (system fail): a machine stops answering during a check.
