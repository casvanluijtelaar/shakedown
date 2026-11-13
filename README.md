<p align="center">
    <img src=".github/assets/banner.svg" alt="Shakedown: test automation framework for Arma 3 mods" width="100%">
</p>

<p align="center">
    <img src="https://img.shields.io/badge/Version-0.1.0-blue.svg?style=flat-square" alt="Version 0.1.0">
    <img src="https://img.shields.io/badge/Arma%203-2.22+-a3c644.svg?style=flat-square" alt="Arma 3 2.22 or later">
    <img src="https://img.shields.io/badge/Platform-Windows-lightgrey.svg?style=flat-square" alt="Windows">
    <img src="https://img.shields.io/badge/Built%20with-Rust%20%2B%20SQF-orange.svg?style=flat-square" alt="Built with Rust and SQF">
</p>

<p align="center">
    <sup><strong><a href="#quickstart">Quickstart</a> | <a href="https://github.com/casvanluijtelaar/shakedown/wiki">Documentation</a></strong></sup>
</p>

**Shakedown** is Playwright for Arma 3. You write tests in SQF, and Shakedown runs them inside a live game and tells you which passed.

## Features

- Tests in plain SQF, run inside a live game
- Singleplayer and dedicated server, with clients and headless clients
- A reset between tests in under a second, no restart
- Check the result on the server, on one client, or on every client at once
- Players act through the mod's own actions and CBA keybinds
- `pass`, `fail` or `invalid`, always with a reason
- Script errors traced to their machine and step
- One JSON result file per run
- Every command as an MCP tool for AI agents

## Requirements

- Windows 10 or 11, 64-bit
- Arma 3 2.22+ from Steam
- Steam running and logged in

## Quickstart

### 1. Install

From the root of this repository:

```
python tools/install.py
```

This builds `shakedown.exe`, the `@shakedown` addon and its extension into `target\shakedown\`. Put that folder on `PATH`. Keep `shakedown.exe` and `@shakedown` together. To install somewhere else, give the folder as the first argument: `python tools/install.py C:\Tools\Shakedown`.

### 2. A project file

A project is a folder that holds a `shakedown.toml`. Put it in your mod's repository:

```toml
game = 'C:\Program Files (x86)\Steam\steamapps\common\Arma 3'
checks = "tests"

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

`path` is the folder the game loads, and `patch` is a `CfgPatches` class of the mod, whose version the result file records. For a CBA-based mod, also declare `[mods.cba]` with the path of CBA's folder, and list it first in each setup's `mods`. Use single quotes so Windows paths keep their backslashes.

### 3. A first test

Save this as `tests\smoke.sqf`:

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

The header names what the test proves. The body runs on the lead machine: `p1` in singleplayer, the server on a dedicated session. Code given to another machine sees only what it is passed, as `_this`. `shakedown list` reads every test header without the game and reports any mistake in one.

### 4. Run it in singleplayer

```
shakedown run --setup sp
```

With no session running, `run --setup` starts a session of that setup, runs every test and stops the session. To keep a session up for many runs:

```
shakedown session start --setup sp
shakedown run smoke
shakedown session stop
```

### 5. Run it on a dedicated server with two clients

```
shakedown run --setup dedicated
```

This starts a dedicated server and the game clients `p1` and `p2` on the one machine, runs the test, and stops them all. The same test runs in both setups without a change.

### 6. Read the result

`run` prints one line per test, then a summary with the path of the result file:

```
pass    smoke  (0.0 s)
1 passed, 0 failed, 0 invalid, 0 not run. Result: C:\...\.shakedown\runs\r-20261105-161317-1\result.json
```

The exit code is 0 when every test passed, 1 when a test failed, and 2 when none failed and one was `invalid` or the run was stopped. Each run folder under `.shakedown\runs\` holds `result.json`, with each test's outcome, reason, steps, script errors and evidence, and each machine's log lines for the run and for each test. Add `--json` to any command for one JSON document on stdout. Git ignores everything in `.shakedown\`.


## AI agents

`shakedown mcp` is an MCP server that gives an AI agent such as Claude Code every command as a tool, returning the same JSON as the command line. Add a `.mcp.json` next to your `shakedown.toml`:

```json
{
  "mcpServers": {
    "shakedown": { "command": "shakedown", "args": ["mcp"], "timeout": 70000 }
  }
}
```

Start Claude Code in that folder, and the agent can:

- start and stop sessions, and run tests
- read results, script errors and each machine's logs
- run SQF on any machine and inspect it live
- reload changed mod files and rerun a test
- call `guide` for a short manual on writing and running tests
