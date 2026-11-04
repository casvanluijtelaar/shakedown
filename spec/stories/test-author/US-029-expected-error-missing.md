---
type: User Story
title: "An expected script error never comes"
description: "A check that expects a script error that never happens fails at its end, saying which error it waited for."
status: draft
id: US-029
role: test-author
kind: error
requirements: [ERR-04, DM-03]
---

# US-029 An expected script error never comes

As a test author, I want to have a check fail when the error it expects never happens, so that a mod that silently stopped reporting a bad state is caught.

## Acceptance criteria

1. `["_skdfx_undefined", { 1 }] call skd_fnc_expectError` in a check that otherwise passes ends `fail` with reason kind `expected-error-missing` and a message quoting `_skdfx_undefined`.
2. The run exits 1, and `evidence.failed_step` names the step that held the `skd_fnc_expectError` call.

## Next to

- [US-027](US-027-expected-error.md) (feature): a check expects a script error.
