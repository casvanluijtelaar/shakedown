---
type: Deferred
title: "The code of wait, expect and inspect is not in the audit"
description: "audit.jsonl records every exec from the command line, MCP and a check, but not the condition of wait or wait_until, the value of expect, or the code inspect sends, though each is code run on a machine."
id: DEF-18
state: open
milestone: null
status: stable
---

# The code of wait, expect and inspect is not in the audit

## Summary

`SEC-04` asks for every `exec` order in `audit.jsonl`, and M5 records each `exec` from the command line and MCP, each check's body and each piece of code a check sends to another machine. `US-052`'s title promises that every piece of code sent to the session is recorded. The `wait` and `expect` commands and their tools send a user's condition or value to a machine as code too, and `inspect` sends the framework's own.

## Current behaviour

`control::exec` and the routing of a check's orders in `crates/shakedown/src/runner/` append to the audit. `control::wait`, `control::expect` and `control::inspect` send their orders without it.

## The gap

A condition given to `shakedown wait` or `wait_until` runs on a machine and leaves no line in the audit, so a reader of `audit.jsonl` does not see all the code that ran from outside a check.

## Options

1. Record `wait` and `expect` with their source and the condition or value as `code`, and leave `inspect` out as the framework's own code.
2. Record all three.
3. Keep `SEC-04` to `exec` and word `US-052`'s title to match.

## Recommendation

The first: the condition of a wait is a user's code like any `exec`, and `inspect` sends nothing the user wrote. It needs the source on the `Wait` and `Expect` requests, as `Exec` has it.

## Acceptance criteria

1. `shakedown wait` and `shakedown expect`, and the tools `wait_until` and `expect`, each add a line to `audit.jsonl` with the source `cli` or `mcp`, the target and the condition or value as `code`.
2. `SEC-04` and `docs/sessions-and-the-runner.md` say so.
