---
type: Deferred
title: "A check cannot restart the server during a session"
description: "MCH-01 names restarting the server during a session beside joining, leaving and rejoining, and M11 builds only the client part, since no check arma-grappling-hook has or plans restarts its server."
id: DEF-31
state: open
milestone: null
status: draft
---

# A check cannot restart the server during a session

## Summary

`MCH-01` (`CR-04`) promises that a check can make a machine leave, join late, rejoin, or restart the server during a session. M11 builds joining, leaving and rejoining for game clients, which arma-grappling-hook's late-joiner and disconnect cases need. None of its checks restarts the server mid-session, and its plan names none that would.

## Current behaviour

A dedicated session's server runs from the session's start to its stop, or to a `game-restart` reload, which restarts every machine. A check cannot restart the server alone.

## The gap

A mod whose server keeps state across a restart, such as one that saves to the profile namespace, cannot have that promise checked: what the clients see after the server comes back, and whether they come back with it.

## Options

1. `skd_fnc_restartServer [ceiling]`: the runner ends the server's process, starts it again with the same `server.cfg`, waits for every client to join again as `MCH-01`'s join does, and returns once every machine is `ready`. The server gets a new start number; each client keeps its start number if it reconnects, or gets a new one if it is restarted.
2. `#restartserver` through `serverCommand` from the server's agent, which restarts the process itself (research `14-sessions.md`), with the runner taking the new process's hello by the process id it finds as the old one's child or by its profile folder.

## Recommendation

The first, when a mod's story needs it: the runner already starts and ends server processes, so it keeps the session's process ids and job object without a process it did not start.

## Acceptance criteria

1. A check that restarts the server reads, on every client, a value the server saved before the restart and loaded after it.
2. A server that does not come back within the start ceiling makes the check `invalid` / `machine-lost` naming `server`, and the session `broken`.
