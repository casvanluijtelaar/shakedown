---
type: User Story
title: "A library file that does not compile"
description: "A library file with a syntax error ends every check of the run as the author's mistake, naming the file and line, instead of failing the checks that call it in ways that hide the cause."
status: draft
id: US-071
role: test-author
kind: mistake
requirements: [API-06, DM-04]
---

# US-071 A library file that does not compile

As a test author, I want a broken helper to be named as the cause, so that I fix the one file instead of chasing failures across every check that uses it.

## Acceptance criteria

1. With `tests/_lib/rope.sqf` holding `1 +`, every check of a run ends `fail` / `check-error` naming `tests/_lib/rope.sqf`, its line and the engine's message, and the run goes on to its end.
2. Once the file is fixed and `shakedown reload` is run, the next run's checks reach their own outcomes.

## Next to

- [US-070](US-070-shared-helper-library.md) (feature): checks share helper code.
