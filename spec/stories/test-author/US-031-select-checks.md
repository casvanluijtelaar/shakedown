---
type: User Story
title: "Run one check, a list of checks, or the checks a story or finding names"
description: "A test author or mod developer runs a single check, a list of checks in a given order, or every check whose header carries a story or finding tag, and lists the selection without starting the game."
status: draft
id: US-031
role: test-author
kind: feature
requirements: [RUN-01, RUN-02, RUN-03, API-01]
---

# US-031 Run one check, a list of checks, or the checks a story or finding names

As a test author, I want to run exactly the checks I care about now, by id or by the story or finding they name, so that I do not wait for the whole suite.

## Acceptance criteria

1. `shakedown run crate/US-019-open b/two` executes exactly those two checks in that order; the result file's `checks` holds two entries in that order.
2. `shakedown run --tag m01-3` executes every check whose `@tags` holds `m01-3`, in check id order, and `shakedown list --tag m01-3` prints the same ids with names, tags, setups, timeouts and reset modes, with no runner and no game running.
3. The MCP tools `list_checks` and `run_start` with `{"tags": ["m01-3"]}` select the same checks.

## Next to

- [US-032](US-032-selection-matches-nothing.md) (mistake): the selection matches nothing.
