---
type: Deferred
title: "A client that hangs in its start holds session start for the whole start ceiling"
description: "A game process that runs but never connects during a start is seen only when the start ceiling of 180 s passes, so session start waits the full ceiling before it fails."
id: DEF-28
state: open
milestone: null
status: stable
---

# A client that hangs in its start holds session start for the whole start ceiling

## Summary

`SES-04` ends a start `failed` at the start ceiling, 180 s by default (`NFR-PF-06`), or at once when a process exits. A game process that hangs in its start, so that it runs but its extension never connects, does neither until the ceiling passes. `session start` then waits the whole ceiling before it fails, though the hang was there from the first seconds.

## Current behaviour

The runner waits for the machines in `wait_ready` (`runner/session.rs`), the loop behind `P-02` steps 6 to 8. On each turn it ends the start early only for a process that exited (`win::exited`), with "`<machine>` exited with code `<n>` before it was ready" or, for code 53, the Steam message. A process that still runs is only waited on: its machine stays `launched` (the machine status machine of `spec/01-domain-model.md`: "The process runs; its extension has not connected"), and nothing compares how long it has been `launched` with how long a machine takes to connect. Only when `start_timeout` passes does the loop fail the start with "the session was not ready within the start ceiling of 180 s", naming each machine not `ready` and its state, the reason's machine being the one that got least far (`SES-04`), and `P-10` then stops every process (`SES-04`, `US-004`). The same loop brings machines back for a mission restart and a game restart (`P-08`), with the same ceiling.

A dedicated start of a server, two clients and a headless client is `ready` in 43 to 45 s on the test machine (`docs/dedicated-sessions.md`), so a hung client is found about 135 s after the start would have been ready. The prior harness met such hangs: three of five singleplayer starts with ACE stalled at different steps, and one start hung while addons loaded (research `17-health.md`).

## The gap

`US-004` promises that "a stuck start never holds the machine". A hung client holds it, with its memory and the game ports, for the whole ceiling, and the person or agent who started it waits as long for an answer the runner could give much sooner. Lowering `start_timeout` to shorten the wait trades it for false failures on a slow but healthy start.

## Options

1. A connect ceiling per machine: a machine still `launched` a set time after its own launch ends the start `failed`, naming it, its state `launched` and its RPT tail, as the start ceiling does. The time is set from measured connect times, well above the slowest seen.
2. Watch the hung process itself: a window that stops answering messages, or a modal error dialog of the game, ends the start at once. This catches a hang only when it shows on a window, and a dedicated server and a headless client have none.
3. Keep the behaviour and document that a hung start costs the whole ceiling, pointing to `start_timeout`.
4. Connect the extension earlier: the framework's addon calls the extension from a `preStart` function, which runs once the game has loaded its addons and before any mission, so `connected` comes at the end of the game's own start instead of at the session mission's `postInit`. The connect ceiling of option 1 then covers only the engine's start, which the hangs measured in `SPK-20` fell in, and a ceiling from `connected` to `ready` covers the mission's load.

## Recommendation

The fourth, then the first. `SPK-20` (M16, research `32-spike-connect-times.md`) measured the time from launch to `connected` with the extension connecting at the session mission's `postInit`, as built: a healthy singleplayer start with CBA and ACE took up to 90 s to connect, a dedicated client up to about 40 s, so a connect ceiling of twice the slowest would reach the start ceiling and the spike failed. In the same series one singleplayer start hung while the engine chose its graphics adapter and held the machine for the whole 180 s. The kinds of a dedicated session would have passed on their own (slowest server 28 s, client 39 s, headless client 35 s). The singleplayer starts' RPTs show where the time goes: the engine took 11 s from launch to its graphics start when warm, 29 s on the first start and 42 s on the slowest, and the mission's load another 16 to 40 s. With the connect moved to `preStart`, `connected` comes after the engine's start but not the mission's load, which shortens the slowest time by less than half; whether twice it is then under 90 s is measured again, as `SPK-20` was, before a number is set.

For option 1 on its own: It needs no window, holds for every kind of machine and keeps `SES-04` as the outer bound. Before its number is set, measure the time from launch to `connected` per kind of machine over a series of starts, cold and warm, and set the connect ceiling from it in `NFR-PF-06`. A matching ceiling from `connected` to `ready` can follow if a hang after the connect shows up in use.

## Acceptance criteria

1. A start in which one game process runs but its extension never connects ends `failed` with exit 4 within the connect ceiling of that machine's launch plus 5 s, well before the start ceiling, and the reason names that machine, its state `launched` and the last 20 lines of its RPT (`SES-04`).
2. No game process of that start runs after `session start` returns, and a new `session start` is accepted (`US-004` criterion 3).
3. Over 20 starts of each setup kind on the test machine, cold and warm, no healthy start fails at the connect ceiling.
4. `SES-04`, `P-02` step 8, `NFR-PF-06` and `docs/sessions-and-the-runner.md` state the connect ceiling.
