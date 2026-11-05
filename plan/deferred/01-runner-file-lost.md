---
type: Deferred
title: "A lost runner.json starts a second runner that ends the live session"
description: "When runner.json is deleted or unreadable while its runner holds a session, the next client starts a second runner, whose recovery ends the first session's game processes."
id: DEF-01
state: open
milestone: null
---

# A lost runner.json starts a second runner that ends the live session

## Summary

If `%LOCALAPPDATA%\Shakedown\runner.json` is deleted or cannot be read while its runner holds a session, the next client finds no live runner and starts a second one. The second runner's recovery (`P-12`) ends every process the run record lists, which are the live session's game processes.

## Current behaviour

`P-01` starts a new runner whenever `runner.json` names no live runner. `P-12` ends every process in the run record that still runs with its recorded start time. `US-060` covers a runner that `runner.json` names and that does not answer; nothing covers a runner `runner.json` no longer names.

## The gap

A session that is still up is ended without a word, and two runners exist until the first exits by itself.

## Options

1. The runner holds a named mutex, `Local\Shakedown-runner`, for its whole life. A second runner that cannot take it exits at once, and the client reports a runner it cannot reach, as `US-060` does.
2. `P-12` skips the processes of a session whose runner still runs, found by the job object that holds them.
3. Leave it: only the runner writes `runner.json`, and nothing else is known to remove it.

## Recommendation

The first option: one mutex, a few lines, and the refusal path `US-060` builds anyway.

## Acceptance criteria

1. With a session `ready` and `runner.json` deleted, `shakedown status` starts no second runner, says that a runner runs that it cannot reach, and the session's machines keep running.
2. After the first runner is ended, the next command starts a fresh runner as `P-01` says.
