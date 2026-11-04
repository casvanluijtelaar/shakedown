---
type: User Story
title: "A selection that matches no check"
description: "A run or a listing whose ids or tags match no check is refused, listing what exists, and starts nothing."
status: draft
id: US-032
role: test-author
kind: mistake
requirements: [RUN-01, AGT-02, NFR-US-01]
---

# US-032 A selection that matches no check

As a test author, I want to be told when my selection matches nothing, with what exists, so that a typo in a tag does not look like a suite that passed.

## Acceptance criteria

1. `shakedown run --tag US-999` exits 3 with a message saying no check carries `US-999`, and lists the tags that exist; `run_start` returns `isError` with code `unknown-check`.
2. No run folder is created and the session stays `ready`.

## Next to

- [US-031](US-031-select-checks.md) (feature): run one check, a list, or a tag.
