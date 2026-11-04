---
type: User Story
title: "Wait for an outcome on a machine with a ceiling, never a fixed sleep"
description: "A test author waits for a condition on a named machine until it holds or a ceiling passes, and the check goes on as soon as it holds."
status: draft
id: US-023
role: test-author
kind: feature
requirements: [OBS-01, OBS-02, OBS-07]
---

# US-023 Wait for an outcome on a machine with a ceiling, never a fixed sleep

As a test author, I want to wait for the outcome I expect with a ceiling instead of sleeping a guessed time, so that a check is as fast as the mod and never flaky from a sleep that was too short.

## Acceptance criteria

1. After `["server", {[true, 3] call skdfx_fnc_setFlagLater}] call skd_fnc_on`, `["p1", {!isNil "skdfx_flag"}, "flag reached p1", 10] call skd_fnc_waitUntil` returns `true`, and its step's duration in the result file is under 5 s.
2. `skd_fnc_tryWaitUntil` with a ceiling of 1 s on a condition that never holds returns `false` and the check goes on to pass.
3. `shakedown wait --machine server --condition "false" --ceiling 2 --json` exits 0 after about 2 s with `met` `false`; the MCP tool `wait_until` returns the same.

## Next to

- [US-024](US-024-condition-never-met.md) (error): the condition is never met.
