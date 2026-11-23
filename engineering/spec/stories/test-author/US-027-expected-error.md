---
type: User Story
title: "A check of an error path expects a script error"
description: "A test author declares that a block of code is expected to raise a script error containing a text, and the check passes when it does, on any machine."
status: stable
id: US-027
role: test-author
kind: feature
requirements: [ERR-04, ERR-01]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-027 A check of an error path expects a script error

As a test author, I want to say that a script error is expected in one part of a check, so that I can check a mod's error path without the error failing the check.

## Acceptance criteria

1. `["_skdfx_undefined", { ["p2", {[] call skdfx_fnc_throw}] call skd_fnc_on }] call skd_fnc_expectError` passes, and the result file lists the error with machine `p2` and `expected` `true`.
2. The text matches case-insensitively, and an error of another text in the same block still fails the check `fail` / `script-error`.

## Next to

- [US-029](US-029-expected-error-missing.md) (error): the expected error never comes.
