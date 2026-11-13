# Shakedown

Shakedown is Playwright for Arma 3: a test automation framework for Arma 3 addons and mods. You write checks in SQF, and Shakedown runs them inside a live game, in singleplayer or on a dedicated server with game clients and headless clients, all on one Windows machine.

- **One session, many checks.** The game starts once. Before every check the session is reset in place, in under a second, to the state its mission started in.
- **Every machine by name.** A check builds a scene, makes a player act through the mod's own actions and CBA keybinds, waits for the outcome and asserts it on the server, `p1`, `p2` or every machine at once.
- **A verdict that explains itself.** Every check ends `pass`, `fail` or `invalid`, with a reason. `fail` is the mod's or the check's fault. `invalid` means the environment failed, such as a crashed client. Script errors are traced to their machine and step. Each machine's log lines are kept with the check, and every run writes one result file.
- **An AI agent can drive it.** `shakedown mcp` gives an agent such as Claude Code every command as a tool, with the same JSON as the command line.

> **Release state.** Shakedown 0.1.0 is in development: phase 1, the MVP. The repository is private, and nothing is published yet: there is no release download, no package and no public wiki. You install it by building it from this repository.

## What you need

- 64-bit Windows 10 or 11, with Arma 3 2.22 or later on the stable branch, installed by Steam.
- The Steam client running and logged in to an account that owns Arma 3. Without it the game exits at once.
- To build: the Rust toolchain 1.99 or later, HEMTT 1.23 and Python 3, each on `PATH`.
- CBA_A3 only when your mod needs it, or when a check presses a CBA keybind with `skd_fnc_pressKey`. A check that presses no key needs no CBA. Shakedown is built against CBA_A3 3.19.0. ACE3 is optional: with ACE medical loaded, the reset gives each player ACE's full heal, checked against ACE3 3.21.2.

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
checks = "checks"

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

### 3. A first check

Save this as `checks\smoke.sqf`:

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

The header names what the check proves. The body runs on the lead machine: `p1` in singleplayer, the server on a dedicated session. Code given to another machine sees only what it is passed, as `_this`. `shakedown list` reads every check header without the game and reports any mistake in one.

### 4. Run it in singleplayer

```
shakedown run --setup sp
```

With no session running, `run --setup` starts a session of that setup, runs every check and stops the session. To keep a session up for many runs:

```
shakedown session start --setup sp
shakedown run smoke
shakedown session stop
```

### 5. Run it on a dedicated server with two clients

```
shakedown run --setup dedicated
```

This starts a dedicated server and the game clients `p1` and `p2` on the one machine, runs the check, and stops them all. The same check runs in both setups without a change.

### 6. Read the result

`run` prints one line per check, then a summary with the path of the result file:

```
pass    smoke  (0.0 s)
1 passed, 0 failed, 0 invalid, 0 not run. Result: C:\...\.shakedown\runs\r-20261105-161317-1\result.json
```

The exit code is 0 when every check passed, 1 when a check failed, and 2 when none failed and one was `invalid` or the run was stopped. Each run folder under `.shakedown\runs\` holds `result.json`, with each check's outcome, reason, steps, script errors and evidence, and each machine's log lines for the run and for each check. Add `--json` to any command for one JSON document on stdout. Git ignores everything in `.shakedown\`.

## Learn more

- **The wiki** covers every feature for mod authors: the project file, setups, the whole check API, running checks, results, reloading, the agent tools and troubleshooting.
- **[docs/](docs/index.md)** says how the built framework works, page by page.
- **[spec/contract/](spec/contract/index.md)** is the public interface: the check API, the command line, the agent tools and the result file.

## How it's built

Shakedown is built with agentic engineering, following the Eidra agentic engineering guide. The specification, the user stories and the contract in this repository are the source. Language models write the code, the tests and the documentation against them, milestone by milestone. A framework developer builds each milestone. A test developer then checks the result against the stories without reading the code, and writes findings for the developer to fix. The work is MVP first and local first: everything that starts the game runs on a Windows test machine, and GitHub Actions runs only the gates that need no game.

| Folder | What it holds |
|---|---|
| `research/` | What is known about the domain, each claim traced to a source. |
| `spec/` | The client requirements, the specification, the contract, the roles and the user stories. |
| `plan/` | The implementation plan, one concept per milestone, and deferred work. |
| `docs/` | How the built framework works, and a report per finished milestone. |
| `acceptance/` | The acceptance suite, its findings and its coverage, owned by the test developer. |
| `crates/`, `addon/`, `fixtures/`, `e2e/` | The runner and command line, the extension, the in-game addon, the fixture mods and the end-to-end projects. |
| `coordinator/` | The playbook a coordinator session loads to drive the loop. |
| `tools/` | The install script and the OKF bundle check. |

Every markdown document in the bundles is an Open Knowledge Format (OKF v0.2) concept, so `python3 tools/okf_check.py` can read the state of the project from the files. It needs Python 3.9+ and PyYAML, and CI runs it on every push. [CLAUDE.md](CLAUDE.md) is the map for agent sessions and holds the working rules. A coordinator session can drive the loop. Start it with:

> You are the coordinator for this repository. Read CLAUDE.md and coordinator/playbook.md in full, and follow the playbook.
