---
type: Deferred
title: "status drops a failed start at once"
description: "Once a start fails, status no longer shows the session, so a program that only watches status cannot see the failed state or its reason; status is to keep the failed start until the next start or stop."
id: DEF-10
state: planned
milestone: M4
status: stable
---

# status drops a failed start at once

## Summary

A start that fails (`SES-04`) ends with the session `failed` and its `reason`: the machine, its state and its last 20 RPT lines. Only the output of `session start` carries them. The owner wants `status` to keep the failed start and its reason until the next start or `stop`. It goes into M4, the milestone that builds the agent tools, where `session_start` returns after 50 s with state `starting` and the agent follows the start with `status`.

## Current behaviour

As soon as a start fails, the runner drops the session, and `status` answers with no session. A program that started the session in the background and watches only `status` sees the session go from `starting` to nothing, with no state `failed` and no reason.

## The gap

An AI agent that follows a long start through `status`, as the MCP contract tells it to, cannot tell a failed start from one that was stopped, and cannot read why it failed.

## Options

1. The runner keeps the failed session and its `reason` after the start ends, and `status` lists it with state `failed` and no machines, since every process was stopped; the next `session start` or `session stop` clears it.
2. The runner writes the failed session to a file that `status` reads.

## Recommendation

The first option, in M4, with the `status` tool. `session stop` on a kept failed start clears it and exits 0, as nothing is running.

## Acceptance criteria

1. After a start that fails, `status` and the `status` tool show the session with state `failed` and the same `reason` that `session start` printed, until the next `session start` or `session stop`.
2. A `session start` after a failed one starts as usual, and `status` then shows the new session.
3. `INS-02`, the session status machine in `spec/01-domain-model.md` and the MCP contract say that `status` keeps a failed start.
