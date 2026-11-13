---
type: User Story
title: "A machine's frame rate collapses or its answers lag during a check"
description: "When a machine's frame rate or answer time crosses the health thresholds during a check while it still answers, the check is invalid naming the machine and the measure, not failed."
status: draft
id: US-094
role: test-author
kind: system fail
requirements: [HLT-01, DM-05, DM-06]
---

# US-094 A machine's frame rate collapses or its answers lag during a check

As a test author, I want a machine starved of CPU to make its checks invalid, so that a slow environment never reads as the mod's failure.

## Acceptance criteria

1. When `p2`'s process is starved of CPU during a check, the check ends `invalid` / `unhealthy` naming `p2` and the measure that crossed its threshold, unless a cause for `fail` came first.
2. The flag clears once the measure is back under its threshold for 5 s, and the next check runs with its own outcome.

## Next to

- [US-093](US-093-machine-health-in-status.md) (feature): see each machine's frame rate and answer time.
- [US-059](US-059-game-clock-stops.md) (system fail): a machine's game clock stops.
