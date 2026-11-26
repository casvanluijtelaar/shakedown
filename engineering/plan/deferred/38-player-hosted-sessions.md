---
type: Deferred
title: "No player-hosted session"
description: "A setup is singleplayer or dedicated, so a mod's behaviour on a player who is also the server is never tested; no source shows how to start a host and its mission with nobody at the keyboard, so the mode waits behind spike SPK-24, and building it would add a mode, a machine kind and new machine names to the contract."
id: DEF-38
state: open
milestone: null
status: draft
---

# No player-hosted session

## Summary

A player-hosted session is one game client, `arma3_x64.exe`, that is the server and a player at once, with other clients joining it. A mod meets it whenever players host a game themselves, and code that runs on the server and on a player's machine in the same process fails there in ways neither singleplayer nor a dedicated session shows. Gap H9 keeps it out of the framework until a spike shows an unattended way to start one ([research 38](../../research/38-player-hosted-start.md)). No story promises it, and the PRD names singleplayer and dedicated only.

## Current behaviour

- `setups.<name>.mode` is `"singleplayer"` or `"dedicated"` (`contract/cli.md`). Any other value is refused when the project file is read.
- A dedicated session's server is `arma3server_x64.exe` with `-config` and `-autoInit`, and its mission comes from `class Missions` in the `server.cfg` the runner writes (`INT-AR-02`, `INT-AR-05`). No session has a machine that is both the server and a player, except singleplayer's `p1`, where `server` names `p1` (`DM-10`).
- `00-overview.md` lists the player-hosted session as out of scope (H9).

## The gap

A check cannot run against a host. The machine kind where `isServer` and `hasInterface` both hold, with clients joined to it, is not tested. Locality, `remoteExec` to the server, and a mod's server-side code running beside its player-side code in one process all behave differently there.

No source shows how to start such a host and load its mission with no person at the machine. Research 38 finds four candidate ways, none documented to work unattended:

- `-host` with `-config` and `-autoInit`. The wiki documents `-config` and `-autoInit` for the dedicated server only.
- The framework's addon pressing the multiplayer displays' OK buttons with `ctrlActivate`. A trial of `ctrlActivate` on a client whose window did not render did nothing (research 33).
- `hostMission`, which must be called in a UI action and takes a config class, not the session's mission folder.
- Eden's `MissionPreviewMP`.

Without a server config a host also has no headless clients, takes no client that loads unpacked data, and is not known to stay loopback-only (research 38, section 1).

## Options

1. **Leave it out.** H9 stays. A mod that must work on a hosted game is tested by a person.
2. **A `hosted` mode through `-host -config -autoInit`.** The start is a dedicated start with the host as the server, and nothing runs before the mission. It needs the client executable to read `-config` and honour `-autoInit`.
3. **A `hosted` mode through the addon driving the menus.** A `preStart` function of the framework's addon presses the multiplayer displays' buttons, inert unless `status` returns a session. It needs the menus to act on `ctrlActivate` with nobody at the machine, and it puts code that hosts a game into every game that loads `@shakedown`.
4. **A `hosted` mode through Eden's preview.** The host starts with the session mission in the editor, and the addon calls `MissionPreviewMP`. Every mission restart and reload then goes through the editor.

Keys or clicks sent from outside the game are not an option: `CLAUDE.md` forbids them, and `SPK-11` failed.

## Recommendation

Run the spike below as a research round ahead of any milestone that would build the mode (implementation plan, rule 18), when the owner wants hosted sessions tested. Try the options in the order 2, 3, 4 and build the first that passes. If none passes, take option 1 and keep H9.

If a way passes, the contract needs these changes, made first in the contract files:

- **`setups.<name>.mode`** in `contract/cli.md` gains `"hosted"`. `clients` counts every machine with a player, the host included. `headless` and `late` are allowed only if the host reads a server config. `start_timeout` holds.
- **`skd_fnc_setup`** in `contract/check-api.md` returns `"mode"` as `"singleplayer"`, `"dedicated"` or `"hosted"`. `skd_fnc_player "server"` returns the host's player on a hosted session.
- **The result file's setup**: `session.setup.mode` in `contract/result-file.md` and its enum in `contract/result.schema.json` gain `"hosted"`. So does the `mode` of the Session object in `contract/mcp-tools.md`.
- **The machine targets**: on a hosted session the host is both `server` and `p1`, as `server` names `p1` in singleplayer, and the joining clients are `p2` to `pN`. `all` and `clients` start with `p1`, and the host is listed once. The lead machine is the host. That touches `DM-10`, `DM-12`, the lead machine in `00-overview.md`, and the machine names in `check-api.md`.

The spec would also change:

- `SES-01`, `SES-03` (the host's ports), `SES-09`, `INT-AR-02` (a host row), `INT-AR-04` (where the host's mission goes) and `INT-AR-05` (whether the host gets a `server.cfg`).
- `SEC-05`: the host must stay loopback-only with a password.
- `P-01`: the host first, then every client at once.
- `STP-04`.
- The reload and the reset: a mission restart on a host is untried.

Under option 3 the spec would also need `INT-AR-03` to say that the host's extension connects in the main menu.

**Spike (`SPK-24`).** On a Windows machine with Arma 3 and Steam running, the desktop shown and nobody giving input, the change built into a scratch branch, with `@skd_fixture`:

1. For each way, in the order of the options, start a host and two game clients 10 times through the runner.
2. Record each start's display sequence on the host from its RPT and the named pipe's `session` answer, each machine's launch-to-`connected` and launch-to-`ready` time, and whether the host takes a connection from outside the machine.
3. On each ready session, run a check whose body runs on the host and executes code on `p2`, asserting a value on each.
4. Stop the session.
5. Once, start a game that loads `@shakedown` with no runner. The addon must leave it in the main menu.

It passes when the runner starts a host and two game clients with no person, the host loads the session mission, every machine's extension connects, a check runs on the host and on a client and passes, and every process is stopped by its process id, 10 of 10 within the start ceiling of `SES-04`. A start that hangs counts against it. If it fails, H9 stays, with each way tried and why it failed written into research 38. Nothing sends keys from outside the game.

These would show a regression: every end-to-end test that starts a singleplayer or dedicated session, since the change touches `INT-AR-02`, the start sequence and, under option 3, code in the framework's addon that runs in the main menu; the M15 hand-over tests; `us_088_sec_05_a_client_sent_to_another_server_is_ended`; and the acceptance suite's checks on game clients.

## Acceptance criteria

- `SPK-24` has run, and its result is written up in research 38: which way passed, or each way tried and why it failed.
- If a way passed: `session start --setup <a hosted setup>` with `clients = 3` is `ready` with the host as `server` and `p1` and two clients as `p2` and `p3`, 10 of 10 within `SES-04`'s ceiling; a check runs on `server` and on `p2` and passes; `session stop` leaves no process, port or mission copy (`STP-04`); and `skd_fnc_setup`, the result file and `status` report the mode `hosted`.
- A game that loads `@shakedown` with no runner is not hosted and stays in the main menu.
- The hosted session takes no connection from outside the machine (`SEC-05`).
- The contract files and the spec say what the recommendation names before the code is written. Otherwise H9 stays and this item is closed as option 1.
