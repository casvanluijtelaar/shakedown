---
type: User Story
title: "A check runs past its timeout"
description: "A check whose body runs past its header's timeout while every machine still answers is ended and fails with reason check-timeout, naming the step it was in."
status: stable
id: US-039
role: test-author
kind: error
requirements: [DM-03, API-01, OBS-03]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-039 A check runs past its timeout

As a test author, I want to have a check that runs too long end at its own timeout with the step it was in, so that a slow mod is told apart from a stalled game.

## Acceptance criteria

1. A check with `// @timeout 5` whose step `slow part` runs a loop of `skd_fnc_tryWaitUntil` calls for 20 s ends `fail` with reason kind `check-timeout` after about 5 s, and `evidence.failed_step` is `["slow part"]` with that step in state `running`.
2. A wait whose ceiling reaches past the check's timeout is cut to the time left, so the check still ends `check-timeout` and not `invalid`.

## Next to

- [US-037](US-037-write-check-in-sqf.md) (feature): write a check in SQF.
