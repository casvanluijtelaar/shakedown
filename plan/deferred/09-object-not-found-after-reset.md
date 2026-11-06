---
type: Deferred
title: "The server logs Object not found after a reset on a dedicated session"
description: "After a reset on a dedicated session the server's RPT shows Server: Object 2:nnn not found (message Type_189); the cause is to be found and fixed at its source in M3."
id: DEF-09
state: planned
milestone: M3
status: stable
---

# The server logs Object not found after a reset on a dedicated session

## Summary

After a reset on a dedicated session, the server's RPT shows lines such as `Server: Object 2:nnn not found (message Type_189)`. The owner wants the cause found and fixed at its source, so that the line is not logged, rather than concealed. It goes into M3, the milestone that reads RPTs for script errors.

## Current behaviour

Checks and resets pass, and the line appears in the server's RPT after a reset. The `2:` of the object id is not the server's own, so the message most likely concerns an object a client made, which the reset deleted before a message about it reached the server.

## The gap

Noise in the server's RPT that a person or an agent reading the log after a failure has to recognise as harmless, and that may point at a reset that deletes objects in an order the engine does not expect.

## Options

1. Find which object and which message: log the net ids the reset deletes on each machine and match them against the line, then change the reset so that no message is sent for an object that is gone, for example by changing which machine deletes it, or when.
2. Filter the line out of what M3 reads. This conceals it, and the owner does not want it.

## Recommendation

The first option, in M3. M3 does not count this line as a script error (it does not match `INT-AR-07`), and M3 grows no code whose only job is to hide it; the fix stops the line from being logged.

## Acceptance criteria

1. The cause is named in `research/` or in the M3 report: the object, the message and why it is sent.
2. Twenty resets on a dedicated session with two clients and a headless client, after checks that create and delete objects on each machine, leave no `Object ... not found` line in the server's RPT.
3. No code in the framework exists only to drop this line.
