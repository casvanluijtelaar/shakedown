---
type: User Story
title: "A slow step threshold that is not a number of seconds"
description: "A project file whose slow_step is not a positive number is refused with the key and its line, like any other wrong key."
status: draft
id: US-080
role: test-author
kind: mistake
requirements: [TIM-01, CFG-01]
---

# US-080 A slow step threshold that is not a number of seconds

As a test author, I want a wrong threshold refused at once, so that I do not run a suite whose times are flagged against a value I did not mean.

## Acceptance criteria

1. With `slow_step = -1` or `slow_step = "10"`, `shakedown run` and `list` exit 3 naming `slow_step` and its line, and start nothing.

## Next to

- [US-078](US-078-slow-steps-flagged.md) (feature): slow steps are flagged.
