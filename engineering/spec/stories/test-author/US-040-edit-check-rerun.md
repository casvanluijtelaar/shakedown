---
type: User Story
title: "Edit a check and rerun it with no reload"
description: "A test author edits a check file and reruns it against the running session; the new text runs at once, with no reload and no restart."
status: stable
id: US-040
role: test-author
kind: feature
requirements: [RLD-04, API-02, NFR-PF-01]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-040 Edit a check and rerun it with no reload

As a test author, I want to change a check and rerun it without reloading anything, so that iterating on a check takes seconds.

## Acceptance criteria

1. After editing a check's assertion from `== 1` to `== 2` and running it again, the result file shows the new expectation in the failure message, and every machine keeps its start number.
2. `shakedown reload` naming only check files reports level `none` with the reason that checks are read fresh for every execution.
3. With a `ready` session, the time from saving the edit to the check's outcome is under 30 s for a check whose body takes under 10 s.

## Next to

- [US-038](US-038-broken-check.md) (mistake): the check itself is broken.
