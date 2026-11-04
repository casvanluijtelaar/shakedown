---
type: User Story
title: "Start a dedicated server with clients and headless clients in one command"
description: "A test author starts a dedicated server with a chosen number of game clients and headless clients with one command and addresses each machine by its name."
status: draft
id: US-002
role: test-author
kind: feature
requirements: [SES-01, SES-05, CTL-01, RES-02, DM-10, DM-12]
---

# US-002 Start a dedicated server with clients and headless clients in one command

As a test author, I want to start a dedicated server with two game clients and a headless client in one command and address each by name, so that multiplayer behaviour is tested as routinely as singleplayer.

## Acceptance criteria

1. `shakedown session start --setup <a dedicated setup with clients = 2 and headless = 1>` returns exit code 0 with machines `server`, `p1`, `p2` and `hc1`, each `ready`.
2. In a check, `[] call skd_fnc_machines` returns `["server", "p1", "p2", "hc1"]`, and `["hc1"] call skd_fnc_player` and `["server"] call skd_fnc_player` return `objNull`.
3. The result file of a run against the session lists every machine in `session.machines` with its kind, start number, `game_version`, and the version of each mod that has a `patch` key as that machine loaded it.

## Next to

- [US-004](US-004-start-never-ready.md) (system fail): the start never becomes ready.
- [US-005](US-005-second-session-refused.md) (mistake): a second session is started while this one runs.
