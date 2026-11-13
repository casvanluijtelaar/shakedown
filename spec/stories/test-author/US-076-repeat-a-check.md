---
type: User Story
title: "Repeat a check and see how often each outcome came up"
description: "A run repeats each selected check a chosen number of times and reports for each check its summary outcome, pass, fail, flaky or invalid, with the count of each outcome."
status: draft
id: US-076
role: test-author
kind: feature
requirements: [OUT-02, DM-09, RES-01]
---

# US-076 Repeat a check and see how often each outcome came up

As a test author, I want to repeat a check and see the spread of its outcomes, so that a check that passes only sometimes shows as flaky instead of as a pass.

## Acceptance criteria

1. `shakedown run US-x --repeat 10` against a running session runs `US-x` ten times in a row, each with its own reset, and the result file gives ten executions with their repeat index, the summary outcome by `DM-09` and the count of each outcome.
2. For the fixture's check that passes about one time in two, the summary is `flaky`, the line printed for it gives the counts, and the run exits 1.
3. A check whose ten repeats all pass is `pass` and the run exits 0.
4. `run_start` with a repeat count through MCP gives the same summary in `run_result`.

## Next to

- [US-077](US-077-session-breaks-during-repeats.md) (system fail): the session breaks during repeats.
