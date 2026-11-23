---
type: Finding
id: m19-8
milestone: M19
title: A slow_step given as the string "10" is refused with a type message, not the one the pages show for it
description: docs/project-file.md and docs/every-setup-and-repeats.md give one message for a slow_step of -1, 0 or "10"; for "10" the build names slow_step and its line, as the contract asks, but in serde's words "invalid type: string "10", expected f64".
raised_by: test
for: writer
checks: []
stories: [US-080]
generated: { by: "claude-code/tester", at: 2026-11-23T20:13:00Z }
addressed: []
verified: []
---

# A slow_step given as the string "10" is refused with a type message, not the one the pages show for it

This finding comes from checking the rewritten `docs/` after M19 closed, not from a milestone round.

## What the spec promises

`US-080` criterion 1 and `spec/contract/cli.md`: a string, `0` or a negative `slow_step` "is refused with exit code 3, naming `slow_step` and its line". `docs/project-file.md`: "A `slow_step` that is not a number above 0, such as `-1`, `0` or `"10"`:" followed by `shakedown.toml, line 3: slow_step must be a number of seconds above 0, got -1.`; `docs/every-setup-and-repeats.md` says the same.

## What the system does

```
> shakedown list            (slow_step = -1 on line 2)
shakedown.toml, line 2: slow_step must be a number of seconds above 0, got -1.
[exit 3]
> shakedown list            (slow_step = "10" on line 2)
shakedown.toml, line 2: key `slow_step`: invalid type: string "10", expected f64. Fix the key as contract/cli.md describes the project file.
[exit 3]
```

Both meet the contract. The pages suggest the first message for both.

## What it costs

Small: a reader who gets the second message finds no match in the docs. The pages should show the type message for a string, or say that it differs.

## The check that holds it

None: a document gap. Recorded in `acceptance/coverage/docs.md`.
