---
type: User Story
title: "A script error the in-game handler misses still reaches the result"
description: "A script error that reaches a machine's log without passing the in-game error handler is still caught from the log and fails the check like any other."
status: draft
id: US-030
role: test-author
kind: system fail
requirements: [ERR-02, INT-AR-07, NFR-RL-05]
---

# US-030 A script error the in-game handler misses still reaches the result

As a test author, I want to have every script error the log shows reach the result, so that an error the game reports where the handler cannot see it is not lost.

## Acceptance criteria

1. When an error line of the form `Error in expression <...>` with its `File ...` line reaches `p1`'s RPT during a check without the `ScriptError` handler firing, the check ends `fail` / `script-error`, and the result file lists the error with machine `p1` and `source` `rpt`.
2. Every script error line in each machine's log excerpt for a check appears in that check's `errors` in the result file.

## Next to

- [US-028](US-028-script-error-on-one-client.md) (error): a script error on one client.
