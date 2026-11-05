---
type: Finding
id: m01-1
milestone: M1
title: A check whose name does not start with a story or requirement ID is accepted
description: list and run take a check whose @check name starts with a plain word, though API-01 requires the name to start with the ID the check proves.
raised_by: developer
for: developer
checks: []
stories: [US-038]
generated: { by: "claude-code/developer", at: 2026-11-05T10:21:39-08:00 }
addressed: []
verified: []
---

# A check whose name does not start with a story or requirement ID is accepted

## What the spec promises

`API-01`: "`@check` (the name, required, starting with the story or requirement ID it proves)". The check API contract gives the value of `@check` as "The check's name: the story or requirement ID it proves, then a short sentence", and says "a value of the wrong form ends the check `fail` / `check-error` naming the line".

`US-038`, criterion 1: a check whose header is malformed "ends `fail` / `check-error` with a message naming the line and the key, without touching the game".

## What the system does

A check file under `checks/` whose header is

```sqf
// @check A hook holds a player
```

is listed by `shakedown list` with the name `A hook holds a player` and an `error` of `null`, and `shakedown run` executes it against the session like any other check. No `check-error` names the line.

## What it costs

A test author who forgets the ID gets a check that runs and passes, but whose result no longer traces back to the story or requirement it proves. Coverage and findings name checks by ID, so the check is invisible to both, and nothing tells the author why.

## The check that holds it

None yet. A check for `US-038` that lists and runs a check named without an ID, and expects `fail` / `check-error` naming the `@check` line, would hold it.
