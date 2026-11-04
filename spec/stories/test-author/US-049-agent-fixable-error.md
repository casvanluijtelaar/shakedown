---
type: User Story
title: "An AI agent's fixable mistake gets a code and the next step"
description: "When an AI agent calls a tool wrongly, the tool returns an error with a fixed code, what is wrong and what to do next, so the agent corrects itself without a person."
status: draft
id: US-049
role: test-author
kind: mistake
requirements: [AGT-02, NFR-US-01]
---

# US-049 An AI agent's fixable mistake gets a code and the next step

As a test author, I want to get an error with a code and the next thing to do when my agent calls a tool wrongly, so that it recovers on its own.

## Acceptance criteria

1. `exec` with `machine` `p9` returns `isError` with text of the form `unknown-machine: <what is wrong>. <what to do next>.` naming the machines that exist.
2. `run_result` on a run that has not ended returns `isError` with code `unknown-run` saying to call `run_status`; `exec` with a `ceiling` of 55 returns `bad-arguments` saying the most is 50.

## Next to

- [US-048](US-048-agent-tools.md) (feature): an AI agent uses every capability as tools.
