---
type: Deferred
title: "Game client windows take the foreground when they start"
description: "Every game client the framework starts shows its window and can take the foreground from whatever the person at the machine is doing; starting the clients minimised and never activated would stop that, behind a spike, and the item is optional because it is a nuisance to the person, not a defect in what a check proves."
id: DEF-34
state: open
milestone: null
status: draft
---

# Game client windows take the foreground when they start

## Summary

The runner starts every `arma3_x64.exe` with a `STARTUPINFOW` that sets no show state, so each client window, and singleplayer's `p1`, comes up however the game asks and can take the foreground from the program the person at the machine is using ([research 37](../../research/37-launching-clients-without-focus.md), sections 1 and 2). This item is optional. A window that takes the foreground is a nuisance to whoever uses the machine while a run goes on; it does not change what a check proves, and no story promises otherwise. It waits until the nuisance is worth a spike.

## Current behaviour

- `Job::spawn` passes a `STARTUPINFOW` that sets only `cb`, with no `STARTF_USESHOWWINDOW` (`INT-WIN-01`). Every client gets `-window -noPause -noPauseAudio` among its options (`INT-AR-02`), so a client without focus keeps running.
- The runner is a detached background process, never the foreground process and never the receiver of the last input (`INT-WIN-02`). By the Windows foreground rules a client it starts can take the foreground when the person has given no input for the foreground lock time-out, 200 s by default, or when nothing is in the foreground; while the person types, Windows should flash the taskbar button instead. Neither case has been watched (research 37, section 2).
- On a dedicated session every client starts at once, so two windows may take the foreground from each other.
- `INT-WIN-04` says no part of the framework takes the foreground; it does not say whether a game window the framework starts falls under that rule.
- A windowed Arma 3 that loses the focus while it loads can enter a state in which keys typed in another program still reach it (Bohemia tracker T85707, research 37, section 4). Whether that happens to a client Shakedown starts is unknown, minimised or not.

## The gap

Nothing keeps a client window from taking the screen and the keyboard from the person at the machine during a run. The framework's own code sends no input and makes no window the foreground; the game's first window does, on the framework's start.

## Options

1. **Start minimised and not activated.** Set `STARTF_USESHOWWINDOW` with `wShowWindow = SW_SHOWMINNOACTIVE` in the `STARTUPINFOW` of every `arma3_x64.exe`. Windows applies it to the game's first `ShowWindow`, so the window comes up minimised and never active. It is two fields in `Job::spawn`. It does not bind a later `ShowWindow` or `SetForegroundWindow` the game makes itself, and every look in a client window starts with a click on its taskbar button.
2. **Start shown and not activated.** The same with `SW_SHOWNOACTIVATE`. The window shows at its size and place without activation; it still covers part of the screen.
3. **Minimise once found.** The runner calls `ShowWindow(SW_SHOWMINNOACTIVE)` on each client window when it finds it, beside `set_window_title`. It catches a window the game shows or activates after its start, but the window is on screen until the runner finds it.
4. **Leave it.** The windows go on taking the foreground; the person works around a run.

A longer foreground lock time-out, `LockSetForegroundWindow` and moving the window off screen with `-posX`/`-posY` do not answer it (research 37, section 5): the first changes the machine for every program, the second only the foreground process may call, and the third still activates the window.

## Recommendation

Option 1, gated by the spike below, when the owner decides the nuisance is worth it: start every `arma3_x64.exe` with `STARTF_USESHOWWINDOW` and `SW_SHOWMINNOACTIVE`, and keep the options of `INT-AR-02` with `-noPause` required. If the game shows or activates its window again after the start, add option 3: the runner minimises each client window without activating it once it finds it. If a minimised client falls under the frame-rate floor of `HLT-01` or misses key behaviour, use option 2 and measure again. Neither sends input or makes a window the foreground, so both stay inside `INT-WIN-04`.

The change needs `INT-AR-02` or `INT-WIN-01` to name the start state, `INT-WIN-04` to say that the game windows the framework starts do not take the foreground either, and `docs/handing-over.md` to tell the person to restore `p1` from the taskbar before looking through it.

**Spike.** On a Windows machine with Arma 3 and Steam running, the change built into a scratch branch:

1. With a window of another program in the foreground and a person giving input, and again after more than 200 s with no input, start 10 dedicated sessions (a server, two clients, a headless client) and 10 singleplayer sessions. A helper samples `GetForegroundWindow` every 100 ms from before each start until `ready` plus 30 s, and the minimised state of each client window.
2. On one dedicated session and one singleplayer session, run 50 passing checks each, as `SPK-22` did, sampling every machine's `health` from `status`.
3. While a check holds `p1` for 30 s, the person types into another program's window; a display-46 `KeyDown` handler on `p1` logs every key.
4. The person restores `p1` from the taskbar during a hand-over of `crates/shakedown/tests/m15.rs` and finds the box in it.

It passes when no sample shows an `arma3_x64` window in the foreground and every client window stays minimised until restored; every session is `ready` within the start ceiling with launch-to-connected times inside `SPK-20`'s range; no machine is flagged and every client's 5 s frame rate stays at or above 10; all 100 checks pass; the `KeyDown` handler logs no key; and the box shows in the restored window. A start that hangs counts against it.

What would show a regression: `spk_22_frame_rate_and_answer_time_over_50_passing_checks`, `spk_15_heartbeats_over_50_passing_checks`, the M15 hand-over tests, `us_088_sec_05_a_client_sent_to_another_server_is_ended` with its owner look, the ACE tests of `crates/shakedown/tests/m17.rs`, the other end-to-end tests that start a dedicated session, and the acceptance suite's checks that run on game clients.

## Acceptance criteria

- The spike above passes on a Windows machine with a display, and its result is written up as research.
- No game client window the framework starts takes the foreground: a helper that samples `GetForegroundWindow` through 10 dedicated and 10 singleplayer starts finds no `arma3_x64` window in the foreground.
- The tests named under the recommendation pass with the change.
- `INT-AR-02` or `INT-WIN-01`, `INT-WIN-04` and `docs/handing-over.md` say what the recommendation names.
