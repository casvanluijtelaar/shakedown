---
type: User Story
title: "A client leaves and joins again"
description: "A check makes a game client leave the session, waits until the server has seen it go, asserts what the server and the others did about it, and makes the client join again."
status: draft
id: US-067
role: test-author
kind: feature
requirements: [MCH-01, DM-05, INS-04]
---

# US-067 A client leaves and joins again

As a test author, I want a check to make a client leave and come back, so that I can test what a mod does when a player disconnects while it holds something, and when they rejoin.

## Acceptance criteria

1. On a dedicated session with two clients, `["p2"] call skd_fnc_leave` returns once the server's `PlayerDisconnected` event has fired for `p2`, within its default ceiling; `p2`'s process has ended and `status` shows no `p2`.
2. A value the fixture's `PlayerDisconnected` handler sets on the server is read with `skd_fnc_expectOn` in the same check, and the check ends `pass`: no outcome is `invalid` and the session stays `ready`.
3. `["p2"] call skd_fnc_join` in the same check brings `p2` back `ready` with start number 2, and `skd_fnc_player` for `p2` gives its new unit.
4. After a check that left `p2` out, the next check starts with `p2` joined again, `ready`, and the reset verified on every machine; after a check that made `p2` join again, the next check starts with that same start of `p2`.
5. The time of each join and leave is recorded as a step in the result file.

## Next to

- [US-066](US-066-client-joins-late.md) (feature): a client joins late.
- [US-069](US-069-join-or-leave-wrong-machine.md) (mistake): joining or leaving a machine that cannot.
