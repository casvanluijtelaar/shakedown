---
type: Documentation
title: Installing, upgrading and removing
description: What Shakedown needs on the machine, how to install a release with winget, the one-line script or the zip by hand, how to upgrade and remove it, what a release holds and its licence, the files the runner keeps for itself, and how to build and install from source.
status: draft
---

# Installing, upgrading and removing

## What the machine needs

- 64-bit Windows 10 or 11 with Arma 3 installed by Steam, on the stable branch (2.22 or later).
- The Steam client running and logged in to an account that owns Arma 3. Without it the game exits at once and the session start fails.
- 2 GB free on the drive that holds the project's `.shakedown` folder; a session start checks it.
- A release needs nothing else: no Rust, HEMTT, Python or clone of the repository. Building from source needs the Rust toolchain (1.99 or later), HEMTT 1.23 and Python 3, each on `PATH` ([Build from source](#build-from-source)).

`list`, `status`, `guide`, `version` and `shakedown mcp` also work on macOS and Linux with no game, so an AI agent on a Mac can read checks; there the MCP server answers `status`, `list_checks` and `guide`. Starting a session needs Windows. A release is for 64-bit Windows only; on macOS and Linux, build the binary with cargo.

## Install a release

Each release on [GitHub](https://github.com/casvanluijtelaar/shakedown/releases) holds `shakedown-windows-x64.zip`, its SHA-256 in `shakedown-windows-x64.zip.sha256` and the install script `install.ps1`. Install it one of three ways. Each gives a folder on your `PATH` that holds `shakedown.exe` with `@shakedown\` next to it. Open a new terminal afterwards and check:

```
> shakedown version
shakedown 0.2.0
```

The binary is not signed, so Windows SmartScreen may warn the first time it runs, as it does for HEMTT.

### With winget

```
winget install casvanluijtelaar.Shakedown
```

winget unpacks the zip into its own package folder and puts that folder on your `PATH`. A pre-release, such as `0.2.0-rc.1`, is never on winget; install it with the script or by hand.

### With the one-line script

In PowerShell:

```
irm https://github.com/casvanluijtelaar/shakedown/releases/latest/download/install.ps1 | iex
```

It downloads the zip and its `.sha256`, refuses a zip whose SHA-256 differs, unpacks it into `%LOCALAPPDATA%\Programs\Shakedown`, and adds that folder to your user `PATH` once, with no administrator rights. Nothing changes until the download has been checked: a corrupt or cut-off download ends with an error that names both hashes and says nothing was installed.

`irm | iex` runs the script's text and is not stopped by PowerShell's execution policy. A downloaded copy of `install.ps1` run as a file is: under the policy `RemoteSigned`, unblock it first, since it comes from the internet and is not signed; under `Restricted`, Windows' own default, use the line above instead.

```
Unblock-File .\install.ps1
powershell -File .\install.ps1
```

The script reads three environment variables:

| Variable | What it does |
|---|---|
| `SHAKEDOWN_VERSION` | Installs that release instead of the latest, such as `$env:SHAKEDOWN_VERSION = '0.2.0-rc.1'`. |
| `SHAKEDOWN_DOWNLOAD` | Takes the zip and its `.sha256` from this URL or local folder instead of GitHub, for a mirror or a test. |
| `SHAKEDOWN_INSTALL_DIR` | Installs into this folder instead of `%LOCALAPPDATA%\Programs\Shakedown`. |

```powershell
$env:SHAKEDOWN_VERSION = '0.2.0-rc.1'
irm https://github.com/casvanluijtelaar/shakedown/releases/latest/download/install.ps1 | iex
```

The line always runs the `install.ps1` of the latest full release, which then installs the version you named. A refusal or a failed download ends the script with an error, exit code 1 when it runs as a file, and leaves the install folder and `PATH` as they were.

### By hand

Download `shakedown-windows-x64.zip` from the release, check it against the release's `.sha256` if you like (`Get-FileHash shakedown-windows-x64.zip`), and unpack it into any folder. The folder is a complete install: put it on your `PATH`, or call `shakedown.exe` by its full path.

## What a release holds

```
shakedown.exe
@shakedown\addons\shakedown_main.pbo     the in-game agent and the check API
@shakedown\shakedown_x64.dll             the extension
LICENSE                                  GPL-2.0-or-later
THIRD-PARTY-LICENSES.txt                 the licence and notices of every package compiled in
```

Keep `shakedown.exe` and `@shakedown\` together: the binary finds the addon next to itself. You never list `@shakedown` in your project file; every session loads it after your mods. A release holds no fixture mods; they are for the framework's own tests and come with an install from source.

Shakedown is licensed GPL-2.0-or-later. `shakedown.exe` includes `rmcp`, which is Apache-2.0 only, so that binary is distributed under the terms of GPL version 3. The licence covers Shakedown's code; what you may do with Arma 3 is still set by the game's EULA.

## Upgrade

Stop what runs from the install first: end each session with `shakedown session stop`, after which the runner exits by itself within 60 s, and end every AI agent session whose `.mcp.json` starts `shakedown mcp`. Windows keeps a running `shakedown.exe` locked, and a runner of one version refuses the clients of another.

- **winget**: `winget upgrade casvanluijtelaar.Shakedown`. With a `shakedown.exe` of the package still running, winget cannot replace the locked file and the upgrade fails; stop them and run it again.
- **The script**: run the same `irm ... | iex` line again. While any `shakedown.exe` of its folder runs, the runner or an agent's `shakedown mcp` among them, it refuses before it changes any file and names each such process by its id and command:

  ```
  Shakedown is running from C:\Users\<you>\AppData\Local\Programs\Shakedown, so nothing was changed. Stop these processes first, then run the install again:
    5880  C:\Users\<you>\AppData\Local\Programs\Shakedown\shakedown.exe runner
    2760  "C:\Users\<you>\AppData\Local\Programs\Shakedown\shakedown.exe" mcp
  ```

- **By hand**: unpack the new zip over the old folder, replacing its files.

If you upgraded while a runner of the old version still ran, every command says the runner runs another version and names its process; see [Sessions and the runner](sessions-and-the-runner.md#a-command-cannot-reach-the-runner).

## Remove

- **winget**: `winget uninstall casvanluijtelaar.Shakedown` removes `shakedown.exe`, `@shakedown\` and the `PATH` entry.
- **The script or by hand**: stop what runs from the folder as for an upgrade, delete the folder, and remove it from your user `PATH` (Settings, System, About, Advanced system settings, Environment Variables).

The runner's own files in `%LOCALAPPDATA%\Shakedown\` and each project's `.shakedown\` folder stay; delete them too if you want nothing left. A dedicated session leaves a link `<game>\@shakedown`, and a reload at level `mission-restart` leaves the folder `<game>\skd_release`; you can delete both.

## The runner and its files

Every command except `list`, `guide`, `version` and `mcp` talks to the **runner**, a background process the first such command starts and that outlives it. `status` asks a runner that runs and starts none: with no runner it answers `not-running`. The runner owns the session's game processes: when it ends, however it ends, they end with it. [Sessions and the runner](sessions-and-the-runner.md) says what that means for a session. It keeps its own files in `%LOCALAPPDATA%\Shakedown\`:

| File | Holds |
|---|---|
| `runner.json` | The running runner's port, token, process id and start time, readable by the current user alone. Deleted when the runner exits. Never delete it while a runner holds a session ([Troubleshooting](troubleshooting.md)). |
| `record.json` | The run record: every session the runner started, with its game processes and its mission copy, so that the next runner clears what a runner that died left. |
| `runner.log` | One line per thing the runner did: how a client started it, started, what its start-up recovery cleared, a process started or stopped, a connection refused, a run written, and a panic or console event that ended it. |

When a command says the framework failed (exit code 5), `runner.log` says what happened.

A project gets one folder, `.shakedown\`, with a `.gitignore` that ignores everything in it; sessions, runs and long tool results are written there and nowhere else in the project.

## Build from source

### On macOS or Linux

A Mac or Linux machine needs only the `shakedown` binary: the addon and the fixtures load in a game it cannot start. It needs the Rust toolchain (1.99 or later) and neither HEMTT nor the game. From the repository root:

```
cargo build --release -p shakedown
```

The binary is `target/release/shakedown`. Put `target/release/` on `PATH` so the project's `.mcp.json` finds it ([AI agents and the MCP tools](agent-tools.md)), or copy the binary into a folder on `PATH`. `tools/install.py` is for Windows: it builds the addon and the fixtures with HEMTT and stops where HEMTT is missing.

### On Windows

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

The fixtures are small mods the framework's own tests run against ([spec/contract/fixture-mod.md](../spec/contract/fixture-mod.md)); a project loads them only when a setup names them. The install also links three folders in the game folder to the installed fixtures: `skdfx`, `skdfx_cba` and `@skd_fixture`. A link left by an earlier install is replaced; a real folder of one of those names stops the install. The game folder is Steam's default unless the `SHAKEDOWN_GAME` variable or `--game <folder>` names another:

```
python tools/install.py --game D:\Games\Arma3
```

`shakedown.exe` finds `@shakedown` next to itself, so keep them together. The first dedicated session links `<game>\@shakedown` to that installed `@shakedown`, and every later dedicated start points the link at the `@shakedown` of the `shakedown.exe` that starts it. Put the folder on `PATH` to call `shakedown` from any project, or call it by its full path.

Install again while no runner runs: Windows keeps the binary of a running runner locked. A runner exits by itself 60 s after its last session stopped.

### A release zip

The release workflow builds each release with `tools/package.py`, which runs on any Windows machine with the build tools:

```
python tools/package.py --tag v<version>
```

It stops when the tag differs from the workspace version in `Cargo.toml`, builds the binary, the extension and the addon, writes `THIRD-PARTY-LICENSES.txt` from `Cargo.lock`, and writes the zip and its `.sha256` into `target\dist\`. Without `--tag` it skips the check. `install.ps1` installs from that folder with `$env:SHAKEDOWN_DOWNLOAD` set to it.
