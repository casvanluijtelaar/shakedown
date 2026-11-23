---
type: User Story
title: "A client joins a dedicated session late and sees what the others see"
description: "A check makes a game client the setup declares as late join the running dedicated session, waits until it is ready, and asserts what it sees against the server and the other clients."
status: draft
id: US-066
role: test-author
kind: feature
requirements: [MCH-01, OBS-06, INS-04, RES-01]
---

# US-066 A client joins a dedicated session late and sees what the others see

As a test author, I want a check to make a client join a session that is already running, so that I can test what a player who joins in progress sees, as arma-grappling-hook's late-joiner cases do.

## Acceptance criteria

1. On a setup with `clients = 2` and `late = 1`, `session start` starts `server`, `p1` and `p2` only: `status` and `skd_fnc_machines` list no `p3`.
2. In a check that first creates a crate and sets a public variable on it on the server, `["p3"] call skd_fnc_join` returns once `p3` is `ready`, within its ceiling, and `skd_fnc_expectAgree` over `server`, `p1`, `p2` and `p3` on the crate's position and variable passes.
3. During the check `status` lists `p3` with start number 1 and its process id; the result file records the versions `p3` loaded, like every other machine's.
4. The next check starts without `p3`: `skd_fnc_machines` gives `server`, `p1` and `p2`, `p3`'s process has ended, and `p1` and `p2` keep their start numbers.
5. The same check run through MCP with `run_start` ends with the same outcome, and no tool call of it lasts more than 60 s.

## Next to

- [US-068](US-068-joining-client-never-ready.md) (system fail): a joining client never becomes ready.
- [US-067](US-067-client-leaves-and-rejoins.md) (feature): a client leaves and joins again.
