---
type: User Story
title: "A mission restart before a check does not bring every machine back"
description: "When the mission restart a check asked for does not bring every machine back, the check is invalid naming the machine, and the session is broken."
status: draft
id: US-012
role: test-author
kind: system fail
requirements: [RST-06, P-05, NFR-RL-01]
---

# US-012 A mission restart before a check does not bring every machine back

As a test author, I want to have a mission restart that loses a machine end the check invalid, naming the machine, so that I know the environment failed and not the mod.

## Acceptance criteria

1. When a client's process is ended while the mission restart before a `@reset mission` check runs, that check ends `invalid` with reason kind `reset-failed` and a message naming the machine that did not come back; following checks end `invalid` / `session-broken`.
2. The run's exit code is 2, and its result file is written and valid against `result.schema.json`.

## Next to

- [US-011](US-011-mission-reset.md) (feature): a check gets a mission restart.
