---
type: Finding
id: m01-2
milestone: M1
title: The check-timeout message names no next step
description: A check that runs past its @timeout ends with the message "The check ran past its timeout of 5 s while the game still answered.", which says what went wrong but not what the author does next, as NFR-US-01 requires.
raised_by: test
for: developer
checks: ["NFR-US-01 US-039 the check-timeout message says what to do next"]
stories: [US-039]
generated: { by: "claude-code/tester", at: 2026-11-05T12:05:00-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-05T21:15:00+01:00, note: "The check-timeout message names the step it ran past its timeout in and says to raise @timeout or find why the step runs long. crates/shakedown/tests/m01.rs us_018_us_021_us_022_us_039_and_the_check_api asserts both in the message." }
verified:
  - { by: "claude-code/tester", at: 2026-11-05T12:17:00-08:00 }
---

# The check-timeout message names no next step

## What the spec promises

`NFR-US-01` (must): "Every error message, command line output and tool error names what went wrong and the next thing to do, in one or two sentences, without a stack trace of the framework."

`US-039`: "A check whose body runs past its header's timeout while every machine still answers is ended and fails with reason check-timeout, naming the step it was in", "so that a slow mod is told apart from a stalled game."

## What the system does

A check with `// @timeout 5` whose step `slow part` loops `skd_fnc_tryWaitUntil` calls for 20 s ends, in the result file:

```json
"outcome": "fail",
"reason": { "kind": "check-timeout", "message": "The check ran past its timeout of 5 s while the game still answered.", "machine": "p1" },
"evidence": { "failed_step": ["slow part"] }
```

`shakedown run` prints the same message. Its criteria hold: the kind, the time and `evidence.failed_step` are right. The message says what went wrong and that the game still answered, but not what to do next. Every `check-error` message of M1 does (`Check its spelling ...`, `Make it return true or false.`, `Add CBA to the setup's mods ...`, or the rule the argument must meet).

## What it costs

A test author whose check times out reads the line in the run output and has to work out alone whether to raise `@timeout` or look at why the step ran long, and which step that was: the message does not name the step, so the author must open the result file for `evidence.failed_step`. Small, but it is the one M1 failure message an author reads without a hint at the fix.

## The check that holds it

`NFR-US-01 US-039 the check-timeout message says what to do next`. It passes when the message names `@timeout` as the next thing to do (raise it, or find why the step ran long), names the step `slow part`, and is one or two sentences with no stack trace.
