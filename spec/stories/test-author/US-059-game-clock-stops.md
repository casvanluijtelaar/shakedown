---
type: User Story
title: "A machine's game clock stops while it still answers"
description: "When a machine's mission time or frame number stops while it still answers, as when a rendering client's Shadow stream is cut, the checks it overlaps are invalid naming the machine and the clock, not failed."
status: draft
id: US-059
role: test-author
kind: system fail
requirements: [HLT-02, DM-05, DM-06, NFR-RL-01, FIX-02, INS-04]
---

# US-059 A machine's game clock stops while it still answers

As a test author, I want to have a check that ran while a machine's game clock stood still reported invalid, naming the machine and the clock, so that a cut stream never reads as a defect in the mod.

## Acceptance criteria

1. When `p1`'s mission time stops advancing during a check while `p1` still answers orders, that check ends `invalid` with reason kind `unhealthy`, `reason.machine` `p1`, and a message saying its mission time stopped; the run exits 2.
2. While the clock stands still, `status` shows `p1` flagged unhealthy; once its clock moves again for 5 s the flag clears and the next check can pass.
3. A check whose assertion had already failed before the clock stopped stays `fail` / `assertion`.

## Next to

- [US-041](US-041-one-outcome-with-reason.md) (feature): every check gets one outcome.
