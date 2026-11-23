---
type: Documentation
title: Quickstart
description: From nothing to a first passing check in singleplayer and on a dedicated server - install Shakedown, write a project file and a check, run it, keep a session up, and read the result and the exit code.
status: draft
---

# Quickstart

This page takes you from nothing to a check that passes in singleplayer and on a dedicated server with two clients. It takes about ten minutes, most of it the game starting. Every step links to the page that has the whole of it.

## What you need

- 64-bit Windows 10 or 11.
- Arma 3 2.22 or later, installed by Steam.
- The Steam client running and logged in to an account that owns Arma 3. Without it the game exits at once.
- 2 GB free on the drive that holds your project.

## 1. Install

In a terminal:

```
winget install casvanluijtelaar.Shakedown
```

or, in PowerShell, with the one-line script:

```
irm https://github.com/casvanluijtelaar/shakedown/releases/latest/download/install.ps1 | iex
```

or unpack `shakedown-windows-x64.zip` from the [latest release](https://github.com/casvanluijtelaar/shakedown/releases/latest) into a folder and put that folder on your `PATH`. Open a new terminal and check:

```
> shakedown version
shakedown 0.2.0
```

[Installing, upgrading and removing](install.md) has the details of each way.

## 2. A project file

A project is a folder that holds a file named `shakedown.toml`. Put it in your mod's repository, at the root or in a folder of its own:

```toml
game = 'C:\Program Files (x86)\Steam\steamapps\common\Arma 3'
tests = "tests"

[mods.mymod]
path = 'C:\path\to\mymod\.hemttout\build'
patch = "mymod_main"

[setups.sp]
mode = "singleplayer"
mods = ["mymod"]

[setups.dedicated]
mode = "dedicated"
clients = 2
mods = ["mymod"]
```

- `game` is the Arma 3 folder, the one that holds `arma3_x64.exe`.
- `[mods.mymod]` declares a mod: `path` is the folder the game loads, such as HEMTT's build output, and `patch` a `CfgPatches` class of the mod, whose version the result file records.
- Each `[setups.<name>]` is one way to run the checks: a mode and the mods it loads. `sp` is singleplayer; `dedicated` is a dedicated server with two game clients, `p1` and `p2`.
- Single quotes keep the backslashes of Windows paths as they are.

No mod yet? Leave out the `[mods.mymod]` table and write `mods = []` in both setups. The check below needs nothing but the game.

For a CBA-based mod, declare CBA too and list it first; [The project file](project-file.md) shows that and every other key.

## 3. A first check

Save this as `tests\smoke.sqf` in the project folder:

```sqf
// @check SMOKE-01 A crate placed in front of p1 reaches every client
// @tags smoke

private _crate = ["build the scene", {
    ["p1", AGLToASL [100, 100, 0], 0] call skd_fnc_place;
    ["Land_WoodenBox_F", AGLToASL [100, 105, 0], 0] call skd_fnc_create
}] call skd_fnc_step;

["every client sees the crate", {
    ["clients", { !isNull (_this select 0) }, "==", true, "the crate exists", 10, [_this select 0]] call skd_fnc_expectEach;
}, [_crate]] call skd_fnc_step;

["p1", { damage player }, "<", 0.1, "p1 is unhurt"] call skd_fnc_expectOn;
```

- **The header** is the comment lines at the top. `@check` is required: an ID for what the check proves, such as a story or a requirement, then a short sentence. `@tags` is optional.
- **The body** is plain SQF. It runs on the lead machine: `p1` in singleplayer, the server on a dedicated session.
- **Steps** name each phase, so a failure says where it happened. A step's code sees the body's local variables.
- `skd_fnc_place` puts `p1` at a spot, `skd_fnc_create` makes the crate on the server, and `skd_fnc_expectEach` reads a value on every client, every frame, until it holds on all of them or 10 s pass.
- **Code sent to another machine** sees only what you pass it, as `_this`: here the crate travels in the last argument.

The check's id is its path under `tests` without `.sqf`: `smoke`. See [Writing checks](writing-checks.md) for the whole file.

## 4. List it, without the game

```
> shakedown list
smoke  SMOKE-01 A crate placed in front of p1 reaches every client  [smoke]
```

`list` reads every header without starting anything and names any mistake in one, such as a missing `@check`.

## 5. Run it in singleplayer

```
> shakedown run --setup sp
session s-20261105-161040: started p1 (process 3604)
session s-20261105-161040: ready
pass    smoke  (0.0 s)
1 passed, 0 failed, 0 invalid, 0 not run. Result: C:\mods\mymod\.shakedown\runs\r-20261105-161040-1\result.json
```

With no session running, `run --setup` starts the game in that setup, runs every check, and stops the game again, whatever the outcome. It prints a line for each game process it starts and one when the game is ready, then a line per check and the summary. Starting the game takes its usual time, about 40 to 60 s. A game window opens; leave it alone while the check runs.

The exit code says how it went: in PowerShell, `$LASTEXITCODE` is `0`.

## 6. Keep the game running between runs

Starting the game for every run is slow. Start a **session** once and run against it as often as you like:

```
> shakedown session start --setup sp
session s-20261105-161208: started p1 (process 3604)
session s-20261105-161208: ready
session s-20261105-161208 ready (setup sp)
> shakedown run smoke
pass    smoke  (0.0 s)
1 passed, 0 failed, 0 invalid, 0 not run. Result: C:\mods\mymod\.shakedown\runs\r-20261105-161317-1\result.json
> shakedown status
runner: running (process 9184)
session: s-20261105-161208 of setup sp, ready
  p1      client    ready      start 1 process 3604 profile C:\mods\mymod\.shakedown\sessions\s-20261105-161208\machines\p1-1
          health: 149.9 frames a second, answers in 0.008 s, mission time 2.1 s, frame 1278
> shakedown session stop
session s-20261105-161208 stopped
```

Before every check the session is reset to the state its mission started in, in under a second, so checks do not see each other's leftovers. Edit the check and run it again: it is read from disk every time. See [Sessions and the runner](sessions-and-the-runner.md).

## 7. Run it on a dedicated server

```
> shakedown run --setup dedicated
session s-20261105-161950: started server (process 3116)
session s-20261105-161950: started p1 (process 5208)
session s-20261105-161950: started p2 (process 7484)
session s-20261105-161950: ready
pass    smoke  (0.4 s)
1 passed, 0 failed, 0 invalid, 0 not run. Result: C:\mods\mymod\.shakedown\runs\r-20261105-162020-2\result.json
```

This starts a dedicated server and the game clients `p1` and `p2`, all on your machine, runs the check and stops them all. The same check runs in both setups without a change. The server needs the game ports 2402 to 2406 free, and Windows Firewall may ask about the game's own executables the first time. See [Dedicated servers, clients and headless clients](dedicated-sessions.md).

## 8. Read the result

Every run writes one folder, `.shakedown\runs\<run id>\` in the project:

```
result.json               the result file
logs\<machine>.log        each machine's log lines for the whole run
checks\<n>\<machine>.log  each machine's log lines for the run's nth check
```

`result.json` holds the run, the session with each machine's game and mod versions, a summary, and one entry per check with its outcome, its reason, its steps with their times, the script errors caught and the evidence. A trimmed check entry:

```json
{
  "check_id": "smoke", "name": "SMOKE-01 A crate placed in front of p1 reaches every client",
  "tags": ["smoke"], "repeat": 1, "outcome": "pass", "reason": null,
  "steps": [
    { "path": ["build the scene"], "machine": "p1", "state": "passed", "duration_s": 0.1, "error": null, "slow": false },
    { "path": ["every client sees the crate"], "machine": "p1", "state": "passed", "duration_s": 0.0, "error": null, "slow": false }
  ],
  "errors": [],
  "evidence": { "failed_step": null, "observed": null, "attached": [], "logs": { "p1": "checks/1/p1.log" } }
}
```

Git ignores everything in `.shakedown\`. See [Results and the result file](results.md).

| Exit code | Means |
|---|---|
| `0` | Every check that ran passed. |
| `1` | A check failed. |
| `2` | None failed, but one was `invalid`, a check could not run because its session did not start or broke, or the run was stopped. |
| `3` | The request is wrong, such as an unknown check or a mistake in `shakedown.toml`. |
| `4` | The request cannot be done now, such as no session, Steam not running or a port taken. |
| `5` | Shakedown itself failed; the message names its log. |

## 9. See a failure

Change the last line of the check to expect damage above 0.5 and run it again in singleplayer. No session runs since step 6 stopped it, so the command names the setup:

```sqf
["p1", { damage player }, ">", 0.5, "p1 is hurt", 3] call skd_fnc_expectOn;
```

```
> shakedown run --setup sp smoke
session s-20261105-162140: started p1 (process 5120)
session s-20261105-162140: ready
fail    smoke  (3.0 s)
        assertion: p1 is hurt: expected > 0.5 on p1 within 3 s, last read 0.
0 passed, 1 failed, 0 invalid, 0 not run. Result: ...
```

In a session started as in step 6, `shakedown run smoke` gives the same failure. The exit code is `1`, and the result file names the failed step, the last value each machine read and each machine's log lines for the check. A check ends with one of three outcomes:

- `pass`: the body ran to its end, no assertion failed and no machine raised a script error.
- `fail`: the mod did not do what the check asserts, or the check itself is wrong.
- `invalid`: something outside the mod stopped the check, such as a crashed game; run it again.

See [Outcomes and a broken session](outcomes-and-broken-sessions.md).

## Where next

- [Writing checks](writing-checks.md) and [The check API](check-api.md) for everything a check can do.
- [The project file](project-file.md) for CBA, ACE, versions and the other keys.
- [Reloading changed mod files](reloading.md) to get a changed mod file into a running session.
- [AI agents and the MCP tools](agent-tools.md) to let an AI agent run and write checks.
