---
type: Specification
title: Functional requirements
description: What Shakedown must do, as numbered requirements traced to the PRD - the must requirements in full, and one entry per should and could that keeps its door open.
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# Functional requirements

Each requirement has an ID of the form `AREA-nn` and cites the PRD requirements (`CR-nn`) it serves. Requirements marked **must** are the first release (the MVP). Those marked **should** or **could** are held open by one entry and are specified when a milestone takes them up. The traceability table at the end maps every `CR-nn` to its requirements.

The exact names, arguments and outputs of the interfaces are in the contract: [contract/check-api.md](contract/check-api.md), [contract/cli.md](contract/cli.md), [contract/mcp-tools.md](contract/mcp-tools.md) and [contract/result-file.md](contract/result-file.md). Where a requirement and the contract disagree, the requirement wins and the contract is fixed.

## Project configuration

- **CFG-01** (must) A project is configured by one `shakedown.toml` file at its root, with the keys the CLI contract lists: the game folder, the checks folder, the server's base port, each mod (its folder, and for a watched mod its `source` folder and its game `prefix`), and the setups (mode, clients, headless clients, mods, world, spawn point, start ceiling). Every command reads it fresh. An unknown key, a missing required key or a wrong type is refused with the key's name and line (`CR-01`, `CR-41`).
- **CFG-02** (must) The framework writes nothing into the project except the `.shakedown/` folder, which it creates with a `.gitignore` that ignores everything in it, so session and run folders stay out of git (`CR-51`, `CR-24`; gap H8).

## Sessions

- **SES-01** (must) `session start --setup <name>` starts a session of the named setup: one singleplayer game, or a dedicated server with the setup's game clients and headless clients, all on the local machine. It returns when the session is `ready` (every machine answers in the session mission) or `failed` (`CR-01`).
- **SES-02** (must) At most one session runs on the machine. A second `session start` is refused, naming the running session's id and setup and the command that stops it (`CR-01`).
- **SES-03** (must) Before any process starts, the start checks that the game's `arma3_x64.exe` (and for a dedicated setup `arma3server_x64.exe`) exist in the configured game folder, that a process `steam.exe` runs in the current user's Windows session, that every mod folder of the setup exists, that the framework's installed addon holds its PBO and its extension, and, for a dedicated setup, that the five ports from the base port are free. A failed check refuses the start and names exactly what is missing (`CR-01`, scenario "things go wrong" 4).
- **SES-04** (must) A start has a ceiling of 180 s from the first process launch to `ready`, settable per setup with `start_timeout`. At the ceiling, or when a process exits during the start, the session is `failed`, every process it started is stopped, and the reason names the machine that was not ready, its state and the last 20 lines of its RPT (`CR-01`, `CR-05`).
- **SES-05** (must) A `ready` session runs any number of runs, checks and `exec` calls without restarting a process. Only a reload at level `mission-restart` or `game-restart`, or a check with reset mode `mission`, restarts anything (`CR-01`).
- **SES-06** (must) A session belongs to the runner, not to the client that started it. When the command line exits or an AI agent's session ends and its MCP server stops, the game session keeps running and a new client finds it (`CR-01`, `CR-53`; gap H6).
- **SES-07** (must) `session stop` stops every process of the session and returns when none is left (`CR-05`).
- **SES-08** (must) The runner runs as its own detached process, outside the process tree and job object of any client, so that a client's host ending the client does not end the runner, whether the host closes a job object with kill-on-close that holds the client, with or without breakaway allowed, or terminates each process of the client's tree. Those are the two ways a host such as Claude Code ends a client, and a test reproduces either one around `shakedown mcp` (`CR-01`, `CR-05`; gap H6, `INT-CC-02`).
- **SES-09** (must) Every game process is started from the installed executables, with its own profile folder in the session folder, `-name=<machine>`, `-filePatching`, `-debug`, and every option of `INT-AR-02`. No game executable is copied (`CR-01`, `CR-03`, `CR-24`).

## Reset between checks

- **RST-01** (must) Before every check whose reset mode is `in-place`, the framework resets the session as process `P-06` says: runs the previous check's cleanups, deletes every entity created since the baseline (whatever created it), restores every player's unit (alive, position, direction, velocity, damage, loadout, animation state, out of any vehicle) and the date and weather. The next check sees the baseline state on every machine (`CR-02`; gap H2).
- **RST-02** (must) The baseline is captured at every mission start, so when the session first becomes `ready` and again after every mission restart and game restart: each machine's agent captures it once its player stands on the ground, before it reports `ready` (`P-02` step 7). It holds each player unit's position (ASL), direction, loadout and animation state, the date, and the overcast, fog and rain (`CR-02`).
- **RST-03** (must) `reset` runs the reset on demand, outside a run. On a `broken` session it is refused with the reason the session broke and the two ways out, a `game-restart` reload and `session stop` (`CR-02`, `CR-27`).
- **RST-04** (must) A reset has a ceiling of 15 s. A reset that does not verify within it fails, names what did not match, and leaves the session `broken` (`CR-02`, `CR-31`).
- **RST-05** (must) A player who died is alive again before the reset ends: on a dedicated session the session mission's respawn brings the unit back at once; in singleplayer, where the engine ends the mission when its player dies even with `respawn = 3`, the agent's `Killed` handler hands the player a new unit of the same class, in the same group, where the old one fell, with `selectPlayer`, before the engine ends the mission, and the reset deletes the body (`SPK-05`). A singleplayer session survives the player's death in every case (`CR-02`).
- **RST-06** (must) A check whose header says `@reset mission` gets a mission restart (level B of `P-08`) instead of a reset in place, for a check that changes state the framework cannot name, such as global variables or mod settings (`CR-02`; Q5).
- **RST-07** (must) The state a check left stays in the game until the next reset, so it can be inspected after the check ends (`CR-27`).

## Reload

- **RLD-01** (must) `reload [files...]` brings changed mod code into the running session at the lowest level the change allows, as process `P-08` says. With no files given it finds the changed files itself by comparing the watched `source` folders with the snapshot taken at the last ready state. A `reload` or a `reset` asked while a run, a reload or a reset is in progress is refused, naming what is in progress and, for a run, its progress and `run stop` (`CR-03`, `CR-05`).
- **RLD-02** (must) Each changed file is classified, and the highest level any file needs is applied (`CR-03`; conflict C1):

  | Change | Level | Reason given |
  |---|---|---|
  | A check file | none | Checks are read fresh for every execution (`RLD-04`). |
  | A `.sqf` file that defines a `CfgFunctions` function which is not final on any machine | `mission` | Recompiled in place. |
  | A `.sqf` file that defines a final function, or any other `.sqf` file (such as a CBA `PREP` function) | `mission-restart` | Final code changes only at mission start, with CBA's cache off in the session mission. |
  | A new or deleted file | `game-restart` | File patching finds only files that exist in the built PBO; the mod must be rebuilt before this reload. |
  | Any other file (`config.cpp`, `.hpp`, `.h`, `.inc`, `.ext`, string tables, models, textures, sounds) | `game-restart` | Config is read at game start; the others are not reloaded by file patching. |

- **RLD-03** (must) A reload reports the level applied, each file with its level and reason, the machines it reached, and the time from the reload order to every machine being ready on the new code (`CR-03`, `CR-42`).
- **RLD-04** (must) Checks are never reloaded by a command: the runner reads each check's source from disk at the start of every execution (`CR-03`).
- **RLD-05** (must) A check executed after a `mission`-level reload of mod code records `reload: partial` in its result, because handlers the mod registered by value keep the old code until a mission restart (`CR-03`, `CR-31`).
- **RLD-06** (must) The framework does not build mods. A watched mod is loaded from a development build that file patching can read (such as HEMTT's `hemtt dev` output linked into the game folder), and a reload of a new or deleted file states that a rebuild comes first (`CR-03`; PRD scope).

## Stopping and cleaning up

- **STP-01** (must) Every game process is started suspended, assigned to its session's job object, which the runner holds, with kill-on-close, and only then resumed, so it ends when the runner ends, however the runner ends (`CR-05`).
- **STP-02** (must) A run that passed, failed, was stopped by hand, or whose session broke leaves no process of its own running: a run that started its own session stops it; a run against an existing session leaves exactly that session's processes (`CR-05`, scenario "as planned" 5).
- **STP-03** (must) The runner keeps a run record at `%LOCALAPPDATA%\Shakedown\record.json` of every process (id and start time), every file and folder it created outside the project, and every session it started. It updates the record before each step that creates something, and clears what the record lists at its next start (`P-12`) (`CR-05`, scenario "things go wrong" 6).
- **STP-04** (must) After `session stop`, no `arma3_x64` or `arma3server_x64` process the session started runs, no port of the session is held, and no mission copy is left in the game's `MPMissions` folder (`CR-05`).

## Control: running code on a machine

- **CTL-01** (must) Code runs on a machine named by `DM-10`, or on each machine of a group named by `DM-12`, and its return value comes back as data, encoded by `DM-13`. For a group, the result is a map from machine name to value (`CR-10`, `CR-22`).
- **CTL-02** (must) Arguments travel as data and arrive as `_this`; they are never formatted into the code's text. The code does not see the caller's local variables (`CR-10`).
- **CTL-03** (must) Code always runs scheduled on the target machine (`spawn`), so it may wait, and every call has an in-game ceiling (default 10 s) and a runner ceiling (`OBS-03`) (`CR-10`, `CR-20`).
- **CTL-04** (must) A call to a machine the session does not have, or code that does not compile on the target, returns an error that names the machine or the engine's compile message, and inside a check ends it `fail` / `check-error` (`CR-10`, `CR-31`).
- **CTL-05** (must) A script error raised by code sent to a machine does not stop that code: it runs on and its value comes back with the error. Inside a check the error fails the check `fail` / `script-error` once the body ends (`ERR-03`), unless an `expectError` block matches it; through `exec` the answer lists the error beside the value, and the call succeeds (`CR-10`, `CR-23`).

## Scene

- **SCN-01** (must) The check API creates an object, a vehicle or a unit by class name at a position (ASL) and direction on the server, with vehicle randomisation off, and returns it (`CR-11`).
- **SCN-02** (must) The check API places a unit at a position (ASL) facing a direction: it sets the direction and then the position on the machine where the unit is local, and returns once the server sees the unit within 0.1 m and 3 degrees, polled to the wait ceiling (`CR-11`).
- **SCN-03** (must) The check API sets the date and the time of day, and the overcast, fog and rain, on the server, forces the weather change, broadcasts the date, and returns once every machine reports the same date and overcast (`CR-11`, `CR-22`).
- **SCN-04** (must) Anything a check creates by its own SQF on any machine is deleted by the next reset as well as what the API created, because each machine records every entity created (`CR-11`, `CR-02`).

## Acting through the mod's real path

- **ACT-01** (must) The check API makes the player of a named machine use an `addAction` action on a target object, found by its title (exact match after stripping structured-text tags). On the acting client it waits, to the wait ceiling, until the action exists, its condition holds with the documented variables, and the player is within its radius; then it fires it with `action ["User", target, id]`. The engine runs an action fired that way whatever its condition and radius (`SPK-10`), so this check of the framework's is the only one. When the ceiling passes it fails with the reason: no action of that title (with the titles that exist), the condition false, or the distance and the radius (`CR-12`).
- **ACT-02** (must) The check API presses a CBA keybind of a named mod and action on a named machine: it reads the bound key from `CBA_fnc_getKeybind` and calls the keybind's down code, its up code, or both in order, with `[display 46, DIK, shift, ctrl, alt]` (`CR-12`).
- **ACT-03** (must) A keybind step on a machine without CBA, or for a keybind that is not registered, ends the check `fail` / `check-error` with the keybinds registered for that mod, or, when it has none, the mods that have keybinds, both in lower case as CBA keeps them (`CR-12`, `CR-31`).
- **ACT-04** (should) Perform an ACE interaction menu action by its path, as ACE's own key-up code does, pinned to the ACE version a run reports (`CR-12`; gap H7, Q6).
- **ACT-05** (should) Send real key input to one game client window without the foreground or the desktop's input, if spike `SPK-11` passes; otherwise dropped, since handlers cover every key path the MVP needs (`CR-15`; conflict C3).

## Observation

- **OBS-01** (must) Every wait polls its condition on the named machine, every frame, until it holds or its in-game ceiling passes on real time (`diag_tickTime`). The default ceiling is 10 s; every wait takes its own. A ceiling is a number of seconds above 0; any other value is a wrong argument (`DM-04`). A condition that raises a script error or returns anything but a Boolean ends the wait at that evaluation, with no retry: inside a check, `fail` / `script-error` when the machine caught a script error during it, otherwise `fail` / `check-error` naming the type it returned; through `wait`, an answer with `met` `false` and that cause (`CR-20`; gaps H4, H5, Q3).
- **OBS-02** (must) The check API has no fixed sleep. Every wait has a condition and a ceiling (`CR-20`).
- **OBS-03** (must) The runner holds its own wall-clock ceiling on every order and every check: the in-game ceiling plus a grace of 10 s. Reaching it with no answer makes the machine `lost` (`CR-20`, `CR-31`; gap H5).
- **OBS-04** (must) A wait or an assertion that ends at its ceiling reports the description, the ceiling, and the last observed value on each machine it polled (`CR-20`, scenario "things go wrong" 1).
- **OBS-05** (must) Every step record, script error and event carries the machine's name, the wall clock (`systemTimeUTC`), the machine's real time (`diag_tickTime`), its mission time (`time`) and its frame number (`diag_frameNo`) (`CR-23`, `CR-42`).
- **OBS-06** (must) The check API reads one value on several machines and asserts that each meets an expectation, or that every machine agrees with the first one named (within a tolerance for numbers and positions), polled to the ceiling. On failure it reports the value of each machine (`CR-22`).
- **OBS-07** (must) The check API has a wait that returns `true` or `false` at its ceiling instead of failing the check (`CR-20`).
- **EVT-01** (should) Record a named mod (CBA) or engine event on each machine with the clocks of `OBS-05`, and wait for one with a ceiling (`CR-21`).

## Script errors

- **ERR-01** (must) Every machine's in-game agent catches each script error with the `ScriptError` mission event and sends it at once with its text, file, line, stack trace, the clocks of `OBS-05`, and the run id, check id and step path that were set on that machine when it fired (`CR-23`).
- **ERR-02** (must) The runner also reads each machine's RPT and records every script error line the handler did not report (errors before the handler existed, at mission load), matched by the pattern of `INT-AR-07` in [04-integrations.md](04-integrations.md), and attributed by time (`CR-23`).
- **ERR-03** (must) A script error on any machine during a check's execution fails it (`DM-03`) unless it happened inside an `expectError` block whose pattern it matches (`CR-23`, `CR-31`).
- **ERR-04** (must) `expectError` declares that an error containing a text (case-insensitive) is expected in a block of code. Matching errors on any machine until the check ends are marked expected; none matching by the end fails the check (`CR-23`).
- **ERR-05** (must) Script errors outside any check are kept with the session and returned by `errors` with their machine and time (`CR-23`, `CR-27`).

## Logs

- **LOG-01** (must) Each machine start writes its RPT and any crash dump to its own profile folder in the session folder, kept after the session stops (`CR-24`).
- **LOG-02** (must) Every log line the framework returns or stores outside the profile folders, every value and message in the result file and in what the command line and the tools return, and every profile folder's RPT once its session stops, has Steam IDs (17-digit numbers starting `7656119`) replaced by `<steamid>`, and the logged-in Steam account's account name and persona name (`INT-STM-04`), each matched as a whole word and case-sensitive, replaced by `<steamname>` (`CR-24`, `CR-51`; gap H8).
- **LOG-03** (must) `logs` returns a machine's log lines filtered by run, by check, by a regular expression and by count (the last N lines), redacted (`CR-24`).
- **LOG-04** (must) A run's folder holds each machine's log lines for the run's time span and for each check execution, redacted, one file per machine, and the in-game agent writes a marker line at each check's start and end so a check's lines can be cut out (`CR-24`, `CR-51`).

## Inspecting a running game

- **INS-01** (must) `inspect <machine>` returns, without stopping anything: the machine's state and start number, the mission and its start count, `time`, `diag_tickTime`, `diag_fps`, `diag_frameNo`, the run, check and step running; the player unit (class, position ASL, direction, damage, alive, vehicle, animation); up to 50 objects within 50 m of the player or of a named object (class, net id, position, distance); up to 50 running scheduled scripts from `diag_activeSQFScripts`; and the last 10 script errors. The radius and the object are arguments. The order has an in-game ceiling of 10 s; a machine that does not answer within it plus the grace of `OBS-03` is `lost` as process `P-07` says, and `inspect` returns its machine with state `lost` and nothing else, instead of an error (`CR-27`).
- **INS-02** (must) `status` answers without a game: whether the runner runs and answers (`P-01`), the session's id, setup, state and each machine of `INS-04`, and the active run with its progress (`CR-27`, `CR-53`).
- **INS-03** (must) `exec`, `inspect`, `logs` and `errors` work while a run is active and while the session is `broken` (`CR-27`).
- **INS-04** (must) `status`, and every session shape the command line and the tools return, list each machine from the moment its process is launched, in every session state including `starting` and `reloading`, with: its state and start number; its operating-system process id and process start time; its profile folder; its last answer time; its health flag of `HLT-02` (`null`, `mission-time` or `frame`) and the time the flag was raised; and the clocks of `OBS-05` from its last heartbeat. `status` also gives the runner's process id. A person or an AI agent finds a machine's window, process and RPT from these, and an outside tool can suspend, end or dump a chosen machine's process by its id, which is how a stall, a crash and a crash dump are made on purpose (`CR-27`, `CR-53`, `CR-30`).

## Outcomes and health

- **OUT-01** (must) Every check execution ends `pass`, `fail` or `invalid` by the rules `DM-01` to `DM-08`, with a reason kind and a message for every outcome that is not `pass` (`CR-31`).
- **HLT-01** (should) Watch each machine's health with a heartbeat (mission time against real time, frame rate, answer time) and process and window checks, and end a check that overlaps an unhealthy interval `invalid` / `unhealthy` (`CR-30`; conflict C6). Its clock watch is `HLT-02`, a must.
- **HLT-02** (must) From the moment a machine is `ready`, its in-game agent sends a heartbeat every second of real time carrying the clocks of `OBS-05`. The heartbeat is sent from a per-frame handler (the `EachFrame` mission event, gated on `diag_tickTime`), not from a scheduled loop, so it keeps coming while the scheduled environment or the simulation stands still. The runner flags the machine unhealthy while, over the last 5 s of real time, its mission time advanced less than half as much as its real time or its frame number did not advance (thresholds until `SPK-15`). A check execution that overlaps a flagged interval ends `invalid` / `unhealthy`, naming the machine and which clock stopped, unless a cause for `fail` came first (`DM-06`). `status` shows the flag and the last heartbeat's clocks on the machine (`INS-04`) (`CR-30`, G3; conflict C6).
- **OUT-02** (should) Repeat each selected check N times and summarise each by `DM-09`, with the count of each outcome (`CR-32`).
- **OUT-03** (could) Run a check marked as expected to fail against a build where its promise is broken, and report it as proven when it fails (`CR-33`).
- **OUT-04** (could) Measure a baseline of the game's own behaviour in the same run and conditions as a check that compares against it (`CR-34`).

## Selecting and running checks

- **RUN-01** (must) A run selects checks by one or more check ids, by one or more tags (a check matches when it carries any of them), or both (the union). With no selection it runs every check (`CR-40`).
- **RUN-02** (must) `list` returns every check's id, name, tags, setups, timeout and reset mode from the files, without a runner or a game, filtered like `RUN-01` (`CR-40`, `CR-53`).
- **RUN-03** (must) Checks run in the order given; checks selected by tag or by default run in check id order (`CR-40`).
- **RUN-04** (must) A run given a setup when no session runs starts a session of that setup, runs, and stops it, whatever the outcomes (`CR-05`, `CR-40`, scenario "as planned" 5).
- **RUN-05** (must) A run can be stopped by hand, with `run stop`, `run_stop`, or a Ctrl+C or Ctrl+Break console event to `shakedown run`, which it treats alike. Its executing check ends `invalid` / `run-stopped`, the checks not started are listed as `not-run`, and the result file is written (`CR-05`, `CR-31`, `CR-51`).
- **RUN-06** (must) A run has a runner ceiling of the sum of its selected checks' timeouts plus 60 s, counted from the start of its first check, after any session the run starts is `ready`. Resets, mission restarts and cleanups count against it. At it the run is stopped as `RUN-05` says, and the result file gives `stopped_reason` `run-ceiling` (`CR-20`, `CR-05`).
- **RUN-07** (should) Choose the setups per check: a check's `@setups` header names the setups it runs in, a run skips the others as `not-run` / `setup`, and one command runs a selection in each setup it needs, one session per setup (`CR-41`). The MVP honours the header for skipping; running several setups in one command waits for this requirement.
- **TIM-01** (should) Report the time of every step and flag steps over a threshold (`CR-42`). The MVP records step times (`OBS-05`) without flagging.

## Check API

- **API-01** (must) A check is one `.sqf` file under the checks folder whose first lines are a header of `// @key value` lines: `@check` (the name, required, starting with the story or requirement ID it proves), `@tags`, `@setups`, `@timeout` (seconds, default 120) and `@reset` (`in-place` or `mission`, default `in-place`), as the check API contract gives. The rest of the file is the body (`CR-50`, `CR-40`).
- **API-02** (must) The body runs scheduled on the lead machine, as SQF compiled with `compile`, so a check needs no build step and is reloaded on every run (`CR-50`, `CR-03`).
- **API-03** (must) The check API offers, as SQF functions with the prefix `skd_fnc_`: steps; running code on one machine and on a group; waits; immediate and polled assertions on one machine and on several; agreement across machines; expected errors; creating objects and units, placing units, setting time and weather; using an action; pressing a keybind; cleanups; evidence; ending a check as `fail` or `invalid` by hand; and the session's machines and setup. Names, arguments and results are in the check API contract (`CR-50`, `CR-10`, `CR-11`, `CR-12`, `CR-20`, `CR-22`, `CR-23`).
- **API-04** (must) The check API is plain SQF functions: a check needs no `#include`, no macros and no language but SQF (`CR-50`).
- **API-05** (must) The framework's addon does not need CBA. It uses CBA only when a check presses a keybind and CBA is loaded (`CR-50`, `CR-12`).

## Result file

- **RES-01** (must) Every run writes exactly one result file, `result.json`, in its run folder, valid against the JSON Schema `contract/result.schema.json`, also when the run was stopped or its session broke (`CR-51`).
- **RES-02** (must) The result file holds the run and session ids, the setup with its machines, the game build and each mod's version as each machine loaded it, the selection, every check execution with its outcome, reason, steps with times, script errors, evidence and reload state, the reloads during the run, the checks not run with the reason, and the summary counts (`CR-51`, `CR-42`, `CR-23`).
- **RES-03** (must) Evidence for an outcome that is not `pass` includes the failed step's path, the last observed value per machine, every unexpected script error, the values the check attached, and the paths to each machine's log excerpt for that check (`CR-51`, `CR-23`).
- **RES-04** (must) Paths in the result file are relative to the run folder (`CR-51`).
- **RES-05** (must) Every reason of an outcome that is not `pass` carries the wall-clock time of the cause that decided it, and every machine that was `lost` carries the wall-clock time it was lost, so the order `DM-06` judges by is in the result (`CR-51`, `CR-31`).

## Agent tools

- **AGT-01** (must) `shakedown mcp` is an MCP server over stdio, declared for Claude Code in the project's `.mcp.json`, that gives an AI agent every capability of the command line as tools with the same results (`CR-53`).
- **AGT-02** (must) Every tool has an `outputSchema` and returns `structuredContent` and the same JSON as text. A check that fails is a successful call that returns the outcome. `isError` is used only for what the AI agent can fix (unknown check, unknown machine, code that does not compile, no session, an expired handle), and the message says what to do next (`CR-53`).
- **AGT-03** (must) No tool call outlasts 60 s. A run returns a run handle at once and is followed with `run_status` and `run_result`; a wait or an `exec` takes a ceiling of at most 50 s. A call that waits sends progress notifications at least every 5 s when the client gave a progress token (`CR-53`, `CR-20`).
- **AGT-04** (must) A cancelled tool call stops the game-side work it started (`CR-53`).
- **AGT-05** (must) The MCP server starts and answers `status` on a machine without the game or without a runner, saying so, instead of failing to start (`CR-53`).
- **AGT-06** (must) A `guide` tool returns a short text on how to use the tools and write a check, for clients that do not load the project's docs (`CR-53`).
- **AGT-07** (must) Tool results stay under 20,000 characters; anything longer is written to a file in the session or run folder and the result gives its path (`CR-53`, `CR-24`).

## Safety

- **SEC-01** (must) The runner listens on `127.0.0.1` only. Every client presents the runner token from `runner.json` on each connection, and every extension presents its session token; a connection with anything else, or whose first message is not the protocol's hello, is closed (`CR-60`).
- **SEC-02** (must) The session token is a random 256-bit value made per session and passed to each game process only in its environment, never on a command line. The runner token file is readable by the current user only (`CR-60`).
- **SEC-03** (must) The runner accepts an extension's hello only from a process id it started for that machine of that session. It takes the process id of the connection's other end from the system's TCP table (`GetExtendedTcpTable`), never from the hello, so a process that holds a stolen token still cannot pose as a game process (`CR-60`).
- **SEC-04** (must) Every `exec` order, from a client or from a check, is appended to `audit.jsonl` in the session folder with its time, source (command line, MCP or check id), target and code (`CR-60`, `CR-53`).
- **SEC-05** (should) Refuse to start with BattlEye on or against a server that is not the session's own loopback server (`CR-61`). The MVP writes `BattlEye = 0` and `loopback = 1` into every `server.cfg` it generates.
- **SEC-06** (must) The runner token is `skd_r_` and the session token `skd_s_`, each followed by 64 lowercase hexadecimal digits of the random value. The fixed form lets any leak be found by searching for the pattern, in command lines, files, logs and tool output (`CR-60`).

## Fixture mods

The framework is developed and accepted against fixture mods of its own, whose promises are [contract/fixture-mod.md](contract/fixture-mod.md). Besides a target for each capability, they give a test the failures of the environment it cannot otherwise cause on purpose. They are test targets, never part of the framework: a session loads them only when its setup names them, like any mod.

- **FIX-01** (must) A fixture function crashes the game process it runs on, after a delay in real time, through a fault in native code, so the engine's own crash handling runs and the process ends unasked; another ends the process with a chosen exit code after a delay. A check places a crash at a known point between its own steps (`CR-31`, G3; `SPK-16`).
- **FIX-02** (must) A fixture function holds singleplayer `p1`'s mission time still for a given number of seconds of real time while its frames go on and it keeps answering, as when the Shadow stream is cut, then lets it run (`CR-30`, G3; `SPK-17`).
- **FIX-03** (must) The fixtures name the files a test may change in their installed copy for a reload: the two reload targets, one file absent from the build that a test may create and must delete after, and any installed fixture file a test may name to `reload` without changing it (`CR-03`).

## Further should and could requirements

- **MCH-01** (should) Make a machine leave, join late, rejoin, or restart the server during a session, and wait on the server's own events with a ceiling set from the disconnect timeout (`CR-04`).
- **ENV-01** (should) Check the environment before a run in full: Steam logged in, each mod present in the version the project names, free disk, and report each gap (`CR-06`).
- **RSM-01** (could) Resume a run from the last finished check after a reboot or a game crash (`CR-07`).
- **CHT-01** (could) Act as a cheating client: send forged, repeated or out-of-role requests and events from one machine (`CR-13`).
- **FLT-01** (could) Simulate failures: kill a client, add lag or packet loss, stall a frame, make a unit unconscious, kill and respawn it (`CR-14`; conflict C4, scenario "things go wrong" 7).
- **SHT-01** (could) A screenshot check that captures chosen steps on a rendering client and returns the images with what they were expected to show, with no verdict of its own (`CR-25`; conflict C5).
- **PRF-01** (could) Measure frame time on every machine and the cost of a mod function on the server and a client (`CR-26`).
- **MAN-01** (should) Put a manual tester at the start of a situation set up by a check, with a list of what to look for, and leave the session up for them (`CR-52`).

## Traceability

| CR | Priority | Requirements |
|---|---|---|
| CR-01 | must | CFG-01, SES-01 to SES-06, SES-08, SES-09 |
| CR-02 | must | RST-01 to RST-06, SCN-04 |
| CR-03 | must | RLD-01 to RLD-06, API-02, SES-09, FIX-03 |
| CR-04 | should | MCH-01 |
| CR-05 | must | SES-04, SES-07, SES-08, STP-01 to STP-04, RUN-04 to RUN-06 |
| CR-06 | should | ENV-01 (SES-03 covers what a start needs) |
| CR-07 | could | RSM-01 |
| CR-10 | must | CTL-01 to CTL-05, API-03 |
| CR-11 | must | SCN-01 to SCN-04 |
| CR-12 | must | ACT-01 to ACT-03, API-05; ACT-04 should |
| CR-13 | could | CHT-01 |
| CR-14 | could | FLT-01 |
| CR-15 | should | ACT-05 |
| CR-20 | must | OBS-01 to OBS-04, OBS-07, CTL-03, RUN-06, AGT-03 |
| CR-21 | should | EVT-01 |
| CR-22 | must | OBS-06, CTL-01, SCN-03 |
| CR-23 | must | ERR-01 to ERR-05, CTL-05, OBS-05, RES-03 |
| CR-24 | must | LOG-01 to LOG-04, SES-09 |
| CR-25 | could | SHT-01 |
| CR-26 | could | PRF-01 |
| CR-27 | must | INS-01 to INS-04, RST-03, RST-07 |
| CR-30 | should | HLT-01; its clock watch HLT-02, INS-04 and FIX-02 are musts |
| CR-31 | must | OUT-01, CTL-04, ACT-03, ERR-03, RST-04, OBS-03, RUN-05, RES-05, FIX-01 |
| CR-32 | should | OUT-02 |
| CR-33 | could | OUT-03 |
| CR-34 | could | OUT-04 |
| CR-40 | must | RUN-01 to RUN-04, API-01 |
| CR-41 | should | RUN-07 (CFG-01 declares setups) |
| CR-42 | should | TIM-01 (OBS-05, RLD-03 record times) |
| CR-50 | must | API-01 to API-05 |
| CR-51 | must | RES-01 to RES-05, CFG-02, LOG-04 |
| CR-52 | should | MAN-01 |
| CR-53 | must | AGT-01 to AGT-07, SES-06, INS-02, INS-04, SEC-04 |
| CR-60 | must | SEC-01 to SEC-04, SEC-06 |
| CR-61 | should | SEC-05 |
