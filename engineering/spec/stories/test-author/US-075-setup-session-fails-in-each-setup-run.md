---
type: User Story
title: "One setup's session fails to start in a run over every setup"
description: "When one setup's session fails to start in a run over every setup, its checks are listed as not run with the start's reason, the other setups still run, and nothing is left running."
status: draft
id: US-075
role: test-author
kind: system fail
requirements: [RUN-07, SES-04, STP-02]
---

# US-075 One setup's session fails to start in a run over every setup

As a test author, I want one setup that cannot start to cost only its own checks, so that a suite I walked away from still gives me the verdicts it could reach.

## Acceptance criteria

1. When the second of three setups names a mod folder that does not exist, `run --each-setup` refuses that setup's start with the missing folder, lists each of its checks in its result file as `not-run` / `session-failed` with that reason, and runs the first and third setups to their ends.
2. When a machine of the second setup exits during its start, the same holds, with the reason of `SES-04`.
3. The command exits 2, and after it no game process runs.

## Next to

- [US-074](US-074-run-in-every-setup.md) (feature): one command runs every setup.
