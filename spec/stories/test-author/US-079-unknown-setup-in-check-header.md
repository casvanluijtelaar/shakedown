---
type: User Story
title: "A check names a setup the project file does not declare"
description: "A check whose @setups header names a setup the project file does not declare ends as the author's mistake naming it, instead of being skipped in every setup."
status: draft
id: US-079
role: test-author
kind: mistake
requirements: [RUN-07, DM-04, API-01]
---

# US-079 A check names a setup the project file does not declare

As a test author, I want a mistyped setup name to be reported, so that a check never goes unrun in every setup without my knowing.

## Acceptance criteria

1. A check with `// @setups nosuch` ends `fail` / `check-error` in every run that selects it, naming `nosuch` and the setups the project file declares; `shakedown list` shows the same error beside it.
2. `run --each-setup` with that check selected starts no session for `nosuch`.

## Next to

- [US-074](US-074-run-in-every-setup.md) (feature): one command runs every setup.
