---
type: Research
title: "Spike SPK-18: joining late, leaving and joining again"
description: "How long a third game client takes to join a running dedicated session, how a client can leave so that the server sees it go at once, that the same client joins again under its machine name, and what four game processes take of the test machine's memory, measured through the framework's own join and leave on the test machine in M11."
status: stable
generated: { by: "claude-code/developer", at: 2026-11-14T12:24:30-08:00 }
sources:
  - { id: tests, resource: "crates/shakedown/tests/m11.rs, spk_18_a_third_client_joins_leaves_and_joins_again_10_of_10, and the same check run ten times from the command line on branch developer/m11", read: 2026-11-14 }
  - { id: r14, resource: "engineering/research/14-sessions.md, section 3 and the spike of a lower DisconnectTimeout", read: 2026-11-14 }
  - { id: rpt, resource: "the server's RPT of the trial sessions on the test machine, its serverCommand lines", read: 2026-11-14 }
---

# Spike SPK-18: joining late, leaving and joining again

M11 ran `SPK-18` (`engineering/spec/06-open-questions.md`) on the Windows test machine (an AMD EPYC 9354 virtual machine with 16 GB of memory, Arma 3 2.22 from Steam) before the join and leave of `MCH-01` were settled. Its pass condition: on a dedicated session with two clients, a third game client joins and is `ready` within 60 s, 10 of 10, with the session's memory within the machine's; a client made to leave fires the server's `PlayerDisconnected` within the server's disconnect timeout plus 5 s, 10 of 10; and the same client, joining again under its machine name, is `ready` within 60 s, 10 of 10. It passed, with a way of leaving that neither candidate named, so no fallback applies: `MCH-01` and `US-067` stand, the default leave ceiling stays the disconnect timeout of 15 s plus 5, and the generated `server.cfg` keeps the default disconnect timeout.

## How it was measured

The measurement ran through the framework's own `skd_fnc_join` and `skd_fnc_leave`, built for it first, so it measured the path a check takes[^tests]. A dedicated session with `clients = 2` and `late = 1` loading `@skd_fixture` ran one check ten times in one run: `["p3", 60] call skd_fnc_join`, `["p3"] call skd_fnc_leave`, `["p3", 60] call skd_fnc_join`. The reset before each check made `p3` leave again, as `MCH-01` says. Each join and leave is a step of the check, and its duration in the result file is the measure. A join is `ready` as a start's client is: the agent sent `ready` and the server sees the user at client state 10 (`INT-AR-09`).

| Measure | Shortest | Median | Longest | Pass condition |
|---|---|---|---|---|
| A late `p3` joins (10) | 28.6 s | 29.6 s | 34.1 s | under 60 s, 10 of 10 |
| `p3` leaves, to the server's `PlayerDisconnected` (10) | 0.51 s | 0.52 s | 0.72 s | under 20 s, 10 of 10 |
| `p3` joins again under its name (10) | 27.7 s | 29.2 s | 31.8 s | under 60 s, 10 of 10 |
| Each check, join, leave and join with the reset | 59.7 s | 62.2 s | 68.6 s | - |

Every check passed. Each join took a new profile folder, `p3-<start>`, and the result file listed the 20 starts of `p3`, each with the version of `@skd_fixture` it loaded, 19 of them `stopped` and the last `ready`.

## Leaving

Three ways were tried; only the last is fast:

1. **Ending the client's process, then kicking its user by its user id.** The server logged each `#kick <id>` as a successful server command, once a second, and dropped the user only at its disconnect timeout: the leave took 15.0 s[^rpt]. This matches the crash in research `14-sessions.md`, about 15.6 s from the process's end to `PlayerDisconnected`[^r14]. It passes the condition (15 s plus 5), but every leave costs the whole timeout.
2. **Kicking the user by its user id while the process still runs, then ending it.** The server again logged the kick and did not drop the user; after 5 s the process was ended and the user went at the timeout, 20.1 s in all. The server takes no user id from `#kick` here, whatever it logs.
3. **Kicking the user by its profile name while the process still runs, then ending it.** The server dropped the user and fired `PlayerDisconnected` 0.063 s after `#kick p3` by hand; through the framework, kick, event and the process's end took 0.51 to 0.72 s. A kicked client whose process was left running had not joined again when the server's users were listed a few seconds later; the leave ends the process right after the event in any case, so it cannot.

The leave of `MCH-01` is the third way: the runner has the server kick the user by its profile name, which is the machine's name (`DM-11`), once a second until `PlayerDisconnected` comes for it, then ends the process. The first candidate of the spike, a clean leave from inside the client, was not tried: the engine's documented commands offer no disconnect from inside a client that this spike found, and the kick gives the server's own clean leave with no code in the client. A client that the kick does not drop within 5 s has its process ended and goes at the disconnect timeout, within the default ceiling of 20 s; a user the server no longer lists has gone already.

The second fallback, lowering `disconnectTimeout` to 5 s, is not taken: with the kick no leave waits for the timeout, and a lower timeout would drop a healthy client that stalls for a few seconds, which the clock watch of `HLT-02` should judge instead.

## Joining again under the machine's name

A client that left joins again under its name with no special step: a new process with `-name=p3` takes the first free playable slot (research `14-sessions.md`: `skipLobby = 1`)[^r14], the session mission's base respawn gives it a unit, and the server lists it by its profile name. So `skd_fnc_player` finds the new unit by name, as for any client. No kick blocked the rejoin: `kickTimeout[]` is 0 for every kind of kick in the generated `server.cfg` (`INT-AR-05`). The mission needs a slot per late client; `INT-AR-04` and `INT-AR-05` count them in `maxPlayers` and the playable units.

## Memory

With the server, `p1`, `p2` and the joined `p3` running, the four processes' working sets summed to 6.4 to 7.0 GB, sampled every 10 s over the run, and the machine kept 4.5 to 5.7 GB of its 16 GB free. A fourth client of the same mods would fit; a setup with ACE3 loaded needs more per process and was not measured here.

## What it means for the spec

- `MCH-01`: a leave is a kick by profile name, then the process's end, with the timeout as the path for a client the kick does not drop; the default leave ceiling stays the disconnect timeout plus 5 s, 20 s.
- `INT-AR-04`, `INT-AR-05`: a slot and a place in `maxPlayers` for every late client.
- A join's time, about 30 s, is most of a client's start, so a check that joins needs about 40 s of `@timeout` per join; `docs/dedicated-sessions.md` says so.

[^tests]: crates/shakedown/tests/m11.rs, and the ten runs of the same check from the command line
[^r14]: engineering/research/14-sessions.md
[^rpt]: the server's RPT of the trial sessions
