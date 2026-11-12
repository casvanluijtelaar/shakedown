---
type: Deferred
title: "The runner's ceilings count the time the runner itself was paused"
description: "The heartbeat watch forgives a runner that was suspended, paused in a debugger or starved, but an order's ceiling plus its grace and a check's runner ceiling still count that time, so a check running while the runner is paused can end invalid though every machine answered."
id: DEF-24
state: open
milestone: null
status: stable
---

# The runner's ceilings count the time the runner itself was paused

## Summary

`HLT-02` counts a machine's heartbeat gap only over time the runner itself runs: a watch round 2 s or more late judges nothing and counts every gap again, so a runner that was paused loses no machine (`SES-06`, `US-060`). The other ceilings the runner keeps, an order's in-game ceiling plus the grace of `OBS-03` and a check's timeout plus the grace, are measured in wall-clock time from when they started.

## Current behaviour

With no check running, a runner suspended for 30 s and resumed keeps its session `ready` with every machine at its start number. With a check running, an order or a check whose ceiling passed while the runner was paused is judged at the resume as unanswered: the machine is `lost` with reason `no-answer`, the check ends `invalid` / `machine-lost` and the session is `broken`, though the machine answered all along and its answer is waiting in the runner's connection.

## The gap

A test author whose runner stalls during a check, through a debugger, a tool that suspends it or a starved machine, loses the session for a pause of the runner's own, and the reason blames a machine. No story of M9 promises a check survives a paused runner: `US-060` asks only that the session is still `ready` after the resume.

## Options

1. Count every runner ceiling over the runner's own running time: one clock that the watch advances only by rounds that came on time, which every ceiling reads instead of the wall clock.
2. At a resume, read every connection first and only then judge the ceilings, so an answer that arrived during the pause counts; a ceiling that truly passed still ends the check.
3. End a check that overlaps a runner pause `invalid` with a reason of its own that names the runner, not a machine, and keep the session `ready`.

## Recommendation

The first, when a suite or a person meets the case: it gives every ceiling the rule `HLT-02` already gives the heartbeat, and it needs no new outcome.

## Acceptance criteria

1. A check waiting 20 s on `p1`, with the runner suspended for 30 s during the wait and then resumed, ends with the outcome it has in a run without the pause, and the session stays `ready` with every machine at its start number.
2. A machine suspended during the same pause is still `lost` within 20 s of the resume.
