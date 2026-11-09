---
type: Deferred
title: "A start refused for no Steam still starts a runner that lingers"
description: "A session start refused because Steam is not running starts the runner first, which stays up about 60 s with no session and makes a quick retry run into it."
id: DEF-15
state: open
milestone: null
status: stable
---

# A start refused for no Steam still starts a runner that lingers

## Summary

`SES-03` refuses a start before any game process starts when Steam is not running, and it does: no game process starts. The runner does start, because the preconditions are checked in the runner, and it stays up after the refusal.

## Current behaviour

`session start` and the `session_start` tool connect to the runner first (`client::connect` in `client.rs`), starting it when none runs. The runner then checks the preconditions (`preconditions` in `runner/session.rs`) and refuses with exit 4 and "missing: a running Steam client of this user (start Steam and log in)". A runner with no session and no client exits by itself after `IDLE_EXIT`, 60 s (`runner/mod.rs`). In the acceptance round with Steam stopped, the runner the refused start brought up exited by itself within 61 s; a run of the check started before it had exited ran into it and was refused with exit 4 (`acceptance/coverage/m04.md`).

## The gap

A refusal that promises nothing started leaves a `shakedown runner` process for a minute. A person or an agent who starts Steam and tries again at once meets a runner they did not expect, and a tool that counts processes, as the acceptance suite does, sees one more than `SES-03` suggests.

## Options

1. Check the preconditions that need no runner, Steam among them, in the client before it starts the runner.
2. Have a runner that was started for a start that was refused exit as soon as the refusal is sent.
3. Say in `SES-03` and the docs that the runner may stay up for a minute after a refusal.

## Recommendation

The second, in M5, which makes the runner own its lifetime: it keeps the preconditions in one place and leaves nothing running after a refusal.

## Acceptance criteria

1. With Steam not running and no runner running, `session start --setup sp` exits 4 and, within 5 s, no `shakedown runner` process is left.
2. Started again within 5 s of that refusal, with Steam running and logged in, `session start --setup sp` starts the session.
