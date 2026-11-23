---
type: Deferred
title: "A cancelled session_start or reset goes on"
description: "Cancelling the session_start or reset tool call does not stop the start or the reset in the game; only exec, wait_until, expect and inspect stop their work when cancelled."
id: DEF-13
state: open
milestone: null
status: stable
---

# A cancelled session_start or reset goes on

## Summary

`AGT-04` says a cancelled tool call stops the game-side work it started. `US-051` proves it for `exec` and `wait_until`, and M4 built it for every tool that runs code on a machine: `exec`, `wait_until`, `expect` and `inspect`. A cancelled `session_start` or `reset` is not stopped.

## Current behaviour

The runner runs a session start and a reset to their end whether or not the client that asked is still connected, so a session is never left half started and a reset never half done. When the client cancels `session_start`, the start goes on, and `status` shows it `starting`, then `ready` or `failed`. `run_start` returns at once, so there is no call to cancel while the run goes on; a run is stopped by `run_stop` (M8).

## The gap

An agent that gives up on a slow start leaves the game processes starting, and must call `session_stop` to end them. Nothing is lost or wrong in the session, but the call did not stop what it started.

## Options

1. A cancelled `session_start` stops the session as `session_stop` does, once the start has reached a state it can stop from.
2. Keep the start going and say so in the tool's description, so the agent calls `session_stop` itself.

## Recommendation

The second for now, as `docs/agent-tools.md` says; the first with `run_stop` in M8, which builds the stopping of work in progress.

## Acceptance criteria

1. A `session_start` cancelled by the client 10 s after it was called leaves no game process of that session within 30 s, and `status` shows no session.
2. A `reset` cancelled by the client leaves the session `ready` or `broken`, never `reloading`.
