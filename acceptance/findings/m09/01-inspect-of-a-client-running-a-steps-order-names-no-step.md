---
type: Finding
id: m09-1
milestone: M9
title: Inspect of a client that runs the wait of a check's step names the check but no step
description: While a check's step waits on p1, shakedown inspect --machine p1 names the run and the check but gives step_path [], while inspect of the lead names the step; INS-01 and US-044 promise the step running.
status: draft
raised_by: test
for: developer
checks:
  - "US-044 INS-01 inspect of p1 while p1 runs the wait of the check's step names the run, the check and that step"
stories: [US-044]
generated: { by: "claude-code/tester", at: 2026-11-12T12:40:00-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-12T14:05:00-08:00, note: "Inspect of a machine that runs no check names the lead's step whose order (a wait, a poll or code of a step) is the newest still running there; INS-01 and docs/asking-a-running-game.md say so. The m09 game test us_044_ins_01_inspect_of_a_client_running_a_steps_wait_names_the_step proves it, and the named check passes." }
verified: []
---

# Inspect of a client that runs the wait of a check's step names the check but no step

## What the spec promises

`INS-01`: "`inspect <machine>` returns, without stopping anything: ... the run, check and step running". `US-044`, criterion 2: "`inspect`, `exec`, `logs` and `errors` called while a check runs answer within 10 s, the inspect names the check and step running". Criterion 1 of the same story asks `inspect --machine p1`. `docs/asking-a-running-game.md` gives `inspect` of any machine "the run, check and step running". Nothing in the contract limits the step to the lead.

## What the system does

On a ready dedicated session of a server and two clients loading `@skd_fixture`, a check whose body is one step, `wait for the flag the server sets 15 s later`, which calls `skd_fnc_waitUntil` on `p1` with a ceiling of 40 s. While the lead names that step running:

```
$ shakedown inspect --machine server --json
{ ..., "run_id": "r-20261112-202852-3", "check_id": "US-044-slow", "step_path": ["wait for the flag the server sets 15 s later"], ... }

$ shakedown inspect --machine p1 --json
{ ..., "run_id": "r-20261112-202852-3", "check_id": "US-044-slow", "step_path": [], ... }
```

`p1` is the machine polling that step's condition at that moment, yet its answer gives an empty step path. Seen in the trial round and in a second run of the check alone, the same each time.

## What it costs

A test author or an AI agent who asks the machine a check is waiting on, which is the natural one to ask when a wait stalls, learns which check runs but not where in it. They have to know to ask the lead instead, which the documents do not say. Either the client's inspect names the step whose order it runs, or `INS-01`, `spec/contract/mcp-tools.md` and `docs/asking-a-running-game.md` say the step is the lead's and a client gives `[]` unless it runs code of a step.

## The check that holds it

`US-044 INS-01 inspect of p1 while p1 runs the wait of the check's step names the run, the check and that step`
