---
type: Deferred
title: "No check has searched a failed start for the Steam identity"
description: "US-064 searches a stopped session after a run for the Steam ID and the Steam name; no check has searched what a failed start prints or keeps for them."
id: DEF-12
state: planned
milestone: M4
status: stable
---

# No check has searched a failed start for the Steam identity

## Summary

`LOG-02` and `INT-STM-04` keep the Steam ID and the Steam name out of everything the framework prints or keeps. `US-064` proves it for a run on a session that is then stopped: the result file, the run folder's logs, the `logs` output and the session's RPTs. A start that fails (`US-004`) prints its `reason` with the last 20 RPT lines of a machine and leaves that machine's RPTs in the session folder, and no check has searched either. It goes into M4 beside `DEF-10`, which makes `status` keep the failed start and its reason.

## Current behaviour

The runner redacts everything it sends a client and every RPT of a session it stops, and a failed start stops its processes. Whether the output of a failed `session start`, its `reason` and the RPTs it leaves hold the Steam ID or the Steam name has not been searched on the test machine.

## The gap

A failed start's `rpt_tail` holds the lines the game wrote during the start, the lines most likely to name the Steam account. If one of these paths is not redacted, the Steam identity reaches the terminal, an agent's context or a shared session folder, and nothing would show it.

## Options

1. A framework test in M4 that makes a start fail with a game client that wrote its connection lines, and searches the output of `session start`, `status` with the kept failed start, and the session folder's RPTs for the Steam ID and the Steam name; any path it finds is redacted.
2. Add a criterion to `US-064` and leave the test to the acceptance round.

## Recommendation

Both: the framework test in M4, with `DEF-10`'s kept failed start, and a third criterion on `US-064` so the acceptance round searches it too.

## Acceptance criteria

1. After a start that fails once a game client has logged in, the output of `session start`, the `reason` that `status` shows and every RPT of that session's machines hold neither the Steam ID nor the Steam name.
2. `US-064` names the failed start among the places searched.
