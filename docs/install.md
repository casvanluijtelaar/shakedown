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

`list` and `version` work on any system with no game. Starting a session needs Windows.

## Build and install

From the repository root:

```
python tools/install.py
```

It builds the `shakedown` binary and the extension in release mode, builds the `@shakedown` addon with HEMTT, and assembles the install folder `target\shakedown\`. Give another folder as the first argument to install there:

```
python tools/install.py C:\Tools\Shakedown
```

The install folder holds:

```
shakedown.exe
@shakedown\addons\shakedown_main.pbo     the in-game agent and the check API
@shakedown\shakedown_x64.dll             the extension
```

`shakedown.exe` finds `@shakedown` next to itself, so keep them together. Put the folder on `PATH` to call `shakedown` from any project, or call it by its full path.

Install again while no runner runs: Windows keeps the binary of a running runner locked. A runner exits by itself 60 s after its last session stopped.

## The runner and its files

Every command except `list` and `version` talks to the runner, a background process the first such command starts and that outlives it. The runner owns the session's game processes: when it ends, however it ends, they end with it. It keeps its own files in `%LOCALAPPDATA%\Shakedown\`:

| File | Holds |
|---|---|
| `runner.json` | The running runner's port, token, process id and start time. Deleted when the runner exits. |
| `runner.log` | One line per thing the runner did: started, a process started or stopped, a connection refused, a run written. |

When a command says the framework failed (exit code 5), `runner.log` says what happened.

A project gets one folder, `.shakedown\`, with a `.gitignore` that ignores everything in it; sessions and runs are written there and nowhere else in the project.
