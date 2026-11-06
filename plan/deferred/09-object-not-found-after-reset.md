---
type: Deferred
title: "The server logs Object not found after a reset on a dedicated session"
description: "A reset on a dedicated session in which every machine deleted every entity it recorded made the server log Server: Object 2:nnn not found (message Type_189); each machine deletes only the entities local to it, and the line is gone."
id: DEF-09
state: done
milestone: M3
status: stable
---

# The server logs Object not found after a reset on a dedicated session

## Summary

After a reset on a dedicated session, the server's RPT shows lines such as `Server: Object 2:nnn not found (message Type_189)`. The owner wants the cause found and fixed at its source, so that the line is not logged, rather than concealed. It goes into M3, the milestone that reads RPTs for script errors.

## Current behaviour

Each reset deletes only the entities that are local to the machine deleting them, and twenty resets after checks that create objects on every machine leave no `Object ... not found` line in the server's RPT (`def_09_twenty_resets_leave_no_object_not_found_line_on_the_server` in `crates/shakedown/tests/m03.rs`).

The cause: every machine's entity recorder sees every entity created in the mission, its own and those other machines created, and the reset had each machine delete everything its recorder saw, so each client also deleted the objects the server had made. The lines name exactly those objects: in an M2 run each check made a soldier, a quad bike, a box and an MRAP on the server, and after each reset the server logged three lines for each of three consecutive server ids (`2:96`, `2:98`, `2:99`), one per client and headless client that deleted an object it did not own, and one line for the fourth (`2:100`), the MRAP `p1` had driven. The `2:` is the id of the machine that made the object, the server's own (`clientOwner` on the server is 2). A deletion by a machine that does not own an object goes to its owner through the server, and the server, which has deleted the object itself by then, no longer finds it. `Type_189` is the engine's own, undocumented message number. Made on purpose in a check, one object deleted by every machine at once gave the line in one of three tries, so the test proves the fix by the twenty resets rather than by making the line. Over four runs of 21 resets in M2 the server logged 125 to 253 such lines.

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
