---
type: Documentation
title: Installing Shakedown
description: What Shakedown needs on the machine, how to build and install it from this repository, what the install folder holds, and the files the runner keeps for itself.
status: draft
---

# Installing Shakedown

## What the machine needs

- 64-bit Windows 10 or 11 with Arma 3 installed by Steam, on the stable branch (2.22 or later).
- The Steam client running and logged in to an account that owns Arma 3. Without it the game exits at once and the session start fails.
- To build from source: the Rust toolchain (1.99 or later), HEMTT 1.23 and Python 3, each on `PATH`.

`list`, `status`, `guide`, `version` and `shakedown mcp` work on macOS and Linux with no game, so an AI agent on a Mac can read checks and the contract (`NFR-PT-02`); the MCP server answers `status`, `list_checks` and `guide` there. Starting a session needs Windows.

## When a session does not start

Before it starts any game process, `session start` checks that `arma3_x64.exe` (and for a dedicated setup `arma3server_x64.exe`) is in the `game` folder, that a `steam.exe` of the current user runs, that every mod folder of the setup exists, that the installed `@shakedown` holds its PBO and its extension, and, for a dedicated setup, that the five ports from `base_port` are free (`SES-03`). When one fails, the start is refused with exit code 4 (the tools: `precondition-failed`) and a message that names everything missing:

```
The session cannot start; missing: a running Steam client of this user (start Steam and log in). Provide what is missing and start the session again.
```

A game process that exits with code 53 during the start, which the game does when Steam is not running or not logged in, fails the start with `Steam is not running or not logged in (p1 exited with code 53)` (`INT-STM-01`).

A start can also fail on the machine itself: the game logs `Error 20 reading file` on one of its own base-game PBOs and ends with `Application terminated intentionally` a few seconds after it starts. The failed start's reason holds the last RPT lines that name the file. It is a fault of the machine, not of the framework or the mod, and a second start works.

## Build on macOS or Linux

A Mac or Linux machine needs only the `shakedown` binary: the addon and the fixtures load in a game it cannot start. It needs the Rust toolchain (1.99 or later) and neither HEMTT nor the game. From the repository root:

```
cargo build --release -p shakedown
```

The binary is `target/release/shakedown`. Put `target/release/` on `PATH` so the project's `.mcp.json` finds it ([Agent tools](agent-tools.md)), or copy the binary into a folder on `PATH`. `tools/install.py` is for Windows: it builds the addon and the fixtures with HEMTT and stops where HEMTT is missing.

## Build and install on Windows

From the repository root:

```
python tools/install.py
```

It builds the `shakedown` binary and the extension in release mode, builds the `@shakedown` addon and the two fixture mods with HEMTT, and assembles the install folder `target\shakedown\`. Give another folder as the first argument to install there:

```
python tools/install.py C:\Tools\Shakedown
```

The install folder holds:

```
shakedown.exe
@shakedown\addons\shakedown_main.pbo     the in-game agent and the check API
@shakedown\shakedown_x64.dll             the extension
fixtures\@skd_fixture\                  the fixture mod, which needs nothing but the game, with its
                                        extension skdfx_x64.dll, which ends a game process on demand
fixtures\@skd_fixture_cba\              the fixture mod with a CBA keybind
fixtures\skdfx\, fixtures\skdfx_cba\    the fixtures' unpacked files
```

The fixtures are small mods the framework's own tests run against ([spec/contract/fixture-mod.md](../spec/contract/fixture-mod.md)); a project loads them only when a setup names them. On Windows the install also links three folders in the game folder to the installed fixtures: `skdfx`, `skdfx_cba` and `@skd_fixture`. A link left by an earlier install is replaced; a real folder of one of those names stops the install. The game folder is Steam's default unless the `SHAKEDOWN_GAME` variable or `--game <folder>` names another:

```
python tools/install.py --game D:\Games\Arma3
```

`shakedown.exe` finds `@shakedown` next to itself, so keep them together. The first dedicated session links `<game>\@shakedown` to that installed `@shakedown`, and every later dedicated start points the link at the `@shakedown` of the `shakedown.exe` that starts it. Put the folder on `PATH` to call `shakedown` from any project, or call it by its full path.

Install again while no runner runs: Windows keeps the binary of a running runner locked. A runner exits by itself 60 s after its last session stopped.

## The runner and its files

Every command except `list`, `guide`, `version` and `mcp` talks to the runner, a background process the first such command starts and that outlives it. `status` asks a runner that runs and starts none: with no runner it answers `not-running`. The runner owns the session's game processes: when it ends, however it ends, they end with it. [When a client or the runner ends](sessions-and-the-runner.md) says what that means for a session. It keeps its own files in `%LOCALAPPDATA%\Shakedown\`:

| File | Holds |
|---|---|
| `runner.json` | The running runner's port, token, process id and start time, readable by the current user alone. Deleted when the runner exits. |
| `record.json` | The run record: every session the runner started, with its game processes and its mission copy, so that the next runner clears what a runner that died left ([When a client or the runner ends](sessions-and-the-runner.md)). |
| `runner.log` | One line per thing the runner did: how a client started it, started, what its start-up recovery cleared, a process started or stopped, a connection refused, a run written, and a panic or console event that ended it. |

When a command says the framework failed (exit code 5), `runner.log` says what happened.

A project gets one folder, `.shakedown\`, with a `.gitignore` that ignores everything in it; sessions and runs are written there and nowhere else in the project.
