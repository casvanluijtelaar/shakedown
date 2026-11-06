---
type: Finding
id: m01-3
milestone: M1
title: A check given more than once runs once
description: shakedown run a a b runs a once and then b, though the owner decided that a check given more than once runs once per time it is given, in the order given.
raised_by: developer
for: developer
checks:
  - "US-031 m01-3 RUN-02 list a a b lists a and b once each"
  - "US-031 m01-3 RUN-03 run a a b executes a, a, b and the result file holds three entries with repeat 1, 2, 1"
  - "US-031 m01-3 RUN-03 run a b a executes a, b, a with repeat 1, 1, 2"
  - "US-031 m01-3 RUN-03 run a a a executes a three times with repeat 1, 2, 3"
  - "US-031 m01-3 RUN-01 a tag adds nothing for a check given twice by id: run a a --tag x executes a, a, c"
  - "US-031 m01-3 RUN-03 run a a b without --json prints one line per execution, the second of a marked #2"
stories: [US-031]
generated: { by: "claude-code/developer", at: 2026-11-06T00:22:37-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-06T00:29:00-08:00, note: "A check id given more than once runs once per time it is given, in the order given, each execution in the result file with repeat 1, 2, ... and a printed line ending in #2 from the second; RUN-01, RUN-03, US-031, the CLI, MCP and result-file contracts and the docs say so. crates/shakedown/tests/m01.rs us_031_m01_3_a_check_given_twice_runs_twice_in_the_order_given proves it in the game; the RUN-0 acceptance checks still pass." }
verified:
  - { by: "claude-code/tester", at: 2026-11-06T00:38:30-08:00 }
---

# A check given more than once runs once

## What the spec promises

The owner, `human:casvanluijtelaar`, wrote on 2026-11-06:

> "if run takes any number of checks, then adding the same name back to back should run it twice, same way that "run check1,check2,check1" should run 1, 2, 1 in that order"

> "so run A A B should also work. any order, just to be clear run A A A runs A three times"

This is a product decision, so the spec takes it as written. `RUN-03` (must): "Checks run in the order given"; `US-031` criterion 1: "`shakedown run crate/US-019-open b/two` executes exactly those two checks in that order; the result file's `checks` holds two entries in that order."

## What the system does

Against a running session, `shakedown run a a b` executes `a` once and then `b`. The result file's `checks` holds two entries, `a` then `b`, and the run prints two outcome lines. `shakedown run a b a` executes `a` and `b`, once each. The second time a check is given is dropped without a word.

## What it costs

A test author who gives a check twice, to see whether it passes a second time in the same session after the reset, or to run `a`, then `b`, then `a` again to see whether `b` leaves something behind, gets one execution and no sign that the second was dropped. The result looks like a pass of what was asked.

## The check that holds it

- `US-031 m01-3 RUN-02 list a a b lists a and b once each`
- `US-031 m01-3 RUN-03 run a a b executes a, a, b and the result file holds three entries with repeat 1, 2, 1`
- `US-031 m01-3 RUN-03 run a b a executes a, b, a with repeat 1, 1, 2`
- `US-031 m01-3 RUN-03 run a a a executes a three times with repeat 1, 2, 3`
- `US-031 m01-3 RUN-01 a tag adds nothing for a check given twice by id: run a a --tag x executes a, a, c`
- `US-031 m01-3 RUN-03 run a a b without --json prints one line per execution, the second of a marked #2`

The first runs offline; the other five run in one `sp` session, against the selection checks `sel/RUN-01-a`, `sel/RUN-01-b` and `sel/RUN-01-c`. `python acceptance/run.py --check m01-3` runs all six.
