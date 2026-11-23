---
type: User Story
title: "Joining or leaving a machine that cannot"
description: "Asking a machine to join or leave that cannot, the server, a headless client, a machine the setup does not name, a client already in or already out, or any machine in singleplayer, ends the check as the author's mistake, naming the machines that can."
status: draft
id: US-069
role: test-author
kind: mistake
requirements: [MCH-01, DM-04]
---

# US-069 Joining or leaving a machine that cannot

As a test author, I want a wrong join or leave to say which machines can, so that I fix my check instead of reading a verdict about the mod.

## Acceptance criteria

1. On a dedicated session with `clients = 2` and `late = 1`, each of `["server"] call skd_fnc_leave`, `["hc1"] call skd_fnc_leave`, `["p1"] call skd_fnc_join`, `["p9"] call skd_fnc_join`, and a second `["p2"] call skd_fnc_leave` after `p2` left, ends the check `fail` / `check-error` naming the machine and the machines that can join or leave at that moment.
2. In singleplayer `["p1"] call skd_fnc_leave` ends the check `fail` / `check-error` saying joining and leaving needs a dedicated session.
3. After each of these the session is `ready` with the machines it had before the check.

## Next to

- [US-067](US-067-client-leaves-and-rejoins.md) (feature): a client leaves and joins again.
