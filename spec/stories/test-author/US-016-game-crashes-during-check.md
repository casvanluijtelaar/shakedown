---
type: User Story
title: "A game process crashes during a check"
description: "When a game process exits during a check, the check is invalid naming the machine, its exit code and any crash dump, and the run still writes its result file."
status: draft
id: US-016
role: test-author
kind: system fail
requirements: [P-09, OUT-01, RES-01, NFR-RL-01]
---

# US-016 A game process crashes during a check

As a test author, I want to have a crash of the game reported as invalid with the machine, its exit code and its crash dump, so that I know the environment failed and have what I need to report it.

## Acceptance criteria

1. When `p2`'s process is ended during a check, that check ends `invalid` with reason kind `machine-lost`, `reason.machine` `p2`, and a message giving the exit code.
2. The result file is written, valid against `result.schema.json`, and lists `p2` with `end_state` `lost`, `lost.reason` `exited`, its exit code, and the paths of any crash dump in the run folder.
3. Following checks of the run end `invalid` / `session-broken`, and the run exits 2.

## Next to

- [US-033](US-033-full-suite-walk-away.md) (feature): run the full suite and walk away.
- [US-041](US-041-one-outcome-with-reason.md) (feature): every check gets one outcome.
