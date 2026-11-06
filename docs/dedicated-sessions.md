---
type: Documentation
title: Dedicated sessions
description: How to declare and start a dedicated server with game clients and headless clients on one machine, how a check names each machine and group, how code, waits and agreement reach other machines, what the reset does across machines, and how to see a session's machines with status.
status: draft
---

# Dedicated sessions

A dedicated session is a dedicated server with game clients and headless clients, all on the one Windows machine, all on the one Steam account. A check runs on the server and reaches every other machine by its name. Its contract is [spec/contract/check-api.md](../spec/contract/check-api.md) and [spec/contract/cli.md](../spec/contract/cli.md); the project file keys are in [The project file](project-file.md).

## Declaring one

```toml
[setups.dedicated]
mode = "dedicated"
clients = 2            # game clients p1 and p2
headless = 1           # headless client hc1
mods = ["cba", "agh"]
```

The server uses `base_port` (default 2402) and the next four ports; the start refuses when one is held. A dedicated session loads the framework's addon through the link `<game>\@shakedown`, because a dedicated server and a headless client load an extension only from inside the game folder; the start makes that link when it is missing and points it at the installed `@shakedown` when it points elsewhere. A mod with an extension of its own must also be inside the game folder for the server and headless clients, as `@skd_fixture` is through its link `<game>\@skd_fixture`.

## Starting and stopping

```
shakedown session start --setup dedicated
shakedown status
shakedown session stop
```

`session start` starts the server, waits for its mission, then starts every game client and headless client at once, and returns when every machine is ready: a client is ready when its agent answers and the server sees it at client state 10. On the test machine a server with two clients and a headless client is ready in 43 to 45 s, so a `start_timeout` below 60 s cannot hold it. Each game client opens a window, titled `Arma 3 - p1 (<session id>)` once its extension connects; a window renders only while someone is connected to the machine's desktop. The server and headless clients have no window.

A start that fails, because a process exits or the start ceiling passes, ends with exit code 4 and stops every process it started. Its message names the machine, what state it was in, and the last 20 lines of its RPT; at the ceiling it lists every machine that was not ready. With `--json` it prints the session with state `failed` and its `reason`.

`status` works at any time, also while a session starts in another command. It lists each machine from the moment its process is launched:

```
runner: running (process 9184)
session: s-20261106-092156 of setup dedicated, ready
  server  server    ready      start 1 process 3116 profile ...\.shakedown\sessions\s-20261106-092156\machines\server-1
  p1      client    ready      start 1 process 3604 profile ...\machines\p1-1
  p2      client    ready      start 1 process 7484 profile ...\machines\p2-1
  hc1     headless  ready      start 1 process 5644 profile ...\machines\hc1-1
```

The process id finds a machine's window or lets a tool end or suspend that one process; the profile folder holds its RPT. `session stop` ends the clients and headless clients first, then the server, and deletes the mission copy it put in `<game>\MPMissions`.

## Machines and groups in a check

| Name | Is |
|---|---|
| `"server"` | The dedicated server. The check body runs here. It has no player: `player` in the body is `objNull`. |
| `"p1"` to `"pN"` | The game clients, each with a player unit. |
| `"hc1"` to `"hcN"` | The headless clients, with no player. |
| `"all"` | Every machine, in the order `server`, `p1`..`pN`, `hc1`..`hcN`. |
| `"clients"` | `p1` to `pN`. |
| `"headless"` | `hc1` to `hcN`. |

In singleplayer the only machine is `p1`, which `"server"` also names, and `"all"` and `"clients"` are `["p1"]`.

```sqf
private _p1 = ["p1"] call skd_fnc_player;          // p1's unit, as the server sees it
private _each = ["all", { profileName }] call skd_fnc_onEach;
// _each: server -> "server", p1 -> "p1", p2 -> "p2", hc1 -> ...
```

A check meant to run in both kinds of setup names a player's unit with `skd_fnc_player`, never with `player`, and names machines through `skd_fnc_machines` or a group where a machine may not exist. This check runs unchanged in singleplayer and on a dedicated session:

```sqf
// @check US-019 p1 opens the crate and the server and every client see the kit
private _p1 = ["p1"] call skd_fnc_player;
private _on = (call skd_fnc_machines) select { (_x select [0, 2]) != "hc" };   // server and clients
private _crate = ["skdfx_Crate", AGLToASL [100, 102, 0], 0] call skd_fnc_create;
["p1", AGLToASL [100, 100, 0], 0] call skd_fnc_place;
private _kits = { { _x == "FirstAidKit" } count items (_this select 0) };
private _before = ["server", _kits, [_p1]] call skd_fnc_on;
["p1", _crate, "Open crate"] call skd_fnc_useAction;
[_on, _kits, "==", _before + 1, "p1's first aid kits", 10, [_p1]] call skd_fnc_expectEach;
```

## Code, waits and agreement across machines

Code given to `skd_fnc_on`, `skd_fnc_onEach`, a wait or a polled assertion is sent to the named machine as text, through the runner, and runs there; objects and groups in `args` and in the result travel by net id. A round trip to another machine takes about 12 ms to a client and about 43 ms to the server or a headless client, which run at a lower frame rate.

A wait or polled assertion on another machine evaluates its code there every frame and sends each new value back, at most one every 0.05 s; the server compares the newest value of each machine. `skd_fnc_expectAgree` holds when every machine reads the first machine's value; when one does not, the check fails naming it:

```
flag agrees: the machines did not agree within 2 s (tolerance 0): p2 read 6 where server read 5. Every machine read: server 5, p1 5, p2 6.
```

`reason.machine` is `p2`, and the result's `evidence.observed` holds each machine's value.

`skd_fnc_useAction` waits for the action and fires it on the acting client; `"server"` or a headless client as the acting machine ends `check-error`, since it has no player. `skd_fnc_pressKey` presses the keybind on the named machine. `skd_fnc_create` creates on the server; `skd_fnc_place` sets a unit where it is local, and returns once the server sees it there. `skd_fnc_setDate` and `skd_fnc_setWeather` set the date and weather on the server and on every other machine, and return once every machine reads them.

## The reset across machines

Before every check, and on `shakedown reset`, every machine is brought back to the state of the mission's start, in four phases the runner drives on every machine at once: the previous check's cleanups run on the server; each client frees its player (alive again after a death, out of any vehicle); each machine deletes what was created on it since the start, restores its own player's loadout, health, position, direction and animation, and sets the server's start date and weather; then each machine checks the result, the server also every player's position as it sees it. On the test machine a reset after a check that changed everything takes about 0.75 s, and one with nothing to undo about 0.45 s.

A player killed in a check respawns at once and is back at its start position for the next check; the body is deleted, and the client keeps its start number 1.

```
shakedown reset
reset done in 0.579 s
```

`reset` is refused while a run is in progress and on a broken session; a reset that does not hold within 15 s exits 4 with what did not match, each item naming its machine, and leaves the session broken.
