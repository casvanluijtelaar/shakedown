---
type: User Story
title: "A broken check is the author's to fix"
description: "A check whose header is malformed, whose body does not compile, or that calls the check API wrongly ends fail with reason check-error saying what to fix, never invalid, and the run goes on."
status: draft
id: US-038
role: test-author
kind: mistake
requirements: [DM-04, API-01, OUT-01, NFR-US-01]
---

# US-038 A broken check is the author's to fix

As a test author, I want to be told when my check itself is wrong and what to fix, so that I never mistake my own error for the mod's or the environment's.

## Acceptance criteria

1. A check with no `@check` line, with `@timeout fast`, or with a key given twice ends `fail` / `check-error` with a message naming the line and the key, without touching the game.
2. A check whose body has a syntax error ends `fail` / `check-error` with the engine's compile message; one that calls `skd_fnc_expect` inside code sent to another machine ends `fail` / `check-error` saying `skd_fnc_*` runs only in the body.
3. The next check of the run executes and can pass, and the session stays `ready`.

## Next to

- [US-037](US-037-write-check-in-sqf.md) (feature): write a check in SQF.
- [US-040](US-040-edit-check-rerun.md) (feature): edit a check and rerun it.
