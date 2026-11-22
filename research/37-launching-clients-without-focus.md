---
type: Research
title: "Starting game clients without taking the foreground"
description: "How Shakedown starts its game client windows, which Windows rules let a new window take the foreground and which start options keep it out, what Arma 3 does without focus or minimised as far as the repository's records and Bohemia's pages show, which features need a rendered window, and the one-field change and spike that would keep client windows from taking the screen."
status: draft
generated: { by: "claude-code/developer", at: 2026-11-22T07:28:09Z }
sources:
  - { id: launch, resource: "crates/shakedown/src/runner/session.rs (Session::launch, the client options) and crates/shakedown/src/win.rs (Job::spawn, spawn_breakaway, set_window_title) at a225174", read: 2026-11-22 }
  - { id: intar, resource: "spec/04-integrations.md, INT-AR-02, INT-WIN-01, INT-WIN-02, INT-WIN-04", read: 2026-11-22 }
  - { id: startup, resource: "https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters (oldid 373124, Wayback snapshot 2025-04-09; the live wiki answered every request with a Cloudflare challenge on 2026-11-22), entries window, posX, posY, limitFPS, noPause, noPauseAudio", read: 2026-11-22 }
  - { id: focus, resource: "https://community.bistudio.com/wiki/isGameFocused (oldid 309152, Wayback snapshot 2024-12-04)", read: 2026-11-22 }
  - { id: reforger, resource: "https://community.bistudio.com/wiki/Arma_Reforger:Startup_Parameters (oldid 370846, Wayback snapshot 2025-03-31), entry noFocus", read: 2026-11-22 }
  - { id: t85707, resource: "https://feedback.bistudio.com/T85707 (Wayback snapshot 2021-03-08), 'clicks and keystrokes while not having focus', open since 2016-01-10", read: 2026-11-22 }
  - { id: setfg, resource: "https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow (ms.date 2025-10-06)", read: 2026-11-22 }
  - { id: lockfg, resource: "https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-locksetforegroundwindow (ms.date 2018-12-05)", read: 2026-11-22 }
  - { id: spi, resource: "https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow (ms.date 2024-03-27), SPI_GETFOREGROUNDLOCKTIMEOUT and SPI_GETFOREGROUNDFLASHCOUNT", read: 2026-11-22 }
  - { id: showwin, resource: "https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow (ms.date 2023-06-06)", read: 2026-11-22 }
  - { id: startupinfo, resource: "https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-startupinfow (ms.date 2018-12-05)", read: 2026-11-22 }
  - { id: reg, resource: "reg query 'HKCU\\Control Panel\\Desktop' on the Windows test machine: ForegroundLockTimeout 0x30d40, ForegroundFlashCount 0x7", read: 2026-11-22 }
  - { id: r17, resource: "research/17-health.md, section 2", read: 2026-11-22 }
  - { id: tclock, resource: "arma-grappling-hook@4ad9127:tests/m00-let-go-clock/README.md and focus.ps1", read: 2026-11-22 }
  - { id: tgun, resource: "arma-grappling-hook@4ad9127:research/tests/grapple-gun-no-ammo/README.md, the dedicated row on gun B", read: 2026-11-22 }
  - { id: r32, resource: "research/32-spike-connect-times.md", read: 2026-11-22 }
  - { id: r33, resource: "research/33-client-to-another-server.md", read: 2026-11-22 }
  - { id: r35, resource: "research/35-spikes-ace-and-keys.md", read: 2026-11-22 }
  - { id: r36, resource: "research/36-spike-frame-rate-and-answer-time.md", read: 2026-11-22 }
  - { id: handover, resource: "docs/handing-over.md", read: 2026-11-22 }
---

# Starting game clients without taking the foreground

Every start of a game client puts its window on the test machine's screen and can take the foreground from whatever the owner is doing. This research answers whether the framework can start the clients without that, and whether a client out of focus or minimised still does everything the framework needs from it. No game was started for it: another round held the game. What it says about the game rests on the repository's own records and Bohemia's pages; what it says about Windows rests on Microsoft's.

## 1. How Shakedown starts a game client

- The runner starts every game process with `CreateProcessW`, suspended, assigns it to the session's job object and resumes it (`STP-01`, `INT-WIN-01`). The `STARTUPINFOW` it passes sets only `cb`: no `STARTF_USESHOWWINDOW`, so the game shows its first window however it asks to[^launch][^intar].
- A game client, and singleplayer's `p1`, get `-window -noPause -noPauseAudio -noSound -skipIntro -noSplash -world=empty` after the common options (`INT-AR-02`). The server and headless clients run `arma3server_x64.exe` and have no game window[^launch][^intar].
- The runner itself runs detached: the client starts it with `DETACHED_PROCESS` and `CREATE_BREAKAWAY_FROM_JOB` through a relay that exits at once (`INT-WIN-02`). The game's parent is therefore a background process with no window that never received the owner's input[^launch][^intar].
- Once a client's extension connects, the runner retitles its visible windows `Arma 3 - <machine> (<session>)` with `SetWindowTextW`, trying for up to 30 s. The code says it touches no input, focus or foreground[^launch].
- `INT-WIN-04` already says no part of the framework takes the foreground[^intar]. A client window that takes it on its start is the game doing so, but on the framework's start; the spec does not say which side of that rule it falls on.

## 2. What decides whether a new window takes the foreground

- A process may set the foreground window only when no menu is active, the foreground process has not locked it with `LockSetForegroundWindow`, and at least one of these holds: the foreground lock time-out has expired, the caller is the foreground process, it was started by the foreground process, there is no foreground window, it received the last input event, or the foreground or the caller is being debugged. Even then it may be refused. Otherwise "Windows flashes the taskbar button of the window"[^setfg].
- The foreground lock time-out is "the amount of time following user input, in milliseconds, during which the system will not allow applications to force themselves into the foreground"[^spi]. On the test machine it is 200,000 ms, 200 s, and a refused switch flashes the taskbar button 7 times[^reg].
- `LockSetForegroundWindow` can be called only by the foreground process, and the lock lifts when the user presses Alt or the system changes the foreground itself[^lockfg]. The runner is never the foreground process, so it cannot lock the owner's window in place.
- **Inference.** The game's parent, the runner, is neither the foreground process nor the receiver of the last input. So a client can take the foreground on its own only when the owner has given no input for 200 s, or when nothing is in the foreground. On a machine where the owner watches an agent work, 200 s without a key or a click is the usual case, which fits the windows taking the screen. While the owner types, the same start should flash the taskbar instead. Neither case has been watched on the test machine **(unverified)**.
- The show state of a GUI program's first window comes from the starter: with `STARTF_USESHOWWINDOW`, "the first time ShowWindow is called, its nCmdShow parameter is ignored [and] wShowWindow specifies the default value"[^startupinfo]. `ShowWindow` says the same, and that it holds whether the program creates its main window with `WS_VISIBLE` or shows it later with `SW_SHOW`[^showwin].
- The show states that do not activate the window are `SW_SHOWNOACTIVATE` (4), shown at its last size and position, `SW_SHOWMINNOACTIVE` (7), "displays the window as a minimized window ... except the window is not activated", and `SW_SHOWNA` (8)[^showwin].
- The start state covers the first `ShowWindow` only. A later `ShowWindow` with another state, a `SetForegroundWindow` call or a second top-level window is the game's own and is bound only by the foreground rules above[^showwin][^setfg]. Whether Arma 3 makes such a call after its first window is not documented anywhere read **(unknown)**.

## 3. Arma 3's own options

- `-window` "Displays Arma 3 windowed instead of full screen"; the size comes from `arma3.cfg`. `-posX` and `-posY` set "the default X/Y Position of the Window". `-noPause` lets the game run "even when its window does not have focus (i.e. running in the background)". `-noPauseAudio` keeps the audio running while tabbed out and goes with `-noPause`[^startup].
- The page names no option that keeps the game from taking the focus, and no `-x` or `-y`[^startup]. Bohemia's newer engine has one: Arma Reforger's `-noFocus` "prevents window focus stealing on game initialization"[^reforger]. That Reforger needed an option for it suggests the engine family takes the focus on its start; for Arma 3 that is **(inference)**.
- "The FPS gets automatically limited in the main menu with `-world=empty` or when windowed mode is unfocused ('tabbed out')"[^startup].
- `isGameFocused` "returns the state of game focus changed by pressing ALT + Tab or similar action"[^focus].
- Moving the window off screen with `-posX`/`-posY` does not stop it being activated, so it answers a different question **(inference)**.

## 4. Does it matter for what the framework does

### Pausing and throttling

- **Known.** With `-noPause` the game runs without focus[^startup]. In the prior harness's clock test the game was minimised on purpose with `ShowWindow(SW_MINIMIZE)` while the owner was connected: `isGameFocused` was false, the frame rate about 32 and mission time ran on[^tclock][^r17]. Every client Shakedown starts already has `-noPause`[^launch].
- **Known.** Without focus a windowed game limits its frame rate[^startup]; the 32 a second minimised[^tclock] against about 58 shown[^r36] is consistent with that.
- **Unknown.** The wiki says "does not have focus"; it does not say minimised[^startup]. The one minimised sample is singleplayer, one let go, minimised after the start by another program, with the stream connected[^tclock]. A client started minimised, on a dedicated session, with the stream cut, has not been tried.

### Rendering, scripts, heartbeats and the frame-rate watch

- **Known.** A client nobody's remote desktop shows ran at 17.7 to 19.7 frames a second with mission time at the rate of real time, and answered pings in 0.002 to 0.005 s[^r36]. The frame-rate floor of `HLT-01` is 10, set to stay under that[^r36]. A minimised client with the stream connected ran at about 32[^tclock].
- **Known.** Scripts, the heartbeat and `exec` orders ran on clients whose windows did not render: research 33's whole trial ran with nobody connected, and its scripts in `uiNamespace` kept running[^r33]. The watch counts frames by the heartbeats' `diag_frameNo`, and it counted 17.7 to 19.7 a second on a client no remote desktop showed, so frames go on counting while nothing reaches a screen[^r36].
- **Unknown.** The frame rate of a client that is both minimised and unseen. If it fell under 10, every check on that client would end `invalid` / `unhealthy` with `frame-rate`. The two known numbers, 32 minimised and 19 unseen, both clear the floor; their combination is the first thing a spike must measure.

### Features that need a rendered window

- **The hand-over (M15, `MAN-01`).** The person looks through `p1`'s window, and the box of things to look for is drawn in it[^handover]. A minimised `p1` must be restored first, by the person clicking it on the taskbar, which is a person's own input and breaks no rule. The hand-over's message does not tell them to **(gap)**. A hand-over could also restore the window itself with `SW_SHOWNOACTIVATE`, which shows it without activating it[^showwin]; whether that is needed is the owner's call.
- **`US-088`, a client sent to another server.** Without a rendered window display 46 was already null and nothing tried took the client out of its mission; the owner look with a rendered window is still to run[^r33]. A minimised window is likely as unrendered as an unseen one **(inference)**, so that look needs `p2` restored before the test drives it.
- **Screenshots (`SHT-01`, a could).** Not built. A minimised window has no client area to capture **(inference)**; a screenshot check would have to restore its window first.
- **The owner's looks.** Any look in a client's window starts with restoring it. That is one click per window.

### Input the owner gives elsewhere

- **Known.** Bohemia's tracker has an open report, reproducible always: a windowed Arma 3 that loses the focus while it loads enters a "half focused" state in which it is not on screen but "plays sounds and catch[es] HID actions as if it were in focus", so keys and clicks in another program reach the game too. The report's steps start the game without `-noPause`[^t85707]. The game reads the keyboard through DirectInput[^r35].
- **Known.** The test machine's keep-alive key reached the prior harness's game as a key event[^tclock][^r17].
- **Unknown.** Whether a client Shakedown starts minimised, with `-noPause`, catches the owner's keys while the owner types in another program. If it does, the owner's typing could move a player in the middle of a check. This is a risk today too, whenever a client loses the focus while it loads, and the spike should look for it either way.

### Two clients and their focus

- **Unknown.** A dedicated session starts every client at once[^launch]. No record says whether two client windows take the foreground from each other, once or over and over. The rules of section 2 let only one of them win at a time, and only while the lock time-out has run out[^setfg]. A start state that never activates either window takes the question away.

### Other records

- **Weak.** In the prior harness, scripted hooks fired from a client whose window was in the background were deleted the same second with no `HitPart`, while one fired at another time hit the wall. The cause was never found[^tgun]. It is one manual run and may have nothing to do with focus, but it is the one record of a game behaving differently in the background, and a check of `arma-grappling-hook`'s projectiles is where a change would show.
- **Unknown.** Whether a window started minimised slows or breaks the engine's graphics start. Research 32 measured the graphics start at 16 to 40 s and saw one start hang choosing its graphics adapter with the window shown[^r32]; a minimised window has a client area of no size **(inference)**, and how the engine creates its swap chain for it is not documented **(unknown)**.

## 5. Ways to keep the windows off the screen

| Way | What it does | Cost | Against |
|---|---|---|---|
| A. `STARTF_USESHOWWINDOW` with `SW_SHOWMINNOACTIVE` in the `STARTUPINFOW` of the game clients | The first window comes up minimised and never activated[^startupinfo][^showwin] | Two fields in `Job::spawn` for `arma3_x64.exe` | Holds only for the first `ShowWindow`; the game may activate itself later (section 2). Every look starts with a click on the taskbar. |
| B. The same with `SW_SHOWNOACTIVATE` | The window shows at its size and place, not activated[^showwin] | The same | The window still covers the screen; whether it comes up on top of the owner's window is not documented. |
| C. The runner minimises each client window with `ShowWindow(SW_SHOWMINNOACTIVE)` once it finds it, as it retitles it | Catches a window the game shows or activates after its start | A call beside `set_window_title` | The window is on screen, maybe in the foreground, until the runner finds it; the clock test's `focus.ps1` did this and the game ran on[^tclock]. |
| D. A longer `ForegroundLockTimeout` on the test machine | Keeps any background program from taking the foreground for longer after input[^spi] | A registry value, the owner's own | Changes the machine for every program, not the product; does nothing for a game the lock does not cover. |
| E. `LockSetForegroundWindow` | Locks the foreground in place | — | Only the foreground process may call it, and the runner never is[^lockfg]. Not possible. |
| F. Off screen with `-posX`/`-posY` | Moves the window | An option | Still activated; still takes the keyboard. |

A and C are both outside the rule against moving the mouse or typing: neither sends input, and neither makes a window the foreground (`INT-WIN-04`). A is the cheapest and the one to try first; C is its fallback if the game shows or activates its window again after the start.

## 6. Recommendation and spike

**Change, if the spike passes:** start every `arma3_x64.exe` with `STARTF_USESHOWWINDOW` and `SW_SHOWMINNOACTIVE` (way A). Nothing else changes: the options of `INT-AR-02` stay, `-noPause` stays required. It needs `INT-AR-02` or `INT-WIN-01` to name the start state, `INT-WIN-04` to say that the game windows the framework starts do not take the foreground either, and `docs/handing-over.md` to tell the person to restore the window from the taskbar. If the game reactivates its window after the start, add way C. If the spike shows a minimised client falls under the frame-rate floor or misses key behaviour, use B, which keeps the window shown and still unactivated, and measure again.

**Spike (proposed as `SPK-23`).** On the test machine, with Steam running, the change built into a scratch branch:

1. With the owner connected and a window of another program in the foreground, and again after the owner has given no input for more than 200 s, start 10 dedicated sessions (a server, two clients, a headless client) and 10 singleplayer sessions. A helper samples `GetForegroundWindow` every 100 ms from before each start until `ready` plus 30 s, and the minimised state of each client window.
2. On one dedicated session and one singleplayer session, run 50 passing checks each, as `SPK-22` did, sampling every machine's `health` from `status`; repeat the dedicated run with the stream disconnected.
3. While one check holds `p1` for 30 s, the owner types into another program's window; a display-46 `KeyDown` handler on `p1` logs every key.
4. The owner restores `p1` from the taskbar during a hand-over of `crates/shakedown/tests/m15.rs` and finds the box in it.

Pass when: no sample shows an `arma3_x64` window in the foreground and every client window stays minimised until restored; every session is `ready` within the start ceiling with launch-to-connected times inside `SPK-20`'s range[^r32]; no machine is flagged and every client's 5 s frame rate stays at or above 10, with the stream connected and disconnected; all 100 checks pass; the `KeyDown` handler logs no key; and the box shows in the restored window. A start that hangs counts against it.

**What would show a regression:** `spk_22_frame_rate_and_answer_time_over_50_passing_checks` and `spk_15_heartbeats_over_50_passing_checks` (the frame-rate and clock watch on healthy machines), the M15 tests `us_083_us_084_man_01_run_04_a_hand_over_keeps_the_session_for_the_manual_tester` and `us_083_m15_1_m15_2_man_01_the_box_leaves_the_action_menu_free_and_p1_opens_the_crate` (the hand-over), `us_088_sec_05_a_client_sent_to_another_server_is_ended` with its owner look, the ACE tests of `crates/shakedown/tests/m17.rs`, the framework's other end-to-end tests that start a dedicated session, and the acceptance suite's checks that run on game clients.

**Forces:** none yet. The change waits on the owner and the spike. If it passes, `INT-AR-02` (or `INT-WIN-01`), `INT-WIN-04` and `docs/handing-over.md` change as above, and the hand-over and `US-088` looks start with restoring the window.

[^launch]: `Session::launch` builds the options; `Job::spawn` starts the process with a `STARTUPINFOW` that sets only `cb`; `spawn_breakaway` starts the runner detached; `set_window_title` retitles the window.
[^intar]: `INT-AR-02`, `INT-WIN-01`, `INT-WIN-02`, `INT-WIN-04`.
[^startup]: Bohemia Interactive, Arma 3: Startup Parameters.
[^focus]: Bohemia Interactive, isGameFocused.
[^reforger]: Bohemia Interactive, Arma Reforger: Startup Parameters, `-noFocus`.
[^t85707]: Bohemia Feedback Tracker T85707.
[^setfg]: Microsoft, SetForegroundWindow function, Remarks.
[^lockfg]: Microsoft, LockSetForegroundWindow function.
[^spi]: Microsoft, SystemParametersInfo, `SPI_GETFOREGROUNDLOCKTIMEOUT`.
[^showwin]: Microsoft, ShowWindow function.
[^startupinfo]: Microsoft, STARTUPINFOW structure, `wShowWindow`.
[^reg]: The test machine's `HKCU\Control Panel\Desktop`, read on 2026-11-22.
[^r17]: Research 17, section 2.
[^tclock]: The prior harness's clock test, its focus round of 2026-11-03.
[^tgun]: The prior harness's test `grapple-gun-no-ammo`.
[^r32]: Research 32, `SPK-20`.
[^r33]: Research 33.
[^r35]: Research 35, `SPK-11`.
[^r36]: Research 36, `SPK-22`.
[^handover]: Handing over to a person.
