---
type: User Story
title: "Slow steps and slow checks are flagged"
description: "A step that takes longer than the project's threshold, and a check that takes more than half its timeout, are flagged slow in the result file and in the command's output."
status: draft
id: US-078
role: test-author
kind: feature
requirements: [TIM-01, OBS-05, RES-01]
---

# US-078 Slow steps and slow checks are flagged

As a test author, I want slow steps pointed out, so that I see where a long suite spends its time without reading every step's times.

## Acceptance criteria

1. A check with a step that waits 12 s on a condition that then holds ends `pass`, and the result file flags that step `slow` with its duration; the printed line for the check names the slow step.
2. With `slow_step = 15` in `shakedown.toml` the same step is not flagged.
3. A check with `// @timeout 20` that takes 11 s is flagged `slow` as a check.

## Next to

- [US-080](US-080-bad-slow-step.md) (mistake): a slow step threshold that is not a number of seconds.
