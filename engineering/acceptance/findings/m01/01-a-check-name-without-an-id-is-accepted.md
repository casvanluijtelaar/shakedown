---
type: Finding
id: m01-1
milestone: M1
title: A check whose name does not start with a story or requirement ID is accepted
description: list and run take a check whose @check name starts with a plain word, though API-01 requires the name to start with the ID the check proves.
raised_by: developer
for: developer
checks: ["US-038 API-01 list shows a @check name that starts with no ID as broken, naming line 1 and @check", "US-038 API-01 a @check name that starts with no ID ends fail check-error naming the line and the key without touching the game"]
stories: [US-038]
generated: { by: "claude-code/developer", at: 2026-11-05T10:21:39-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-05T19:50:00+01:00, note: "list and run refuse a @check name whose first word is not a story or requirement ID, naming the line, with check-error; the contract says what an ID looks like. crates/shakedown/src/checks.rs us_038_m01_1_a_name_without_an_id_is_refused_naming_the_line proves it." }
verified:
  - { by: "claude-code/tester", at: 2026-11-05T12:17:00-08:00 }
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

`US-038 API-01 list shows a @check name that starts with no ID as broken, naming line 1 and @check` and `US-038 API-01 a @check name that starts with no ID ends fail check-error naming the line and the key without touching the game`. They list and run a check whose header is `// @check A hook holds a player`.
