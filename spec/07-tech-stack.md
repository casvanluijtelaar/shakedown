---
type: Specification
title: Tech stack
description: The engineering principles behind Shakedown, what it is built from with the version of each component, how each choice serves each principle, the alternatives rejected, the source code folders and the contract files.
status: draft
---

# Tech stack

## Engineering principles

Each principle has an ID of the form `EP-nn`. A change to the stack names the principles it serves and the ones it costs.

- **EP-01** Documented interfaces only. The framework talks to the game through SQF, `callExtension` and its callback, start options and server configuration, and nothing else: no hooks, no patches, no memory reads (`NFR-LG-01`).
- **EP-02** Thin native code, logic where it reloads. The one native component inside the game, the extension, cannot be replaced while the game runs. It only moves bytes. Every behaviour lives in the runner, which restarts in a second, or in SQF, which reloads with the mission (`NFR-MT-01`).
- **EP-03** One toolchain outside the game. Everything that runs outside the game is one language and one build, so the runner, the command line, the MCP server and the extension share their message types and their tests (gap H11, Q4).
- **EP-04** The test author writes only SQF. Nothing a test author touches needs another language, a build step or a macro header (`CR-50`).
- **EP-05** Local and private by default. Every socket is on loopback with a token; the agent-facing server is stdio; nothing listens for the network (`CR-60`).
- **EP-06** The outer runner owns the verdict. Processes, ceilings and `invalid` belong to a process outside the game that keeps working when a game stops answering; steps, waits and assertions belong to the SQF inside it, which sees the game (the Unreal Gauntlet split; research `18-test-tools.md`).
- **EP-07** One result, every front end. The command line and the MCP server call the same runner requests and return the same JSON, defined once (`NFR-US-03`).
- **EP-08** Few dependencies, each looked up. A dependency earns its place by doing what cannot be written in an afternoon; its version is looked up when it is added or changed (CLAUDE.md).
- **EP-09** Testable without the game. Every part has tests that run without Arma 3, on GitHub Actions; only end-to-end tests need the test machine (CLAUDE.md, "Local first").

## The stack

Versions were looked up on 2026-11-04 from crates.io, GitHub releases, `static.rust-lang.org` and the game's Steam news. They are the versions the first milestone starts from; each is looked up again when it is installed.

| Component | Version | Used for | Source of the version |
|---|---|---|---|
| Arma 3 | 2.22, stable branch, with the mini-update of 2026-09-30 (SPOTREP #00122) | The game under test (`INT-AR-01`). | Steam news for app 107410 |
| CBA_A3 | 3.19.0 (2026-08-11) | Keybinds, when a mod uses CBA (`INT-CBA-01`). | GitHub release |
| ACE3 | 3.21.2 (2026-08-18) | Full heal at reset, when ACE medical is loaded (`INT-ACE-02`). | GitHub release |
| HEMTT | 1.23.1 (2026-10-25) | Building the framework's addon; the mod under test's dev build (`INT-HEM-01`). | GitHub release |
| Rust | 1.99.1 stable (2026-10-15), edition 2024 | The runner, command line, MCP server and extension. | `channel-rust-stable.toml` |
| `arma-rs` | 1.13.0, taken from crates.io under its declared MIT license (its repository's LICENSE file says GPL-2.0; research `20-licenses-and-rules.md`) | The extension's entry points, callback and retry. | crates.io |
| `rmcp` | 3.8.0 | The MCP server over stdio; protocol 2026-07-28 with negotiation of earlier revisions (`INT-MCP-01`). | crates.io, GitHub release |
| `tokio` | 1.54.1 | The runner's sockets, timers and process waits. | crates.io |
| `serde`, `serde_json` | 1.0.229, 1.0.154 | Every message, the result file, the runner files. | crates.io |
| `schemars` | 1.2.3 | Tool `outputSchema`s generated from the same types as the result file. | crates.io |
| `clap` | 4.6.7 | The command line. | crates.io |
| `toml` | 1.2.0 | Reading `shakedown.toml`. | crates.io |
| `windows` | 0.62.2 | Job objects, process creation and WMI on Windows. | crates.io |
| `getrandom` | 0.4.3 | Tokens and passwords. | crates.io |
| `regex` | 1.13.1 | The `logs` filter and the RPT error scan. | crates.io |
| `jsonschema` | 0.60.2, tests only | Validating every result file the tests produce against `result.schema.json`. | crates.io |
| SQF | the game's | The in-game agent and the check API. | - |
| Python 3 and PyYAML 6.0.3 | as in CI | The bundle check (`tools/okf_check.py`). | `.github/workflows/ci.yml` |

What is written in the project instead of taken as a dependency: the FNV-1a hash for file snapshots, the redaction of Steam IDs, the `-autotest` and `mission.sqm` generators, the newline-delimited JSON framing between the runner and the extension.

### Shape

- **One binary, `shakedown.exe`**: `shakedown runner` (the long-lived runner), `shakedown mcp` (the stdio MCP server), and every other subcommand as a thin client that talks to the runner over loopback. One binary means one version: a client refuses a runner of another version and tells the user to stop it.
- **The extension, `shakedown_x64.dll`**: a `cdylib` with `arma-rs`, the three calls of `INT-AR-03`, and a thread holding the connection to the runner.
- **The addon, `@shakedown`**: one PBO, `shakedown_main.pbo`, with the in-game agent and the check API as `CfgFunctions` functions with the tag `skd` (`skd_fnc_*`), compiled without `compileFinal` so they can be recompiled in a running mission. The agent starts in `postInit`.
- **Install layout**: one folder holding `shakedown.exe`, `@shakedown\` (with `addons\shakedown_main.pbo` and `shakedown_x64.dll`) and `fixtures\` (an installed copy of the fixture mods, [contract/fixture-mod.md](contract/fixture-mod.md)). The binary finds `@shakedown` next to itself. The project's `.mcp.json` calls `shakedown`, so the folder is on `PATH`.

## How each choice serves each principle

`+` serves it, `-` costs it, blank is neutral.

| Choice | EP-01 | EP-02 | EP-03 | EP-04 | EP-05 | EP-06 | EP-07 | EP-08 | EP-09 |
|---|---|---|---|---|---|---|---|---|---|
| Extension with `callExtension` and `ExtensionCallback` | + | + | | | + | | | | |
| Extension in Rust with `arma-rs` | + | + | + | | | | | + | + |
| Runner, CLI and MCP server in Rust, one binary | | + | + | | | + | + | | + |
| Runner as a detached, long-lived process with a job object | + | | | | | + | + | | |
| In-game agent and check API in SQF, plain functions | + | + | | + | | + | | | - |
| Check header in comments, parsed by the runner | | | | + | | | + | + | + |
| Orders between machines routed through the runner | + | + | | | + | + | | | + |
| Values as JSON through `toJSON`/`fromJSON` | + | | | | | | + | + | + |
| MCP over stdio with `rmcp` | | | + | | + | | + | + | |
| Loopback, per-session token in the environment | + | | | | + | | | + | |
| `schemars` types shared by tools and result file | | | + | | | | + | | + |
| TOML project file | | | + | | | | | + | + |

The one cost: SQF cannot be unit-tested outside the game (EP-09). The in-game agent is kept small, and the end-to-end tests on the test machine cover it.

## Alternatives rejected

| Alternative | For | Rejected because |
|---|---|---|
| Intercept (C++ host over an extension) | The channel | It is patched per game build and bypasses SQF the framework needs to run (EP-01, EP-02; research `12-remote-execution.md`). |
| The named pipe (`-command=`), `serverCommand`, the debug console, files read with file patching | The channel | None carries arbitrary SQF and a result back: five fixed commands, server commands, a UI for a person, a one-way read (research `12-remote-execution.md`). |
| C or C++ with the engine's header for the extension | The extension | It means writing the callback retry, typed calls and the test harness `arma-rs` has, and splits the toolchain (EP-03, EP-08). |
| C# with an export tool for the extension | The extension | A second runtime inside the game process and a third language (EP-03). |
| Logic in the extension | The extension | `freeExtension` works only on the development branch, so every change would be a game restart (EP-02). |
| Python, Node or PowerShell for the runner | The runner | A second toolchain beside the extension's Rust, and weaker control of Windows job objects and process creation (EP-03; gap H11). |
| Go for the runner | The runner | A second toolchain beside the extension's Rust (EP-03). |
| The MCP server owning the game processes | The runner | Claude Code ends a stdio server's process tree when its session stops, which would end the game session (EP-06; `INT-CC-02`). |
| Streamable HTTP for the MCP server | Agent tools | A listening socket needs `Origin` checks and a bearer token; stdio confines access to the launching client and needs neither (EP-05). It can come later for a remote agent. |
| The TypeScript or Python MCP SDK | Agent tools | A second runtime on the test machine (EP-03). `rmcp` speaks 2026-07-28. |
| `remoteExec` as the only route between machines | Control | `CfgRemoteExec` in a mod or mission can block it, a result does not say which machines answered, and the runner could not see a machine that stopped answering (EP-06). It is `SPK-01`'s fallback for a machine kind whose callback does not fire. |
| The check API as CBA-style macros | Check API | A check sent at run time is compiled from a string with `compile`, which does not run the preprocessor, and a macro header ties checks to CBA (EP-04). |
| Checks in YAML (Maestro) or another language | Check API | The PRD fixes SQF (`CR-50`, EP-04). |
| Check metadata as an SQF call (`[name, tags, ...] call skd_fnc_check`) | Check API | The runner could not list checks without a running game, and an AI agent on the Mac could not either (`RUN-02`, EP-09). |
| Results as `diag_log` lines read from the RPT | Results | A client's last lines were lost when it was ended; the RPT is evidence, not a channel (research `10-prior-harness.md`). |
| `-autotest` lists of checks, a mission per check | Sessions | The list is fixed at start and a mission change under connected clients broke one time in two (research `14-sessions.md`). |
| JSON or YAML for the project file | Configuration | TOML takes comments and is the Rust toolchain's own format; JSON takes no comments, and YAML's implicit typing turns a mistyped value into a different value instead of an error (EP-08). |
| A copied game executable per client | Sessions | The EULA grants one copy (EP-01; research `20-licenses-and-rules.md`). |

## Source code folders

The framework developer owns these. The test developer opens none of them.

| Folder | Holds |
|---|---|
| `Cargo.toml` | The Rust workspace. |
| `crates/shakedown/` | The `shakedown` binary: runner, command line, MCP server; its unit tests and its integration tests that need no game (in `tests/`). |
| `crates/shakedown-ext/` | The extension `cdylib` and its tests with `arma-rs`'s test harness. |
| `crates/shakedown-protocol/` | The message types between runner and extension, and the result file types that generate the schemas. |
| `addon/` | The HEMTT project of the `@shakedown` addon: the in-game agent and the check API in SQF. |
| `fixtures/` | The source of the fixture mods `@skd_fixture` and `@skd_fixture_cba`, HEMTT projects whose behaviour is [contract/fixture-mod.md](contract/fixture-mod.md). |
| `e2e/` | The framework developer's end-to-end tests: a `shakedown.toml` and SQF checks against the fixture mods, run on the test machine. |

GitHub Actions runs, besides the bundle check: `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` on Windows and Linux, and `hemtt check` on the addon. Nothing there starts the game.

## Contract files

The contract is the framework's public interface. The test developer works from it, the docs and the stories, and never past it. A change to any of these is a deliberate edit to the contract file first.

| File | Defines |
|---|---|
| [contract/check-api.md](contract/check-api.md) | The check file format and every `skd_fnc_*` function a check may call. |
| [contract/cli.md](contract/cli.md) | The `shakedown` command line, `shakedown.toml`, the `--json` output and the exit codes. |
| [contract/mcp-tools.md](contract/mcp-tools.md) | The MCP server's tools, their inputs and outputs. |
| [contract/result-file.md](contract/result-file.md) | The result file and the run folder. |
| [contract/result.schema.json](contract/result.schema.json) | The JSON Schema (2020-12) of the result file. |
| [contract/fixture-mod.md](contract/fixture-mod.md) | What the fixture mods do, for checks written against them. |

## The acceptance harness

The test developer writes the acceptance checks in SQF against the check API, as a test author does, and runs them through the `shakedown` command line. Python 3, already required by the bundle check, drives the command line and the MCP server for what no SQF check can reach and validates every result file against [contract/result.schema.json](contract/result.schema.json). The harness shares no code with the framework's source, which keeps the tester in a different language from it.
