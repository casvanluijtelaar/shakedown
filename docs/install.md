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
fixtures\@skd_fixture\                  the fixture mod, which needs nothing but the game
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

Every command except `list`, `guide`, `version` and `mcp` talks to the runner, a background process the first such command starts and that outlives it. `status` asks a runner that runs and starts none: with no runner it answers `not-running`. The runner owns the session's game processes: when it ends, however it ends, they end with it. It keeps its own files in `%LOCALAPPDATA%\Shakedown\`:

| File | Holds |
|---|---|
| `runner.json` | The running runner's port, token, process id and start time. Deleted when the runner exits. |
| `runner.log` | One line per thing the runner did: started, a process started or stopped, a connection refused, a run written. |

When a command says the framework failed (exit code 5), `runner.log` says what happened.

A project gets one folder, `.shakedown\`, with a `.gitignore` that ignores everything in it; sessions and runs are written there and nowhere else in the project.
