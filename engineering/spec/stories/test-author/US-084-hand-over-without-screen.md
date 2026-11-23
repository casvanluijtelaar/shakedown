---
type: User Story
title: "A hand over to a machine with no screen"
description: "A check that hands over to the server or a headless client ends as the author's mistake, naming the game clients it can hand over to."
status: draft
id: US-084
role: test-author
kind: mistake
requirements: [MAN-01, DM-04]
---

# US-084 A hand over to a machine with no screen

As a test author, I want a hand over to a machine no person can look through to be refused, so that the manual tester is never sent to a window that does not exist.

## Acceptance criteria

1. On a dedicated session with two clients and a headless client, `skd_fnc_handOver` on `server` or `hc1` ends the check `fail` / `check-error` naming `p1` and `p2`, and the run goes on with its reset as usual.

## Next to

- [US-083](../manual-tester/US-083-handed-a-situation.md) (feature): be put at the start of a situation.
