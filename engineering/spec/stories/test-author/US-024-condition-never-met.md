---
type: User Story
title: "A condition that is never met"
description: "A wait whose condition never holds ends the check fail at its ceiling, with what was awaited, the ceiling, and the last value seen on each machine."
status: stable
id: US-024
role: test-author
kind: error
requirements: [OBS-04, DM-07, RES-03, NFR-PF-06]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-024 A condition that is never met

As a test author, I want to see what was awaited and the last value on each machine when a wait runs out, so that I can tell from the result alone how far the mod got.

## Acceptance criteria

1. A check whose `skd_fnc_waitUntil` with description `flag reached p1` and ceiling 3 never holds ends `fail` with reason kind `wait-ceiling` and a message naming the description and the 3 s ceiling; the run exits 1.
2. `skd_fnc_expectEach` on `"all"` that never holds ends `fail` / `assertion`, and the result file's `evidence.observed` holds the last value read on each machine, `evidence.failed_step` the step's path, and `evidence.logs` a log excerpt path per machine that exists in the run folder.
3. A `skd_fnc_waitUntil` given no ceiling ends at 10 s, and its message names the 10 s ceiling.

## Next to

- [US-023](US-023-wait-with-ceiling.md) (feature): wait for an outcome.
