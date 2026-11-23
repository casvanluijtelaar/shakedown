---
type: Research
title: "Moving a client to another server from a check"
description: "What SQF code sent to a dedicated session's game client can do to make it join another server, tried on a Windows machine in M16 against a second loopback server: connectToServer, endMission and the multiplayer pause menu's Abort, none of which took the client off the session's server while no one was connected to the machine's desktop."
status: draft
generated: { by: "claude-code/developer", at: 2026-11-20T07:10:00Z }
sources:
  - { id: trial, resource: "The M16 developer round's trials with shakedown exec on p2 of a dedicated session with two clients and @skd_fixture, beside a second arma3server_x64 named other on port 2432 with loopback = 1, and p2's RPT", read: 2026-11-20 }
  - { id: test, resource: "crates/shakedown/tests/m16.rs, us_088_sec_05_a_client_sent_to_another_server_is_ended", read: 2026-11-20 }
---

# Moving a client to another server from a check

`SEC-05` leaves one way for a session's client to reach a server that is not the session's own: code a check sends. M16 built the guard for it, the in-game agent comparing `serverName` with `shakedown <session id>` at every mission start, and tried to drive that path on a Windows machine[^trial], with a second dedicated server named `other` on loopback beside a session's own, both loading `@skd_fixture` and `@shakedown`.

## What was tried

All on `p2`, through `shakedown exec`, with no one connected to the machine's desktop, so the client windows did not render:

| Code | What `p2` did |
|---|---|
| `connectToServer ["127.0.0.1", 2432, ""]` | Nothing: it stayed in the session's mission, `serverName` unchanged, no line in its RPT. |
| A script spawned in `uiNamespace` that waits for display 46 to close, then `connectToServer`, and `(findDisplay 46) closeDisplay 0` | Display 46 was already null on a client with no rendered window; the script ran `connectToServer` at once, with no effect. |
| The same script waiting for `getClientStateNumber` below 9, and `endMission "END1"` | The client state stayed 11 for 60 s; `connectToServer` after it had no effect. |
| `createDialog "RscDisplayMPInterrupt"` and `ctrlActivate` on its Abort button (idc 104, text `ABORT`) | The dialog opened and the button was enabled; activating it did not take the client off the server in 60 s. |

The scripts spawned in `uiNamespace` kept running past each attempt (their `diag_log` lines are in `p2`'s RPT), and the session stayed `ready`, every machine answering.

## What it means

`connectToServer` acts only outside a mission. Without a rendered window, nothing tried here took a client out of its mission, so code a check sends could not make a client join another server on a Windows machine as the framework runs it. The guard of `SEC-05` stays as built: it costs one comparison at each mission start, and the end-to-end tests of M16 show it does not fire on the session's own server. Whether the Abort path works with a visible desktop, so the window renders, is left to the owner look of `US-088`, which runs the test[^test] that drives it.

[^trial]: Trials on 2026-11-20 between 06:58 and 07:06 UTC, session `s-20261120-065803`.
[^test]: The test aborts `p2` through the pause menu and joins `other` from a script spawned in `uiNamespace`.
