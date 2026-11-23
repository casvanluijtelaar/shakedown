---
type: User Story
title: "Be put at the start of a situation with a list of what to look for"
description: "A manual tester is put in a client's window at the start of a situation a check built, with the list of what to look for on screen and in the command's output, and the session stays up until they are done."
status: draft
id: US-083
role: manual-tester
kind: feature
requirements: [MAN-01, SES-06, INS-02]
---

# US-083 Be put at the start of a situation with a list of what to look for

As a manual tester, I want the framework to set up the situation and tell me what to look for, so that I judge how it looks and feels instead of pasting into the debug console to get there.

## Acceptance criteria

1. A check that builds a scene and calls `skd_fnc_handOver` on `p1` with three items to look for ends the run with that check handed over; the command and `run_result` print the three items, the machine whose window to use and the command that ends the session.
2. `p1`'s window shows the three items on screen, and `p1` stands where the check put it: no reset ran after the check.
3. A `run --setup` that started its own session leaves it up after a hand over, and `status` shows it `ready`.
4. The next `run`, `reset`, `reload` or `session stop` takes the session back as usual.

## Next to

- [US-084](../test-author/US-084-hand-over-without-screen.md) (mistake): a hand over to a machine with no screen.
