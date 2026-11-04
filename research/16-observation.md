---
type: Research
title: "Observing the game: waits, events, errors, logs, screenshots"
description: How a check waits for a condition with a ceiling, which engine and CBA events it can collect per machine with timestamps, how script errors are caught live and tied to a step, where each machine's log is, how screenshots are taken, and which performance probes exist.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T22:40:00+01:00 }
sources:
  - { id: waituntil, resource: "https://community.bistudio.com/wiki/waitUntil (Wayback snapshot 2025-02-26)", read: 2026-11-04 }
  - { id: time, resource: "https://community.bistudio.com/wiki/time (Wayback snapshot 2025-02-15)", read: 2026-11-04 }
  - { id: servertime, resource: "https://community.bistudio.com/wiki/serverTime (Wayback snapshot 2025-01-22)", read: 2026-11-04 }
  - { id: ticktime, resource: "https://community.bistudio.com/wiki/diag_tickTime (Wayback snapshot 2025-04-19)", read: 2026-11-04 }
  - { id: utc, resource: "https://community.bistudio.com/wiki/systemTimeUTC (Wayback snapshot 2025-04-20)", read: 2026-11-04 }
  - { id: frameno, resource: "https://community.bistudio.com/wiki/diag_frameNo (Wayback snapshot 2025-03-25)", read: 2026-11-04 }
  - { id: meh, resource: "https://community.bistudio.com/wiki/Arma_3:_Mission_Event_Handlers (live list, and Wayback snapshot 2025-03-27 for entry text)", read: 2026-11-04 }
  - { id: oeh, resource: "https://community.bistudio.com/wiki/Arma_3:_Event_Handlers (Wayback snapshot 2025-03-27)", read: 2026-11-04 }
  - { id: startup, resource: "https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters, entries showScriptErrors, debug, noLogs, profiles, noPause", read: 2026-11-04 }
  - { id: debugmode, resource: "https://community.bistudio.com/wiki/Arma_3:_Debug_Mode (Wayback snapshot 2025-01-15)", read: 2026-11-04 }
  - { id: stacktrace, resource: "https://community.bistudio.com/wiki/diag_stacktrace (Wayback snapshot 2025-03-18)", read: 2026-11-04 }
  - { id: crash, resource: "https://community.bistudio.com/wiki/Crash_Files (Wayback snapshot 2025-04-17)", read: 2026-11-04 }
  - { id: diaglog, resource: "https://community.bistudio.com/wiki/diag_log (Wayback snapshot 2025-03-27)", read: 2026-11-04 }
  - { id: rptcount, resource: "https://steamcommunity.com/workshop/filedetails/discussion/1200127537/1483235412216270672 (user comment, via search result)", read: 2026-11-04 }
  - { id: screenshot, resource: "https://community.bistudio.com/wiki/screenshot", read: 2026-11-04 }
  - { id: fps, resource: "https://community.bistudio.com/wiki/diag_fps (Wayback snapshot 2025-01-16)", read: 2026-11-04 }
  - { id: codeperf, resource: "https://community.bistudio.com/wiki/diag_codePerformance (Wayback snapshot 2024-12-06)", read: 2026-11-04 }
  - { id: capframe, resource: "https://community.bistudio.com/wiki/diag_captureFrame (Wayback snapshot 2025-01-12)", read: 2026-11-04 }
  - { id: activesqf, resource: "https://community.bistudio.com/wiki/diag_activeSQFScripts (Wayback snapshot 2025-03-16)", read: 2026-11-04 }
  - { id: spotrep120, resource: "https://dev.arma3.com/post/spotrep-00120 (Game Update 2.22, 2026-08-11)", read: 2026-11-04 }
  - { id: spotrep122, resource: "https://dev.arma3.com/latest/spotrep (SPOTREP #00122, 2.22 hotfix, 2026-09-30)", read: 2026-11-04 }
  - { id: cba-wait, resource: "CBATeam/CBA_A3@8b15a04:addons/common/fnc_waitUntilAndExecute.sqf", read: 2026-11-04 }
  - { id: cba-pfh, resource: "CBATeam/CBA_A3@8b15a04:addons/common/init_perFrameHandler.sqf", read: 2026-11-04 }
  - { id: cba-events, resource: "CBATeam/CBA_A3@8b15a04:addons/events/fnc_addEventHandler.sqf, fnc_localEvent.sqf, fnc_serverEvent.sqf, fnc_globalEvent.sqf, fnc_targetEvent.sqf", read: 2026-11-04 }
  - { id: mcp-ext, resource: "Zihuange/arma3-mcp@380af42:game/extension/src/arma3_mcp_extension.cpp (MIT)", read: 2026-11-04 }
  - { id: mcp-vision, resource: "Zihuange/arma3-mcp@380af42:docs/vision.md, test record 2026-09-17", read: 2026-11-04 }
  - { id: t1, resource: "research/10-prior-harness.md", read: 2026-11-04 }
---

# Observing the game: waits, events, errors, logs, screenshots

This topic covers what a check can see: waits with a ceiling (`CR-20`), events per machine (`CR-21`), script errors (`CR-23`), logs (`CR-24`), screenshots (`CR-25`) and performance (`CR-26`). Engine features are checked against Arma 3 2.22, the current stable game update[^spotrep120][^spotrep122]; one that needs the profiling branch says so. Most BI wiki pages were read through Wayback Machine snapshots dated in the source list, because the live wiki refuses automated reads; a snapshot older than 2.22 can miss later changes.

## 1. Waiting with a ceiling

- `waitUntil` suspends a scheduled script and re-evaluates its condition "mostly every frame, depends on complexity of the condition and the overall engine load"[^waituntil]. It needs a scheduled environment unless the condition is true at once[^waituntil].
- `CBA_fnc_waitUntilAndExecute` runs a condition every frame, unscheduled, and takes a timeout with its own timeout code[^cba-wait]. The timeout is measured against `CBA_missionTime`[^cba-wait].
- `CBA_missionTime` grows by `diag_tickTime` deltas only on frames where `time` changed, so it stops while the game is paused[^cba-pfh]. `time` stops when the game is paused[^time]. `diag_tickTime` is real time since game start and keeps counting while paused[^ticktime].

What it forces:

- **A wait has two ceilings.** The in-game wait polls every frame and gives up on a real-time ceiling (`diag_tickTime`), not a game-time one, so a paused game still ends the wait. The runner outside the game holds a wall-clock ceiling of its own, because a frozen frame runs no SQF at all. A wait that ends on the runner's ceiling without an answer from the game is `invalid`, not `fail`. Touches `CR-20`, `CR-30`, `CR-31`.
- **The wait returns the last observed value** when it ends at its ceiling, so the failure says what was seen (PRD scenario "things go wrong" 1). Touches `CR-20`, `CR-31`.
- **The wait runs on the machine the condition is about**, through the channel of T3. Touches `CR-20`, `CR-10`.

## 2. Clocks for timestamps

| Clock | What it is |
|---|---|
| `time` | Seconds since mission start, different on each client, stops when paused; JIP clients start at 0 and jump after about 2.5 s; on overloaded servers it lags real time[^time]. |
| `serverTime` | Time since server start, synced to clients at mission start, on connect and every 5 minutes; in singleplayer it equals `diag_tickTime` since 2.18[^servertime]. |
| `CBA_missionTime` | CBA's mission time, sent by the server to each connecting player, advancing only while `time` does[^cba-pfh]. |
| `diag_tickTime` | Real seconds since this process started; precision falls the longer the game runs[^ticktime]. |
| `systemTimeUTC` | Wall clock as `[year, month, day, hour, minute, second, millisecond]`, since 2.00[^utc]. |
| `diag_frameNo` | Frames drawn since the game started, not reset between missions[^frameno]. |

What it forces:

- **Every event and step record carries real time and game time**: `systemTimeUTC` for ordering across machines (all processes of a local run share one machine clock), `diag_tickTime` for durations within a machine, and `time` or `CBA_missionTime` for game time. Touches `CR-21`, `CR-42`.
- **A frame number on each record** shows whether two events fell in the same frame. Touches `CR-21`.

## 3. Engine events and CBA events

- Mission event handlers in 2.22 include `EntityCreated` (2.10), `EntityDeleted` (2.18), `EntityKilled`, `EntityRespawned`, `GroupCreated`, `GroupDeleted`, `EachFrame`, `Ended`, `MPEnded`, `ExtensionCallback`, `PlayerConnected`, `PlayerDisconnected`, `HandleDisconnect`, the `OnUser...` family, `ScriptError` (2.14), `ScriptSpawned` (2.22), `ActionTargetChanged` (2.22) and `DisplayOpened` (2.22)[^meh][^spotrep120]. `HandleDisconnect`, `PlayerConnected` and the `OnUser...` handlers fire only on the server[^meh]. `ParachuteLanded` is listed for 2.24, past the stable branch[^meh].
- Object event handlers such as `Killed` take a local argument, so they fire where the object is local[^oeh]. The multiplayer handlers `MPHit`, `MPKilled` and `MPRespawn` can be added on any machine and run on every connected machine; they are not saved with a save game[^oeh].
- CBA events are named strings. `CBA_fnc_addEventHandler` registers code for a name and returns an id; `CBA_fnc_localEvent` raises on this machine, `CBA_fnc_serverEvent` on the server, `CBA_fnc_globalEvent` everywhere including here, and `CBA_fnc_targetEvent` where given objects are local[^cba-events].

What it forces:

- **An event recorder on every machine** adds the handlers a check names (engine mission and object handlers, and the mod's CBA event names) and stores each firing with the clocks of section 2 and the machine's name. Server-only handlers are added on the server. Touches `CR-21`.
- **CBA events are subscribed by name.** The check or the mod's documentation names the events; CBA has no public call that lists every event, so "record all of a mod's events" is **unverified**. Touches `CR-21`.
- **The recorder installs before the check acts** and is cleared by the reset between checks, so one check's handlers do not fire in the next. Touches `CR-21`, `CR-02`.

## 4. Script errors, caught live

- The `ScriptError` mission event handler (2.14, so on the stable branch) is "triggered when a script error occurs" and passes the error text, source file (empty for spawned code), line number, byte position, the whole script as a string, and a stack trace in `diag_stacktrace` format[^meh]. Its recursion limit is 2: an error inside the handler triggers it once more, no more[^meh].
- `diag_stacktrace` (2.06) returns, per frame, the function name, line number, scope name and a hash map of all local variables[^stacktrace].
- `-showScriptErrors` shows errors on screen; in the Eden Editor they are always shown[^startup]. `-debug` (2.02) makes the engine log more context, such as a full stack trace with each script error, and unlocks errors that are hidden otherwise[^debugmode][^startup]. `-noLogs` stops errors from being written to the RPT[^startup].
- The `arma-grappling-hook` harness finds script errors by matching RPT lines against a fixed list of `Error ...` patterns after the run, and fails a mission on any of them[^t1].

What it forces:

- **A `ScriptError` handler on every machine** (server, each client, each headless client), installed as early as the framework's addon can run, sends each error with its machine, clocks and stack trace back through the result channel. Touches `CR-23`.
- **Errors are tied to the step that was running** on that machine when they fired: the framework keeps the current check and step id in a variable on every machine and stamps each error with it. An error from code a step spawned that fires later still carries the file and stack trace, so the report shows both. Touches `CR-23`, `CR-42`.
- **The RPT stays a second source**: errors before the handler exists, and engine errors that are not script errors, are read from the log, as T1 does. Runs never use `-noLogs`. Runs use `-debug` for the extra context. Touches `CR-23`, `CR-24`.
- **Needs a spike.** Pass condition: in a session with a dedicated server, two clients and a headless client, a deliberate error (`1/0`) in unscheduled code, in spawned code and in a function called from `remoteExec` raises `ScriptError` on the machine where it ran, with the right file and line, with and without `-showScriptErrors`; and every error the RPT shows from mission start on was also caught by the handler.

## 5. Logs

- Arma 3 writes its RPT, `.bidmp` and `.mdmp` files to `%localappdata%\Arma 3` on Windows and `~/.local/share/Arma 3/` on Linux; with `-profiles` they go to that folder[^crash][^startup]. Server logs are written to the server's profile folder[^startup].
- Each start writes a new file named with the date and time, and only the last 10 are kept, according to a user comment: **unverified**[^rptcount].
- `diag_log` writes one line per call to the RPT[^diaglog]; a 2011 note says a line is cut at 1044 characters, which is **unverified** for 2.22[^diaglog].
- The T1 harness gives each machine its own profile folder, so each start keeps its own RPT, and reads them every 500 ms while the run goes[^t1].

What it forces:

- **One profile folder per machine per run**, inside the run's output, so each machine's RPT is found by path and kept with the run. Touches `CR-24`, `CR-51`.
- **Logs are read while the game runs** and filtered by machine, check, step and level. Whether a running game's RPT can be read without blocking it, and how long a `diag_log` line takes to reach the file, **needs a spike**. Pass condition: a reader opening the RPT of a running client sees a `diag_log` line within 2 s of the call, 100 times in a row, and the game logs no error about the file. Touches `CR-24`, `CR-27`.
- **Results do not travel through the RPT.** T1 lost a client's last lines when the client was ended; values come back through the result channel and the RPT is evidence only[^t1]. Touches `CR-10`, `CR-24`.

## 6. Screenshots and video

- `screenshot` (1.60) saves a PNG to the `Screenshots` folder of the profile directory and returns `true` on success. The folder is limited to 250 MB unless the profile sets `maxScreenShotFolderSizeMB`. Only the 3D scene is captured, without UI or cursor[^screenshot]. A 2018 note says it fails silently when `-profiles` points elsewhere or the HDR setting is below 16[^screenshot].
- A dedicated server has no game window[^mcp-ext], so it cannot take a screenshot.
- `Zihuange/arma3-mcp` captures the game window from inside the process with `PrintWindow` and `PW_RENDERFULLCONTENT`, falling back to `BitBlt`, without needing the game in the foreground[^mcp-ext][^mcp-vision]. Its record shows a valid 1024x575 capture of the Eden Editor on 2026-09-17[^mcp-vision].
- No engine command to record video was found: **unverified**.

What it forces:

- **The screenshot check runs on a client that renders** and returns the files with what it expected to show. It has no verdict. Touches `CR-25`.
- **The `-profiles` failure needs a spike**, because the framework runs every machine with its own profile folder. Pass condition: a client started with `-profiles=<run folder>` and default video settings returns `true` from `screenshot "t.png"` and the file exists under `<run folder>\Screenshots` within 5 s. If it fails, the framework uses an in-process window capture like arma3-mcp's, which also includes the UI. Touches `CR-25`.
- **Screenshots are taken only while the Shadow stream is connected**, since the window renders only then (CLAUDE.md, "Machines"). A capture with no stream is reported as missing, not as an empty image. Touches `CR-25`, `CR-30`.

## 7. Performance probes

| Probe | What it gives |
|---|---|
| `diag_fps` | Average frame rate over the last 16 frames, loading screens included[^fps]. |
| `diag_frameNo` | Frame counter[^frameno]. |
| `diag_codePerformance` | Average milliseconds per run of given code over N runs, unscheduled, stopping after 1 s unless `ignoreTimeLimit` (2.14). In multiplayer it runs one cycle only, unless the in-game debug console is available[^codeperf]. |
| `diag_captureFrame` | Frame capture; only on the diagnostic or profiling branch[^capframe]. |
| `diag_activeSQFScripts` | Each running scheduled script's name, file, running state and current line[^activesqf]; since 2.22 also a hashed handle[^spotrep120]. |

What it forces:

- **Frame time is sampled on every machine** from `diag_fps` and `diag_frameNo` and kept with the run; the same numbers feed the health watch of T8. Touches `CR-26`, `CR-30`.
- **Test missions enable the debug console**, or `diag_codePerformance` measures a single cycle in multiplayer and the cost of a mod function on the server cannot be measured. Touches `CR-26`.
- **Frame captures are profiling-branch only** and stay out of the MVP. Touches `CR-26`.
- **`diag_activeSQFScripts` answers "what is running"** when an agent asks a stalled game, with `diag_stacktrace` for the calling code. Touches `CR-27`.

## Findings that need a spike

| Item | Pass condition |
|---|---|
| `ScriptError` on every kind of machine and code | A deliberate error in unscheduled, spawned and `remoteExec` code raises it on the machine where it ran, with file and line, with and without `-showScriptErrors`; every RPT error after mission start was also caught. |
| Reading a live RPT | A `diag_log` line is visible to a reader within 2 s, 100 times in a row, and the game logs no file error. |
| `screenshot` with `-profiles` | Returns `true` and the file exists under the run folder's `Screenshots` within 5 s. |

## Unverified

- RPT file naming and the count of RPTs kept.
- The 1044-character cut of a `diag_log` line on 2.22.
- A public way to list every CBA event name.
- Any engine command to record video.

[^waituntil]: waitUntil
[^time]: time
[^servertime]: serverTime
[^ticktime]: diag_tickTime
[^utc]: systemTimeUTC
[^frameno]: diag_frameNo
[^meh]: Arma 3: Mission Event Handlers
[^oeh]: Arma 3: Event Handlers
[^startup]: Arma 3: Startup Parameters
[^debugmode]: Arma 3: Debug Mode
[^stacktrace]: diag_stacktrace
[^crash]: Crash Files
[^diaglog]: diag_log
[^rptcount]: Steam Workshop discussion comment on RPT files
[^screenshot]: screenshot
[^fps]: diag_fps
[^codeperf]: diag_codePerformance
[^capframe]: diag_captureFrame
[^activesqf]: diag_activeSQFScripts
[^spotrep120]: SPOTREP #00120, Game Update 2.22
[^spotrep122]: SPOTREP #00122, 2.22 hotfix
[^cba-wait]: CBA_fnc_waitUntilAndExecute source
[^cba-pfh]: CBA per-frame handler and CBA_missionTime source
[^cba-events]: CBA event functions source
[^mcp-ext]: arma3-mcp extension source
[^mcp-vision]: arma3-mcp vision test record
[^t1]: T1, the harness of arma-grappling-hook
