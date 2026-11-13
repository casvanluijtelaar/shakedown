---
type: User Story
title: "The session breaks during repeats"
description: "When the session breaks during a repeated check, the repeat that was running is invalid, the repeats not started are not run, and the summary counts only the repeats that ran."
status: draft
id: US-077
role: test-author
kind: system fail
requirements: [OUT-02, DM-09, DM-05]
---

# US-077 The session breaks during repeats

As a test author, I want a run of repeats that lost its session to say how far it got, so that I never read a flaky or failing summary that the environment caused.

## Acceptance criteria

1. When `p2`'s process is ended during the third of five repeats on a dedicated session, the third repeat is `invalid` / `machine-lost`, the fourth and fifth are `not-run` / `session-broken`, and the summary counts two `pass` and one `invalid`, with summary outcome `pass` and the not-run count beside it.
2. When every repeat that ran was `invalid`, the summary outcome is `invalid` and the run exits 2.

## Next to

- [US-076](US-076-repeat-a-check.md) (feature): repeat a check.
