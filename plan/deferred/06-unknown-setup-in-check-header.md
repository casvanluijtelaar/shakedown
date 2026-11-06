---
type: Deferred
title: "A check whose @setups names an unknown setup is not reported"
description: "A check whose @setups names a setup the project file does not declare, such as @setups nosuch, is listed with no error and skipped in every session."
id: DEF-06
state: done
milestone: M2
status: stable
---

# A check whose @setups names an unknown setup is not reported

## Summary

`@setups` names the setups a check runs in (`RUN-07`), and the setups are those `shakedown.toml` declares. The check API contract checks only the form of the value: a key given twice, an unknown key, a missing `@check` or a value of the wrong form ends the check `fail` / `check-error`. It says nothing of a setup name the project file does not declare. With one setup in a project, as in M0 and M1, every check either leaves `@setups` out or names that setup, so a misspelt name has little room to hide.

## Current behaviour

A check with `@setups nosuch` is listed by `list` with `setups` `["nosuch"]` and an `error` of `null`. A run skips it as `not-run` / `setup` in every session, since no session's setup is named `nosuch`.

## The gap

A test author who misspells a setup name gets a check that never runs and is never reported as wrong; it shows only as one more `not-run` / `setup` in each result. Once M2 brings dedicated setups next to `sp`, a project has several setups and a check names a subset of them, which is where a misspelt name goes unnoticed.

## Options

1. `list` and `run` give a check whose `@setups` names a setup the project file does not declare a `check-error` that names the line, the unknown name and the setups the file declares, as `US-006` does for `session start`.
2. Keep the check valid, and have `list` add a warning for an unknown setup name while `run` still skips it.

## Recommendation

The first option. It matches how a malformed header is treated everywhere else (`US-038`) and how an unknown setup is refused at `session start` (`US-006`), and `list` already reads the project file, so it has the declared setups at hand. The spec edit goes into `spec/contract/check-api.md` and `API-01` when the work is planned.

## Acceptance criteria

1. A check with `@setups nosuch` in a project that declares no setup `nosuch` is listed by `list` with an `error` that names the line, `nosuch` and the setups the project file declares.
2. A run that selects it ends it `fail` / `check-error` with the same message, and the next check of the run executes.
3. A check whose `@setups` names only declared setups lists and runs as it does now.
