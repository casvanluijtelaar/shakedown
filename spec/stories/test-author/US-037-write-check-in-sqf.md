---
type: User Story
title: "Write a check in SQF with steps, named by the story it proves"
description: "A test author writes a check as one SQF file with a short header and named steps, using only the check API, and the result shows each step with its times."
status: stable
id: US-037
role: test-author
kind: feature
requirements: [API-01, API-02, API-03, API-04, OBS-05, NFR-US-02]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-037 Write a check in SQF with steps, named by the story it proves

As a test author, I want to write a check in plain SQF with named steps and the story ID in its name, from the contract alone, so that anyone can read what it proves and where it failed.

## Acceptance criteria

1. A check file with only a `// @check US-019 ...` header and a body of `skd_fnc_step` calls, with no `#include` and no macro, runs and passes; `list` shows its name starting with `US-019`.
2. The result file lists every step in start order, nested steps after their parent with their full path, each with the machine it ran on, its state, its start and end on the wall clock, `diag_tickTime`, mission time and frame number, and its duration.
3. Values attached with `skd_fnc_evidence` appear in `evidence.attached` for a passing check as well as a failing one.

## Next to

- [US-038](US-038-broken-check.md) (mistake): the check itself is broken.
- [US-039](US-039-check-timeout.md) (error): the check runs past its timeout.
- [US-028](US-028-script-error-on-one-client.md) (error): a script error on one client.
