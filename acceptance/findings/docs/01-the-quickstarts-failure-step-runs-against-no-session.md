---
type: Finding
id: m19-1
milestone: M19
title: The quickstart's failure step runs against no session and exits 4
description: Step 9 of docs/quickstart.md says to run `shakedown run smoke` again after step 7, whose `run --setup dedicated` stopped every game, so the command exits 4 with "No session runs" instead of the failure the page shows.
raised_by: test
for: writer
checks: []
stories: [US-047, US-001]
generated: { by: "claude-code/tester", at: 2026-11-23T20:13:00Z }
addressed: []
verified: []
---

# The quickstart's failure step runs against no session and exits 4

This finding comes from checking the rewritten `docs/` after M19 closed, not from a milestone round.

## What the spec promises

`US-047`: a command that needs a session when none runs exits 4 and names `shakedown session start --setup`. `docs/running-checks.md` says the same: "`run` without `--setup` needs a running session; with none it exits 4". `docs/quickstart.md` step 9 then says: "Change the last line of the check to expect damage above 0.5 and run it again", with `> shakedown run smoke` printing `fail smoke (3.0 s)` and the assertion, and exit code 1.

## What the system does

The quickstart taken in order, in a project with `mods = []` as the page allows: step 6 ends with `shakedown session stop`, and step 7 runs `shakedown run --setup dedicated`, which starts and stops its own session (exit 0). Step 9 as written:

```
> shakedown run smoke
No session runs. Start one with `shakedown session start --setup <name>`; the project declares the setups: dedicated, sp.
[exit 4]
```

The build follows the contract. The same changed check, run in a session from `session start --setup sp`, prints what the page shows: `fail smoke (4.9 s)`, `assertion: p1 is hurt: expected > 0.5 on p1 within 3 s, last read 0.`, exit 1.

## What it costs

A new user who follows the quickstart line by line meets an error at the step meant to show them a failure. Step 9 needs `--setup sp`, or a session started first.

## The check that holds it

None: a document gap. Recorded in `acceptance/coverage/docs.md`.
