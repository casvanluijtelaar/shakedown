---
type: Documentation
title: Troubleshooting
description: What to do when a session does not start, a port is taken, the project file is refused, a session is broken, checks end invalid, fail with errors you did not cause or stop at the runner ceiling, the runner does not answer, the install fails, code on a client sees objNull or an editor cannot save a mod file, and the known limits.
status: draft
---

# Troubleshooting

Start with `shakedown status`. It works at any time, starts nothing, and shows the runner, the session with each machine's state, process id, profile folder and health, a failed start with its reason, and the run in progress. When a command exits 5, the framework itself failed: read `%LOCALAPPDATA%\Shakedown\runner.log`.

## A session does not start

`session start` checks its preconditions before it starts any game process, and when one fails it exits 4 naming everything missing ([Starting a session, and when it fails](starting-and-failed-starts.md)):

```
The session cannot start; missing: a running Steam client of this user (start Steam and log in). Provide what is missing and start the session again.
```

| Missing | Fix |
|---|---|
| `a running Steam client of this user (start Steam and log in)` | Start Steam and log in to an account that owns Arma 3, then start again. |
| `an account logged in to Steam: Steam runs, but no account is logged in (log in to Steam)` | Log in to Steam. |
| `the game executable ... (check `game` in shakedown.toml)` | Point `game` at the folder that holds `arma3_x64.exe`, and for a dedicated setup `arma3server_x64.exe`. |
| `the folder ... of mod ...` | Fix the mod's `path`, or build the mod first. |
| `the framework's addon file ... (install Shakedown again)` | Keep `@shakedown` next to `shakedown.exe`, or install again. |
| `2 GB free on drive ...` | Free space on the drive that holds the project's `.shakedown` folder; old run and session folders there can go. |
| `free ports 2402 to 2406 for the server; ... held by another program` | Stop the program holding them, often a game or server left from another run, or set `base_port` in `shakedown.toml`. |
| `the framework's link ...\@shakedown in the game folder: a folder of that name that is not a link is there (move it away)` | Move the real `@shakedown` folder out of the game folder. |

Through the [MCP tools](agent-tools.md) the same refusal is `precondition-failed`.

- **Steam runs, but the start fails with `Steam is not running or not logged in (p1 exited with code 53)`.** The game exits with 53 when it cannot reach a logged-in Steam client. Make sure Steam is logged in, not only running, and is the same Windows user's.
- **`Error 20 reading file`.** A game process logs it on one of the game's own base-game PBOs and ends with `Application terminated intentionally` a few seconds after it starts. It does not come from Shakedown or your mod, and a second start usually works: run `session start` again.
- **The start passes its ceiling.** At `start_timeout` (180 s by default) the start fails, listing every machine that was not ready, with the last 20 RPT lines of the machine at fault. Read those lines, and each machine's RPT in its profile folder. A dedicated start with large mods may need a longer `start_timeout`. A client that hangs during its start is only seen at the ceiling, so the start waits the full time first.
- **A start was refused, and a quick retry too.** A refused start still starts the runner, which stays up about 60 s with no session. A retry within that minute meets it; that is expected, and the retry works once the precondition holds.
- **A mod in another version.** With `version` set on a mod, a start on any other version fails, naming the mod, the machine, the version found and the version wanted. Install that version, or set `version` to the one found.
- **Where the reason went.** `status` keeps a failed start, state `failed` with its reason, until the next `session start` or `session stop`; after that, `session.json` in the session folder holds it.

## Ports

A dedicated server uses `base_port` (default 2402) and the next four ports, and the start refuses when one is held. Another game or server on the machine, or a server left by a session that did not stop, holds them. Check `shakedown status` for a session still up and stop it with `shakedown session stop`. Otherwise give the project another `base_port`, from 1024 to 65530. A singleplayer setup uses no server port.

## The project file is refused

A command that reads `shakedown.toml` refuses a file with a mistake, exit code 3, naming the key and its line ([The project file](project-file.md#mistakes)). The folder of the check files is the key `tests`, default `tests`; a file that names it with `checks` is refused as an unknown key. Through the MCP tools the same refusal is `config-error`.

## A check's header is wrong

`shakedown list` names every header mistake, with the line, without the game. The check itself ends `fail` / `check-error` with the same message ([Writing checks](writing-checks.md#when-the-check-itself-is-wrong)).

## A broken session

`status` shows the session `broken` with a `reason`: a machine was lost, a reset failed, a reload failed, or a client joined a server not the session's own. Every check run on it ends `invalid` / `session-broken`, `run` is refused with exit 4, and `reset` is refused.

1. Read the reason in `status`: the message, the machine and its last 20 RPT lines.
2. Look while it is still up: `shakedown inspect`, `shakedown logs --machine <m>`, `shakedown errors`, `shakedown exec` on the machines that answer ([Looking into a broken session](outcomes-and-broken-sessions.md#looking-into-a-broken-session)).
3. Get out with `shakedown reload`, which on a broken session restarts the game of every machine, or with `shakedown session stop`.

## Checks end `invalid`

- **`unhealthy`.** A machine's mission time or frame stopped, its frame rate fell under 10 frames a second, or its answers took longer than 2 s, while it still answered. On a remote machine, the likeliest cause is a game client whose remote desktop stream was cut: a client window renders only while someone is connected to the desktop, and runs about 19 frames a second while it is not presented. A singleplayer game paused with Escape stops its clock too, and so does a machine starved of CPU by other programs. Keep the desktop connected while a session with game clients runs, and keep other heavy programs off the machine.
- **`machine-lost`.** A process exited (a crash, or something ended it), or the machine stopped answering for 10 s. The message says which, with its exit code and crash dumps under `dumps\` in the run folder. A crash in the mod is still worth looking into: run the check again and read the machine's last RPT lines in the evidence.
- **`reset-failed`.** The reset could not bring the session back within 15 s. The message lists what did not match, each item naming its machine, or names the check whose cleanup did not end, or the ACE version without the full heal. Keep cleanups short, and stop the units a check made from acting at its end, such as an enemy whose shot in flight hits a player who is already back at the start ([When the reset fails](scenes-resets-and-cleanups.md#when-the-reset-fails)).
- **`run-stopped` with "The run reached its runner ceiling".** The run took longer than the sum of its checks' `@timeout` plus 60 s. A check's `@timeout` is too short for what it, its reset or its cleanups take: raise it ([The runner ceiling](unattended-runs.md#the-runner-ceiling)).

## A check passes alone and fails after another

An earlier check left state the reset does not touch: a variable, an action on a player, a mod's setting. Register a cleanup in the earlier check, or give the later one `// @reset mission` ([Scenes, resets and cleanups](scenes-resets-and-cleanups.md#what-the-reset-does-not-touch)).

## Checks fail with errors you did not cause

- A `script-error` from the game's death-camera script, such as `Undefined variable ... tsw_camtargetunit`, can follow a singleplayer check in which `p1` died.
- On a dedicated session, the server can log `Server: Object n:nnn not found` for a dead player's body the reset deletes while it still falls. It comes from the game and the reset, not from your mod.

## A check fails with `wait-ceiling` on something that plainly happened

- **The object had not reached the machine yet.** On a dedicated session a client sees an object the server just created a moment later. `skd_fnc_on` looks an object up once, so it can get `objNull`. Wait for the object on that machine with `skd_fnc_waitUntil` first ([When an object is looked up](dedicated-sessions.md#when-an-object-is-looked-up)).
- **`player` on the server.** On a dedicated session the body runs on the server, where `player` is `objNull`. Use `["p1"] call skd_fnc_player`, or run the code on the client with `skd_fnc_on`.
- **The ceiling is too short** for a step that goes through the network or a long animation. Give it a longer one.

## A command says the runner does not answer

```
The runner (process 12380) does not answer. Ending that process ends its session's game processes with it; the next command then starts a fresh runner.
```

The runner is paused, hung, or of another Shakedown version left from before an install. Resume it, or end it by the process id it names, and only by that id: `taskkill /PID 12380 /F`. Its game processes end with it, and the next command starts a fresh runner that clears what the old one left ([Sessions and the runner](sessions-and-the-runner.md#a-command-cannot-reach-the-runner)).

**Never delete `runner.json`** while a runner holds a session: the next command would start a second runner, whose start-up recovery ends the first session's game.

## The install or upgrade fails

- **The binary is locked.** A runner or an agent's `shakedown mcp` still runs. Stop the session, wait for the runner to exit (60 s after its last session stopped), end the agent session, or end the runner by the process id `status` gives, then install again. The install script names each process that runs from its folder.
- **HEMTT is missing** (an install from source). The Windows install builds the addon with HEMTT 1.23; put it on `PATH`.
- **SmartScreen warns.** The binary is not signed; allow it.

## An editor cannot save a mod file while a session runs

The game holds every loose mod file it reads. Load a build of your files, not your files themselves, and do not load the mod through `hemtt dev` or `hemtt launch` links ([Setting a mod up for reload](reloading.md#setting-a-mod-up-for-reload)).

## A game window takes the foreground

Every game client opens its window, which can take the foreground from whatever you are doing on the machine. Leave the machine to the run while a session with game clients starts, or use a setup with fewer clients.

## Nothing must be left running

After a run, `shakedown status` shows no session, and no `arma3_x64` or `arma3server_x64` process of the session runs. If a session is still up, `shakedown session stop` ends it. End a game process by its process id from `status`, never by name: another Arma 3 on the machine may not be yours.

## Known limits

- One session runs on a machine at a time.
- A session needs Windows, Arma 3 from Steam and a logged-in Steam client, so a hosted CI runner cannot run one.
- A check cannot send a real key to the game, take screenshots, or restart the server during a session.
- `skd_fnc_create` makes a vehicle as the engine's `createVehicle` does, so a class that randomises its look can differ between runs.
- A start whose client hangs fails only at its `start_timeout`.
- Cancelling the `session_start` or `reset` tool does not stop the start or the reset.
- A reload of a `.sqf` on a dedicated session takes about as long as a game restart, since the clients start again.
- The privacy limits in [Security and privacy](security-and-privacy.md#not-covered-yet).
