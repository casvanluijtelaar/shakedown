---
type: Specification
title: Processes
description: Every flow of Shakedown end to end, naming the process or machine that does each step - starting a session, an order round trip, running a check, a reset, a reload at each level, a stall, a crash, stopping, and recovery after the runner dies.
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# Processes

Each process has an ID of the form `P-nn`. A step names who does it: **client** (the command line or the MCP server), **runner**, **extension** (in a named machine), **agent** (the in-game agent on a named machine), **lead** (the agent on the lead machine), or **game** (the engine itself). Ceilings and values named here are defined in [03-functional-requirements.md](03-functional-requirements.md); the start options and configuration keys in [04-integrations.md](04-integrations.md).

## The channel

Every process below uses one channel, so it is described once:

- The runner listens on `127.0.0.1` only, on a port it picks at its own start (`SEC-01`).
- Each game process gets four environment variables from the runner when it is started: `SHAKEDOWN_RUNNER` (`127.0.0.1:<port>`), `SHAKEDOWN_SESSION`, `SHAKEDOWN_MACHINE` and `SHAKEDOWN_TOKEN` (`SEC-02`).
- When the game loads the extension, the extension opens one connection to the runner and sends a hello with the session id, machine name and token. The runner accepts it only if the token matches the session's and the process at the other end of the connection, as the system's TCP table names it, is the one the runner started for that machine (`SEC-03`). Otherwise it closes the connection and logs the refusal.
- **Runner to game.** The runner sends an order (id, kind, payload) to the extension. The extension queues it and calls the engine's callback with name `shakedown`, function `order` and the order id as data. The agent's `ExtensionCallback` handler fetches the order with `"shakedown" callExtension ["fetch", [id, part]]` in parts of at most 10000 bytes until the last part, since one `callExtension` result is limited to 10240 bytes. It fetches in a scheduled script, so a large order spreads over frames.
- **Game to runner.** The agent sends results, errors and requests with `"shakedown" callExtension ["send", [json]]`, which carries any size in one call. The extension queues the message and returns at once; its own thread writes it to the runner.
- The extension does no work inside `callExtension` beyond queueing, and retries a callback the engine refused (`INT-AR-03`).

## **P-01** Start the runner

1. **client**: reads `%LOCALAPPDATA%\Shakedown\runner.json` (port, token, process id and start time of the runner). If it names a live runner, a running process with that id whose start time falls in the same second as the file's, the client connects with that token, sends its hello and waits up to 5 s for the runner's answer, which carries the runner's version. An answer of the client's own version ends the process here. With no answer, or an answer of another version, the client starts no other runner: it refuses the request, naming the runner's process id and, when it answered, its version, and saying that ending that process ends its session's game processes with it (`STP-01`) and that the next command then starts a fresh runner. `status` answers instead with the runner `unreachable` (`INS-02`).
2. **client**: otherwise deletes a `runner.json` that names no live runner, starts `shakedown runner` as a detached process outside the client's own job object and process tree (`SES-08`, `INT-WIN-02`), and waits for `runner.json` to appear, polling every 100 ms with a ceiling of 10 s.
3. **runner**: at start, before it writes `runner.json`, runs the recovery of `P-12`, picks a free port on `127.0.0.1`, makes a random 256-bit token in the form of `SEC-06`, and writes `runner.json`, with its own process's start time, readable by the current user only.
4. **runner**: exits by itself 60 s after its last session stopped with no client connected; it then deletes `runner.json`.

## **P-02** Start a session

Triggered by `session start --setup <name>` (or by `P-04` when no session runs).

1. **client**: reads `shakedown.toml`, checks it against `CFG-01`, and sends the setup to the runner. A malformed file is refused here with the key at fault and its line; an unknown setup with the setups the file declares.
2. **runner**: refuses if a session already runs (`SES-02`), naming it and the command that stops it.
3. **runner**: checks the preconditions of `SES-03`: the game executables exist, a process `steam.exe` runs in the current user's Windows session, every mod folder of the setup exists, the framework's installed addon holds its PBO and its extension, and for a dedicated setup the setup's ports are free and the framework's addon link exists in the game folder (creating it if missing, `INT-AR-06`). Any failure ends the session as `failed` before a process starts, naming what is missing.
4. **runner**: creates the session id, the session token (`SEC-06`) and the session folder; generates the session mission (`INT-AR-04`) and, for a dedicated setup, `server.cfg` (`INT-AR-05`); copies the mission to `<game>\MPMissions\shakedown_<session id>.<world>` for a dedicated setup; writes every file and folder it created into the run record (`STP-03`).
5. **runner**: creates the session's job object with kill-on-close and starts the processes, each one suspended, assigned to that job object, then resumed (`STP-01`):
   - **singleplayer**: `arma3_x64.exe` with `-autotest` pointing at the session's autotest list (`INT-AR-04`) as machine `p1`.
   - **dedicated**: `arma3server_x64.exe` as `server` with `-autoInit`; then, once the server is `ready`, all at once, every headless client (`arma3server_x64.exe -client`) and every game client (`arma3_x64.exe -connect`), so no client tries to join a server whose mission is not up.
6. **extension** (each machine): connects and says hello. **runner**: marks the machine `connected`.
7. **agent** (each machine): at mission start (`postInit`), registers the `ExtensionCallback` and `ScriptError` handlers, the entity recorder and the `EntityRespawned` handler of `RST-05`, and in singleplayer the player's `Killed` handler of `RST-05`; once its player stands still on the ground in a settled animation state, or, on a machine without a player of its own (a dedicated server, a headless client), once the mission time runs, captures the reset baseline (`RST-02`); then sends `ready` with the mission start count, `productVersion`, whether it has a player, and its baseline. On a dedicated session a client is `ready` only when the server's agent also reports that client at client state 10 or later; the runner asks the server every 0.5 s for each client whose agent sent `ready` (`INT-AR-09`).
8. **runner**: when every machine of the setup is `ready`, asks each machine with an `exec` order for the `versionStr` of the `CfgPatches` class of each mod of the setup that has a `patch` key and marks the session `ready`. If any process exits first, or the start ceiling passes (`SES-04`), it stops every process (`P-10`) and marks the session `failed` with the machine and the cause, and the last 20 lines of that machine's RPT. The machine named is the one whose process exited, or at the ceiling the first machine not `ready`; at the ceiling the cause lists each machine that was not `ready` with its state, or says it was not launched because the server was not ready.

## **P-03** An order round trip

Used by `exec`, by every check API call that names another machine, and by the runner's own probes.

1. **runner** (or the **lead**, through `send` with a `route` request): sends an order of kind `exec` to the target machine: SQF source text, arguments as JSON, the in-game ceiling, and the context (run id, check id, step path) it runs under. A `route` request names the target, a reference of the lead's own and the order; the runner sends the order on and gives the lead the target's answer in an order of kind `routed` that carries the reference.
2. **agent** (target): fetches the order, turns tagged objects in the arguments back into objects (`DM-14`), compiles the source with `compile`, sets the context variables, and `spawn`s the code with the arguments as `_this`. A compile error is answered at once as `check-error` with the engine's message.
3. **agent** (target): when the code returns, encodes the value (`DM-13`) and sends the result. If the in-game ceiling passes first, it terminates the script and sends `ceiling` with no value.
4. **runner**: forwards the result to whoever sent the order. If no answer arrives within the in-game ceiling plus the grace of `OBS-03`, the runner marks the target `lost`, answers the order with `machine-lost`, and the session goes `broken`.
5. **lead**: an order whose target is the lead machine itself runs locally, without the round trip, with the same encoding and ceilings.
6. **A poll.** A wait or a polled assertion on another machine (`OBS-01`, `OBS-06`) is a routed order of kind `poll`: the target evaluates the code every frame and sends each value that differs from the last one it sent, at most one every 0.05 s, as a `poll` message; the runner forwards each to the lead as an order of kind `routed-poll`, and the lead keeps the newest. The lead stops the poll with a `route-stop` request, which the runner passes to the target as an order of kind `stop`; a poll also ends by itself 5 s after its ceiling.

## **P-04** A run

Triggered by `run` (command line) or `run_start` (MCP).

1. **client**: resolves the selection (check ids, tags) against the checks folder and refuses if it matches nothing, listing what exists. With `--setup` and no session running, it starts one (`P-02`) and stops it after the run (`P-10`), so the run leaves nothing running (`RUN-04`).
2. **runner**: refuses if the session is not `ready`, saying its state. Otherwise creates the run id and the run folder and marks the session `running`.
3. **runner**: for each selected check in the selection's order, a check given more than once each time it is given (`RUN-03`), and for each repeat, executes the check (`P-05`). Checks whose `@setups` header excludes the session's setup are not executed and are listed as `not-run` with reason `setup`.
4. **runner**: after the last check, or when the run is stopped, cuts each machine's log for the run's time span into the run folder, redacted (`LOG-02`), writes the result file (`RES-01`), and marks the session `ready` (or leaves it `broken`).
5. **client**: the command line prints one line per check and the summary and exits with the code [contract/cli.md](contract/cli.md) gives; the MCP server answers `run_status` and `run_result` from the runner.

## **P-05** Executing one check

1. **runner**: reads the check file fresh from disk, parses its header (`API-01`). A header error ends the check as `fail` / `check-error` without touching the game.
2. **runner**: if the session is `broken`, ends the check `invalid` / `session-broken`.
3. **runner**: if the check's reset mode is `mission`, restarts the mission first (`P-08` level B), since a mission restart clears the context variables and a replaced machine writes a new RPT. Then it tells every machine the run, the check id and the execution's key `<run id>#<n>` (an order that sets the context variables), and each machine writes the execution's start marker into its RPT (`LOG-04`); then, for reset mode `in-place`, resets (`P-06`). A failure of either ends the check `invalid` / `reset-failed`, naming the machine that did not come back or did not verify, and leaves the session `broken`.
4. **runner**: sends the lead an order of kind `check` with the check source, its id and its timeout.
5. **lead**: compiles the body (a compile error ends it `fail` / `check-error`), `spawn`s it, and records each step's start and end on the wall clock and the clocks of `OBS-05` as the body calls `skd_fnc_step`. Calls that target other machines go through `P-03`.
6. **agent** (every machine): sends each caught script error at once, stamped with the context variables and a step path: on the lead the innermost step running in the check; on another machine the step path the newest order still running there carried, or none (`ERR-01`).
7. **runner**: attributes each error to the check execution by the key it was stamped with. An error with no step path is attributed to the innermost step that was running on the lead at its wall-clock time, or, between two steps, to the last step that had started.
8. **lead**: when the body ends, or calls `skd_fnc_fail` or `skd_fnc_invalid`, or an assertion or wait fails, or its timeout passes (it then terminates the body's script), sends the end report: state per step, the reason, the evidence.
9. **runner**: waits 1 s for script errors still in flight, has every machine write the execution's end marker and clear the context, cuts each machine's lines between the two markers out of its RPT into the run folder, redacted (`LOG-04`), adds the script errors those lines show that no handler sent (`ERR-02`), marks the errors an `expectError` block matched (`ERR-04`), then fixes the outcome by the rules `DM-01` to `DM-08`. If the end report does not arrive within the check's timeout plus the grace of `OBS-03`, the outcome is `invalid` / `machine-lost` naming the lead.
10. The state the check left stays in the game until the next reset, so a person or an AI agent can inspect it (`INS-01`).

## **P-06** A reset

Run before every check with reset mode `in-place`, and by the `reset` command. The **runner** drives it through orders in four phases; each phase after the first is one order to every machine at once, and the next phase starts when every machine has answered. The whole reset has the ceiling of `RST-04`. In singleplayer every phase runs on `p1`.

1. **lead**: runs the cleanups the previous check registered with `skd_fnc_cleanup`, last registered first. An error in one is listed with the execution whose reset ran it, without failing it (`ERR-03`), and the reset goes on. A cleanup still running when the reset's ceiling passes is terminated, and the reset fails naming the check that registered it.
2. **agent** (each machine with a player, for its own unit): if the unit is dead, waits for its respawn (`RST-05`); then moves it out of any vehicle, so no deletion takes a player with it.
3. **agent** (every machine): deletes the entities its own recorder saw created since the baseline that still exist and are local to it, so that every entity is deleted by the one machine it is local to and no machine asks another to delete an object that one has deleted already: crew first with `deleteVehicleCrew`, then each object with `deleteVehicle`. Player units and headless client units are never deleted; the body a dead player left is (`RST-05`). On a machine with a player it restores its own unit: its loadout with `setUnitLoadout`, its damage to 0 (and ACE's full heal when ACE medical is loaded, `INT-ACE-02`), its direction and then its position to the baseline, its velocity to zero, its group's baseline behaviour, since a group that saw an enemy during the check stays in combat and raises the player's weapon, and its baseline animation state with `switchMove`. Every machine then sets the server's baseline date and weather itself (`setDate`, `setOvercast`, `setFog`, `setRain`, then `forceWeatherChange`), which the runner sends with the order from the server's `ready`, so no machine waits for the server's to reach it.
4. **agent** (every machine): verifies, every frame, deleting any entity its recorder saw that is local to it by then, such as a vehicle a player has just left or a crater made while the reset runs: no entity its recorder saw is left, its date is the baseline's to the minute, and its own player is alive, on foot and within 0.1 m and 3 degrees of its baseline; on the lead, also each player's unit within 0.1 m and 3 degrees of that player's baseline as the lead sees it, and the overcast within 0.01. While its own player is more than 2 cm from its baseline position or more than 1 degree from its baseline direction it sets its direction and position there again, and while its animation state is not the baseline's it puts it back with `switchMove`, for up to 1.5 s, and the verification holds only once the player has stayed within 2 cm, within 1 degree and in that animation state for 0.3 s, since a unit just out of a vehicle can still move as its get-out ends, and a unit can be turned in place after it is set, with no step away from its position. Once it holds, the machine deletes the empty groups of its own created since the baseline. A machine that does not verify by the ceiling fails the reset with what did not match, each item named with its machine.

## **P-07** A stall: a machine stops answering

1. **runner**: an order to a machine has no answer within its in-game ceiling plus the grace (`OBS-03`), a check's end report does not come within its timeout plus the grace, or a `ready` machine's heartbeat has not come for 10 s of the runner's own running time (`HLT-02`).
2. **runner**: marks the machine `lost` with reason `no-answer` and the wall-clock time the ceiling or the heartbeat's 10 s passed, answers every order still waiting on it as lost, marks the session `broken`, ends the check executing at once `invalid` / `machine-lost` with the machine, the orders it was given that went unanswered and the machine's last 20 RPT lines, and ends each following check of the run `invalid` / `session-broken`. A lost machine is sent no further order: `exec` returns it `lost` and `inspect` returns it with state `lost`, at once.
3. The game processes keep running, so a person or an AI agent can still look at the logs and the processes. The session leaves `broken` by a `game-restart` reload, which any `reload` of a broken session is (`RLD-01`), or by `session stop`.

## **P-08** A reload

Triggered by `reload [files...]`, and its step B by a check with `@reset mission` (`RST-06`). **runner** drives it. A reload applies level none or level C: every change of mod code is level C, because the game holds every mod file it has read and reads a changed one only in a new process (`RLD-02`, `SPK-08`). On a `broken` session the level applied is C whatever the files need, since only a game restart replaces every machine (`RLD-01`).

While a machine's mission restarts or its process is replaced, at level B or C, the runner holds every new order to it until the machine is `ready` again, for at most the setup's start ceiling (`SES-04`), and then sends it with its whole ceiling; before it ends a mission or a process it waits up to 10 s for the orders already sent there to be answered, so no order is lost or run twice across a restart (`NFR-RL-04`).

1. **Classify.** The runner takes the given files, or, with none given, compares every file under each watched mod's `source` folder (`CFG-01`) with the snapshot taken at the last ready state, by size, modification time and content hash. It classifies each changed file by `RLD-02`. The level applied is the highest any file needs.
2. **Level B, mission restart, singleplayer.** The lead ends the mission with `endMission "END1"`. The game loads the next entry of the `-autotest` list, which is the same session mission (`INT-AR-04`). The agent reports `ready` with the mission start count plus one.
3. **Level B, mission restart, dedicated.** The runner stops every client and headless client process (`P-10` step 2). The server's agent kicks any user still listed (`#kick` through `serverCommand` with the session's command password) until `allUsers` holds no one, with a ceiling of 20 s, then sends `#restart`. When the server's agent reports `ready` with the next mission start count, the runner starts fresh client and headless client processes (new start numbers) as in `P-02` step 5 and waits for them as in step 7.
4. **Level C, game restart.** The runner stops every process of the session (`P-10` steps 1 to 3, keeping the session folder and id). With no game process left to hold a mod file, it runs the `rebuild` of each of the setup's mods that has one, in the setup's order, in the project folder (`RLD-06`); then it starts the processes again as in `P-02` steps 5 to 8 with new start numbers. The stop, the change and the start are one step, so the change is made while no game process holds the files.
5. **Verify.** After level B or C, every agent has captured a new baseline at the mission start (`RST-02`), and the runner takes a new file snapshot; it takes one too after a reload of level none and after the mission restart of a `@reset mission` check. The reload result names the level applied, each file with its level and reason, the machines it reached, and the time taken on the wall clock (`RLD-03`).
6. A failure at any step marks the reload `failed` and the session `broken`, with the step (the level being applied) and the machine that did not come back, in the reload's `reason` and the session's. A failed rebuild names the mod, and the lead as the machine.

## **P-09** A crash: a process exits unasked

1. **runner**: sees a game process of the session end without having asked it to (it waits on every process handle).
2. **runner**: records the exit code and the wall-clock time of the exit, looks in that machine's profile folder for every `.mdmp` and `.bidmp` file written after the machine's process started, waiting up to 5 s for a file still being written, and copies each one into the active run's folder as `dumps/<machine>/<file>`, marks the machine `lost` with reason `exited`, and marks the session `broken`.
3. **runner**: the check executing ends `invalid` / `machine-lost` naming the machine, its exit code and any dump; following checks of the run end `invalid` / `session-broken`.
4. On a dedicated session the other processes keep running until the session stops or a `game-restart` reload replaces every machine.

## **P-10** Stop a session

Triggered by `session stop`, by the end of a run that started its own session (`RUN-04`), or by a failed start.

1. **runner**: if a run is active, stops it as `RUN-05` says, with `stopped_reason` `session-stopped`, and waits for its result file; then marks the session `stopping`.
2. **runner**: stops the game clients and headless clients first, then the server, so no machine logs the errors of a mission ending under it: each by its process handle, with `TerminateProcess` after asking nothing, since no clean shutdown is needed for a test session. It waits on each handle with a ceiling of 10 s.
3. **runner**: deletes the mission copy from `<game>\MPMissions`, redacts each machine's RPT in the session folder (`LOG-02`), removes the session from the run record, and marks the session `stopped`, also in the `session.json` of its session folder.
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
3. **runner**: deletes every mission copy and temporary file the record lists, and marks each listed session `stopped` with reason `runner-died` in the `session.json` of its session folder.
4. **runner**: empties the record.

Processes the runner started end with it anyway, because the job object closes when the runner dies (`STP-01`). This process covers what a job object cannot: processes that escaped it, files and the record of sessions.
