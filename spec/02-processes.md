---
type: Specification
title: Processes
description: Every flow of Shakedown end to end, naming the process or machine that does each step - starting a session, an order round trip, running a check, a reset, a reload at each level, a stall, a crash, stopping, and recovery after the runner dies.
status: draft
---

# Processes

Each process has an ID of the form `P-nn`. A step names who does it: **client** (the command line or the MCP server), **runner**, **extension** (in a named machine), **agent** (the in-game agent on a named machine), **lead** (the agent on the lead machine), or **game** (the engine itself). Ceilings and values named here are defined in [03-functional-requirements.md](03-functional-requirements.md); the start options and configuration keys in [04-integrations.md](04-integrations.md).

## The channel

Every process below uses one channel, so it is described once:

- The runner listens on `127.0.0.1` only, on a port it picks at its own start (`SEC-01`).
- Each game process gets four environment variables from the runner when it is started: `SHAKEDOWN_RUNNER` (`127.0.0.1:<port>`), `SHAKEDOWN_SESSION`, `SHAKEDOWN_MACHINE` and `SHAKEDOWN_TOKEN` (`SEC-02`).
- When the game loads the extension, the extension opens one connection to the runner and sends a hello with the session id, machine name, token and its process id. The runner accepts it only if the token matches the session's and the process id is one the runner started for that machine. Otherwise it closes the connection and logs the refusal.
- **Runner to game.** The runner sends an order (id, kind, payload) to the extension. The extension queues it and calls the engine's callback with name `shakedown`, function `order` and the order id as data. The agent's `ExtensionCallback` handler fetches the order with `"shakedown" callExtension ["fetch", [id, part]]` in parts of at most 10000 bytes until the last part, since one `callExtension` result is limited to 10240 bytes.
- **Game to runner.** The agent sends results, errors and requests with `"shakedown" callExtension ["send", [json]]`, which carries any size in one call. The extension queues the message and returns at once; its own thread writes it to the runner.
- The extension does no work inside `callExtension` beyond queueing, and retries a callback the engine refused (`INT-AR-03`).

## **P-01** Start the runner

1. **client**: reads `%LOCALAPPDATA%\Shakedown\runner.json` (port, token, process id and start time of the runner). If it names a live runner whose process start time matches, the client connects with that token and the process ends here.
2. **client**: otherwise starts `shakedown runner` as a detached process outside the client's own job object and process tree (`SES-08`), and waits for `runner.json` to appear, polling every 100 ms with a ceiling of 10 s.
3. **runner**: at start, before it writes `runner.json`, runs the recovery of `P-12`, picks a free port on `127.0.0.1`, makes a random 256-bit token, writes `runner.json` readable by the current user only, and creates its job object with kill-on-close.
4. **runner**: exits by itself 60 s after its last session stopped with no client connected; it then deletes `runner.json`.

## **P-02** Start a session

Triggered by `session start --setup <name>` (or by `P-04` when no session runs).

1. **client**: reads `shakedown.toml`, checks it against `CFG-01`, and sends the setup to the runner. A malformed file is refused here with the key at fault and its line; an unknown setup with the setups the file declares.
2. **runner**: refuses if a session already runs (`SES-02`), naming it and the command that stops it.
3. **runner**: checks the preconditions of `SES-03`: the game executables exist, a `steam` process runs, every mod folder of the setup exists, the setup's ports are free, the framework's addon link exists in the game folder (creating it if missing, `INT-AR-06`). Any failure ends the session as `failed` before a process starts, naming what is missing.
4. **runner**: creates the session id, the session token and the session folder; generates the session mission (`INT-AR-04`) and, for a dedicated setup, `server.cfg` (`INT-AR-05`); copies the mission to `<game>\MPMissions\shakedown_<session id>.<world>` for a dedicated setup; writes every file and folder it created into the run record (`STP-03`).
5. **runner**: starts the processes, each one suspended, assigned to the job object, then resumed (`STP-01`):
   - **singleplayer**: `arma3_x64.exe` with `-autotest` pointing at the session's autotest list (`INT-AR-04`) as machine `p1`.
   - **dedicated**: `arma3server_x64.exe` as `server` with `-autoInit`; then, all at once, every headless client (`arma3server_x64.exe -client`) and every game client (`arma3_x64.exe -connect`).
6. **extension** (each machine): connects and says hello. **runner**: marks the machine `connected`.
7. **agent** (each machine): at mission start (`postInit`), registers the `ExtensionCallback` and `ScriptError` handlers and the entity recorder, then sends `ready` with the mission start count, `productVersion`, each loaded mod's `CfgPatches` version and, on a client, whether `player` exists. On a dedicated session a client is `ready` only when the server's agent also reports that client at client state 10 or later.
8. **runner**: when every machine of the setup is `ready`, captures the reset baseline (`RST-02`) through the lead and marks the session `ready`. If any process exits first, or the start ceiling passes (`SES-04`), it stops every process (`P-10`) and marks the session `failed` with the machine and the cause, and the last 20 lines of that machine's RPT.

## **P-03** An order round trip

Used by `exec`, by every check API call that names another machine, and by the runner's own probes.

1. **runner** (or the **lead**, through `send` with a `route` request): sends an order of kind `exec` to the target machine: SQF source text, arguments as JSON, the in-game ceiling, and the context (run id, check id, step path) it runs under.
2. **agent** (target): fetches the order, turns tagged objects in the arguments back into objects (`DM-14`), compiles the source with `compile`, sets the context variables, and `spawn`s the code with the arguments as `_this`. A compile error is answered at once as `check-error` with the engine's message.
3. **agent** (target): when the code returns, encodes the value (`DM-13`) and sends the result. If the in-game ceiling passes first, it terminates the script and sends `ceiling` with no value.
4. **runner**: forwards the result to whoever sent the order. If no answer arrives within the in-game ceiling plus the grace of `OBS-03`, the runner marks the target `lost`, answers the order with `machine-lost`, and the session goes `broken`.
5. **lead**: an order whose target is the lead machine itself runs locally, without the round trip, with the same encoding and ceilings.

## **P-04** A run

Triggered by `run` (command line) or `run_start` (MCP).

1. **client**: resolves the selection (check ids, tags) against the checks folder and refuses if it matches nothing, listing what exists. With `--setup` and no session running, it starts one (`P-02`) and stops it after the run (`P-10`), so the run leaves nothing running (`RUN-04`).
2. **runner**: refuses if the session is not `ready`, saying its state. Otherwise creates the run id and the run folder and marks the session `running`.
3. **runner**: for each selected check and each repeat, in order, executes the check (`P-05`). Checks whose `@setups` header excludes the session's setup are not executed and are listed as `not-run` with reason `setup`.
4. **runner**: after the last check, or when the run is stopped, cuts each machine's log for the run's time span into the run folder, redacted (`LOG-02`), writes the result file (`RES-01`), and marks the session `ready` (or leaves it `broken`).
5. **client**: the command line prints one line per check and the summary and exits with the code [contract/cli.md](contract/cli.md) gives; the MCP server answers `run_status` and `run_result` from the runner.

## **P-05** Executing one check

1. **runner**: reads the check file fresh from disk, parses its header (`API-01`). A header error ends the check as `fail` / `check-error` without touching the game.
2. **runner**: if the session is `broken`, ends the check `invalid` / `session-broken`.
3. **runner**: prepares: if the check's reset mode is `mission`, a mission restart (`P-08` level B); otherwise a reset (`P-06`). A failure ends the check `invalid` / `reset-failed` and leaves the session `broken`.
4. **runner**: tells every machine the current run and check id (an order that sets the context variables), then sends the lead an order of kind `check` with the check source, its id and its timeout.
5. **lead**: compiles the body (a compile error ends it `fail` / `check-error`), `spawn`s it, and records each step's start and end on the wall clock and the clocks of `OBS-05` as the body calls `skd_fnc_step`. Calls that target other machines go through `P-03`.
6. **agent** (every machine): stamps each caught script error with the context variables (the step path of the order it ran in, or of the check if none) and sends it at once (`ERR-01`).
7. **runner**: attributes each error to the check execution by its check id. An error with a check id but no step path is attributed to the step that was running on the lead at that wall-clock time.
8. **lead**: when the body ends, or calls `skd_fnc_fail` or `skd_fnc_invalid`, or an assertion or wait fails, or its timeout passes (it then terminates the body's script), sends the end report: state per step, the reason, the evidence.
9. **runner**: waits 1 s for script errors still in flight, then fixes the outcome by the rules `DM-01` to `DM-08`. If the end report does not arrive within the check's timeout plus the grace of `OBS-03`, the outcome is `invalid` / `machine-lost` naming the lead.
10. The state the check left stays in the game until the next reset, so a person or an AI agent can inspect it (`INS-01`).

## **P-06** A reset

Run before every check with reset mode `in-place`, and by the `reset` command. The **runner** drives it through orders; the whole reset has the ceiling of `RST-04`.

1. **lead**: runs the cleanups the previous check registered with `skd_fnc_cleanup`, last registered first. An error in one is logged, attributed to the previous check's execution, and the reset goes on. A cleanup still running when the reset's ceiling passes is terminated, and the reset fails naming the check that registered it.
2. **agent** (every machine): returns the entities its recorder saw created since the baseline that still exist, as net ids.
3. **agent** (server): deletes them: crew first with `deleteVehicleCrew`, then each object with `deleteVehicle`, then empty groups created since the baseline. Player units and the session mission's own units are never deleted.
4. **agent** (each player's machine, for its own unit): if the unit is dead, waits for its respawn (`RST-05`); then moves it out of any vehicle, restores its loadout with `setUnitLoadout`, sets its damage to 0 (and calls ACE's full heal when ACE medical is loaded, `INT-ACE-02`), sets its direction and then its position to the baseline, sets its velocity to zero and resets its animation with `switchMove ""`.
5. **agent** (server): restores the baseline date and weather (`setDate`, `setOvercast`, `setFog`, `setRain`, then `forceWeatherChange`) and broadcasts the date to every machine.
6. **runner**: verifies through each machine: no recorded entity exists, each player is within 0.1 m and 3 degrees of its baseline as the server sees it, and the date matches on every machine. A failure, or the ceiling passing, fails the reset with what did not match.

## **P-07** A stall: a machine stops answering

1. **runner**: an order to a machine has no answer within its in-game ceiling plus the grace (`OBS-03`), or a check's end report does not come within its timeout plus the grace.
2. **runner**: marks the machine `lost` with reason `no-answer`, marks the session `broken`, ends the check executing `invalid` / `machine-lost` with the machine, the order that went unanswered and the machine's last 20 RPT lines, and ends each following check of the run `invalid` / `session-broken`.
3. The game processes keep running, so a person or an AI agent can still look at the logs and the processes. The session leaves `broken` by a `game-restart` reload or by `session stop`.

## **P-08** A reload

Triggered by `reload [files...]`. **runner** drives it.

1. **Classify.** The runner takes the given files, or, with none given, compares every file under each watched mod's `source` folder (`CFG-01`) with the snapshot taken at the last ready state, by size, modification time and content hash. It classifies each changed file by `RLD-02`, asking the lead which functions each `.sqf` defines in `CfgFunctions` and whether each is final on every machine. The level applied is the highest any file needs.
2. **Level A, mission.** For each function to recompile, every machine recompiles it from its file (`BIS_fnc_recompile` with the function's name), then returns a hash of `toString` of the function. The level is done when every machine returns the hash of the new file's source. A machine returning another hash fails the reload.
3. **Level B, mission restart, singleplayer.** The lead ends the mission with `endMission "END1"`. The game loads the next entry of the `-autotest` list, which is the same session mission (`INT-AR-04`). The agent reports `ready` with the mission start count plus one.
4. **Level B, mission restart, dedicated.** The runner stops every client and headless client process (`P-10` step 2). The server's agent kicks any user still listed (`#kick` through `serverCommand` with the session's command password) until `allUsers` holds no one, with a ceiling of 20 s, then sends `#restart`. When the server's agent reports `ready` with the next mission start count, the runner starts fresh client and headless client processes (new start numbers) as in `P-02` step 5 and waits for them as in step 7.
5. **Level C, game restart.** The runner stops every process of the session (`P-10` steps 1 to 3, keeping the session folder and id) and starts them again as in `P-02` steps 5 to 8 with new start numbers.
6. **Verify.** After level B or C, the runner takes a new baseline and a new file snapshot. The reload result names the level applied, each file with its level and reason, and the time taken on the wall clock (`RLD-03`).
7. A failure at any step marks the reload `failed` and the session `broken`, with the step and the machine.

## **P-09** A crash: a process exits unasked

1. **runner**: sees a game process of the session end without having asked it to (it waits on every process handle).
2. **runner**: records the exit code, looks for a new `.mdmp` or `.bidmp` in that machine's profile folder and copies each one into the active run's folder as `dumps/<machine>/<file>`, marks the machine `lost` with reason `exited`, and marks the session `broken`.
3. **runner**: the check executing ends `invalid` / `machine-lost` naming the machine, its exit code and any dump; following checks of the run end `invalid` / `session-broken`.
4. On a dedicated session the other processes keep running until the session stops or a `game-restart` reload replaces every machine.

## **P-10** Stop a session

Triggered by `session stop`, by the end of a run that started its own session (`RUN-04`), or by a failed start.

1. **runner**: marks the session `stopping` and, if a run is active, stops it (its executing check ends `invalid` / `run-stopped`).
2. **runner**: stops the game clients and headless clients first, then the server, so no machine logs the errors of a mission ending under it: each by its process handle, with `TerminateProcess` after asking nothing, since no clean shutdown is needed for a test session. It waits on each handle with a ceiling of 10 s.
3. **runner**: deletes the mission copy from `<game>\MPMissions`, redacts each machine's RPT in the session folder (`LOG-02`), removes the session from the run record, and marks the session `stopped`.
4. **runner**: if a process outlives step 2's ceiling, it closes the job object, which ends every process left in it, and reports the process id.

## **P-11** Ask a running game what is going on

Triggered by `status`, `inspect`, `exec`, `logs` or `errors` at any time, including while a run is active or the session is `broken`.

1. **client**: sends the request to the runner. `status` needs no game: it answers from the runner's own state, and says when no runner or no session runs.
2. **runner**: for `inspect` and `exec`, sends an order to the named machine (`P-03`) that runs alongside a running check without stopping it. For `logs` and `errors`, reads its own records and the machine's RPT.
3. **client**: returns the compact answer of `INS-01`, with paths for anything larger.

## **P-12** Recovery after the runner died

Triggered at every runner start, before it accepts a client.

1. **runner**: reads the run record `%LOCALAPPDATA%\Shakedown\record.json`.
2. **runner**: for each process id it lists, if a process with that id runs and its start time matches the recorded one, terminates it and logs it.
3. **runner**: deletes every mission copy and temporary file the record lists, and marks each listed session `stopped` in its session folder with reason `runner-died`.
4. **runner**: empties the record.

Processes the runner started end with it anyway, because the job object closes when the runner dies (`STP-01`). This process covers what a job object cannot: processes that escaped it, files and the record of sessions.
