---
type: Deferred
title: "A reset a stop cut short runs on in the game"
description: "A run stopped during a check's reset ends that check at once, but the reset phase already sent to the machines, with any cleanup it runs, goes on in the game until it ends or reaches the reset's 15 s ceiling, so a run started right after can meet it."
id: DEF-23
state: open
milestone: null
status: stable
---

# A reset a stop cut short runs on in the game

## Summary

`RUN-05` and `RUN-06` end the check being prepared or executing when a run stops. M8 stops the check's body in the game: the runner cancels the check order and the lead terminates the body's script. A reset phase already sent is not stopped.

## Current behaviour

The runner drops its wait for the reset and writes the result file. The phase order already sent to each machine goes on: the `cleanups` phase runs the cleanup it was on, and terminates it only at the reset's ceiling of 15 s (`RST-04`). The runner sends no later phase. The session is `ready` as soon as the result file is written.

## The gap

A run, a reset or a reload asked within those 15 s can run beside the leftover phase. Its own reset runs the cleanups again from an empty list, so nothing runs twice, but a long cleanup of the stopped check still changes the game while the next check is being reset. `US-063` meets this case on purpose and passes, since its cleanups only wait.

## Options

1. Send each reset phase as a tracked order, so a stop cancels it, and terminate the cleanup's own script from the phase's script when it is terminated.
2. Keep the session busy after a stop until every reset phase sent has answered, within the reset's ceiling.

## Recommendation

The first, when a suite meets the case.

## Acceptance criteria

1. A run stopped during a cleanup that runs 14 s leaves no script of that cleanup running in the game 1 s after `run stop` returns, as `inspect` shows.
2. A run started at once after it passes its first check.
