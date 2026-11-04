---
type: Research
title: Existing Arma automation, MCP and debug tools
description: What each existing tool for driving, observing or debugging Arma 3 from outside does, how it talks to the game, how mature it is, its license, and what the framework takes from it.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T21:54:00+01:00 }
sources:
  - { id: neefay, resource: "https://github.com/Neefay/arma3-llm-harness at 21aac67e8184901e4016ed4effa06c99afd2e067 (README.md, docs/ARCHITECTURE.md, host/src/arma_harness/harness.py, host/src/arma_harness/serverroot.py)", read: 2026-11-04 }
  - { id: headswe, resource: "https://github.com/headswe/arma3-mcp at d778d75e87bf7590793297e1afb63558c8b88f14 (README.md, file tree)", read: 2026-11-04 }
  - { id: a3bridge, resource: "https://github.com/lepitinox/a3-bridge at cf42c73914052405238b5646d1449b406eef887d (README.md, LICENSE, ext/src/net.rs)", read: 2026-11-04 }
  - { id: zihuange, resource: "https://github.com/Zihuange/arma3-mcp at 380af427af8604f1d04996946c6e3ccfa325ed10 (README.md, LICENSE, NOTICE, docs/decisions.md D20, game/extension/src/arma3_mcp_extension.cpp)", read: 2026-11-04 }
  - { id: a3ref, resource: "https://github.com/a3ref/mcp at 3109b0ef97d2e282b142c3ab0e6fd811f897a93f (README.md, LICENSE)", read: 2026-11-04 }
  - { id: wikimcp, resource: "https://github.com/ScottTMConnors/arma3-wiki-mcp at 90418253ccd85a91d94c570f5525f4fe0d3c14f6 (README.md, LICENSE, NOTICE.md)", read: 2026-11-04 }
  - { id: nighthawk, resource: "https://github.com/Nighthawk42/arma3-mcp at 4c4bc777f8283228a6b40ef7659b790a681750c7 (README.md, LICENSE)", read: 2026-11-04 }
  - { id: intelpkg, resource: "https://github.com/habitualdev/IntelPackage at b365a2600b7b3508485a785a229e8304b08d3692 (README.md, LICENSE)", read: 2026-11-04 }
  - { id: jtm, resource: "https://github.com/JTM-rootstorm/arma-mcp at 1beb60613d06ec292ee0ce1e6aeae89f0f24943f (README.md, file tree, no LICENSE file)", read: 2026-11-04 }
  - { id: ade, resource: "https://github.com/dedmen/ArmaDebugEngine at e01f80e42665082bf209046dd97f87787d5c0547 (README.md, LICENSE, BIDebugEngine/BIDebugEngine/NamedPipeServer.cpp, EngineHook.cpp)", read: 2026-11-04 }
  - { id: vscodedbg, resource: "https://github.com/Vindicta-Team/vscode-sqf-debugger at 854c32f73c78517a7f5c3092cdaef2276c329d20 (readme.md, LICENSE)", read: 2026-11-04 }
  - { id: rce, resource: "https://github.com/realbadidas/Arma-Remote-Code-Executor at bb762ae426adad9a507b08ae919428b744feaa65 (README.md, LICENSE)", read: 2026-11-04 }
  - { id: armaws, resource: "https://github.com/g0mb4/arma-ws at 7958508bf2f210935626e75b061c2b8427998feb (README.md, no LICENSE file)", read: 2026-11-04 }
  - { id: intercept, resource: "https://github.com/intercept/intercept at 47ce89fc5fa78b3a04fb35d9672277c46f7787ea (README.md, LICENSE, commit history)", read: 2026-11-04 }
  - { id: armars, resource: "https://github.com/BrettMayson/arma-rs at 8e7b243ccf6fcf98fbe83a9dd86982336ac660d1 (README.md, LICENSE, arma-rs/Cargo.toml, arma-rs-proc/Cargo.toml)", read: 2026-11-04 }
  - { id: armarscrate, resource: "https://crates.io/api/v1/crates/arma-rs (version 1.13.0, license field)", read: 2026-11-04 }
  - { id: hemttdoc, resource: "https://hemtt.dev/commands/launch.html (HEMTT book, launch command)", read: 2026-11-04 }
  - { id: hemttsrc, resource: "https://github.com/BrettMayson/HEMTT at 903589ed4a26a9e9e4b8f05e4ce0a562f847bd5b (bin/src/commands/launch/launcher.rs, platforms.rs, license GPL-2.0, release v1.23.1)", read: 2026-11-04 }
  - { id: tesqf, resource: "https://github.com/10Dozen/ArmaTesqf branch dev at dacfff1c955b1869d0429cde3255cbfcf997a978 (README.md, TestCase.sqf, LICENSE)", read: 2026-11-04 }
  - { id: cbatest, resource: "https://github.com/CBATeam/CBA_A3 at 8b15a04bd52ff7acc568a93cf6e90343e43a8a56 (addons/common/test.sqf, addons/main/script_macros_common.hpp)", read: 2026-11-04 }
  - { id: pythia, resource: "https://github.com/overfl0/Pythia at b0b0061dd272b69f58d3876a539399fc1473d851 (repository metadata, license MIT)", read: 2026-11-04 }
  - { id: ghmeta, resource: "GitHub API repository metadata, commit counts and searches (gh api repos/<owner>/<repo>, gh search repos 'arma 3 mcp', 'arma mcp', 'sqf test', 'arma 3 testing')", read: 2026-11-04 }
  - { id: wikiext, resource: "https://community.bistudio.com/wiki/Extensions (revision 379582, 2026-11-03)", read: 2026-11-04 }
---

# Existing Arma automation, MCP and debug tools

This topic reads every known tool that drives, observes or debugs Arma 3 from outside, at a named commit, and says what the framework takes from it. A feature counts only where the tool's code or docs show it. A tool without a LICENSE file gives ideas, never code.

## Summary

Every tool that runs code in a live game does it the same way: a native extension, loaded with `callExtension`, owns a local socket, and an SQF addon in the game picks requests up and runs them with `compile` and `call`[^headswe][^neefay][^a3bridge][^zihuange]. Only two, `Neefay/arma3-llm-harness` and `headswe/arma3-mcp`, route code to a chosen machine of a multiplayer session[^neefay][^headswe], and only the first also starts and stops the session, attributes script errors to a request, streams events and runs declarative tests[^neefay]. It is the closest prior art to this framework, and it has no license file, so it gives ideas only[^neefay].

All the Arma MCP servers are new: created between 2026-06 and 2026-11, with 3 to 51 commits and 0 to 3 stars[^ghmeta]. None is a dependency to build on. The mature projects are libraries and build tools: Intercept (979 commits), arma-rs (210), HEMTT (1225)[^ghmeta].

## Comparison

Maturity is the date of the commit read, the commit count, and the stars, from the GitHub API on 2026-11-04[^ghmeta]. "Machines" says where code can be run: SP singleplayer, DS dedicated server, C client, HC headless client.

| Tool | Covers | Transport | Machines | Platform | Maturity | License | Worth taking |
|---|---|---|---|---|---|---|---|
| `Neefay/arma3-llm-harness` | CR-01, CR-04, CR-05, CR-10, CR-20, CR-21, CR-23, CR-24, CR-25, CR-27, CR-31, CR-53, CR-60 | MCP stdio and CLI → HTTP on loopback with a per-run token → Pythia (Python in the game) on the server → CBA events to other machines | DS, C, HC; SP and hosted only by attach | Windows, Arma 3 2.22+, CBA, Pythia, own mod | 2026-11-03, 7 commits, 0 stars | none | idea |
| `headswe/arma3-mcp` | CR-10, CR-11, CR-22, CR-24, CR-27, CR-53, CR-60 | MCP stdio or HTTP → TCP line protocol on 127.0.0.1:9120 → extension → `ExtensionCallback` → SQF; `remoteExecCall` to other machines | SP, DS, hosted, Eden; C and HC as targets | Windows and Linux `.so`, Arma 3 2.18+ | 2026-10-24, 13 commits, 1 star | none | idea |
| `lepitinox/a3-bridge` | CR-11, CR-21, CR-27, CR-53 | HTTP and WebSocket on 127.0.0.1:8765 inside a Rust extension; Node MCP server | SP, DS; client relays by `remoteExec` | Windows | 2026-10-27, 5 commits, 0 stars | MIT | idea, small code |
| `Zihuange/arma3-mcp` | CR-10, CR-15, CR-25, CR-27, CR-53 | MCP stdio or HTTP; the extension connects out to the MCP server over TCP 32132 | SP, DS (UI tools refused on DS) | Windows | 2026-10-23, 15 commits, 0 stars | MIT | idea |
| `JTM-rootstorm/arma-mcp` | CR-53 (Eden only) | stdio sidecar → HTTP on 127.0.0.1 with bearer token → extension | Eden editor | Windows, Linux/Proton | 2026-06-22, 51 commits, 0 stars | none | nothing |
| `a3ref/mcp` | CR-53 (reference and SQF lint, no game) | MCP stdio | none | Node 18+ | 2026-11-04, 19 commits, 1 star | MIT (code; wiki content excluded) | idea (lint for test authors) |
| `ScottTMConnors/arma3-wiki-mcp` | reference only | MCP stdio | none | Python 3.10+ | 2026-11-01, 3 commits, 3 stars | MIT (code only; wiki data non-commercial) | nothing |
| `Nighthawk42/arma3-mcp` | reference only | MCP stdio | none | Node | 2026-10-25, 11 commits, 1 star | MIT (code only) | nothing |
| `habitualdev/IntelPackage` | reference, SQF lint, PBO index | MCP stdio | none | Go 1.26.2+ | 2026-08-18, 6 commits, 1 star | MIT | nothing now |
| `dedmen/ArmaDebugEngine` | CR-27 (breakpoints, variables) | named pipe `\\.\pipe\ArmaDebugEnginePipeIface`, engine hooks | SP, Eden (only path tested by its VS Code client) | Windows, Intercept | 2025-02-01, 132 commits, 49 stars | own Arma-only, no adaptation | nothing |
| `Vindicta-Team/vscode-sqf-debugger` | CR-27 (VS Code front end for ADE) | ADE pipe | Eden | VS Code | 2020-03-14, 265 commits, 5 stars | MIT | nothing |
| `realbadidas/Arma-Remote-Code-Executor` | CR-10 on a server | HTTP(S) POST with user and password | DS | Windows only | 2024-02-07, 21 commits, 9 stars | GPL-3.0 | nothing |
| `g0mb4/arma-ws` | CR-10 | WebSocket in an extension | where loaded | Windows, Linux | 2020-12-07, 31 commits, 3 stars | none (README grant only) | nothing |
| Intercept | none directly (native SQF bindings) | in-process C++ host and client DLLs | where loaded | Windows, Linux | 2025-08-08, 979 commits, 248 stars | MIT | nothing for MVP |
| arma-rs | CR-10 building block (extension in Rust) | `callExtension`, callbacks, call context | where loaded | Windows, Linux | 2026-08-24, 210 commits, 50 stars | crate MIT, repo LICENSE GPL-2.0 | library, owner to confirm |
| HEMTT `launch` | CR-01 (clients only), CR-03 (file patching build) | starts `arma3_x64` | SP, clients | Windows, Linux, macOS | v1.23.1, 1225 commits, 159 stars | GPL-2.0 | tool, not code |
| `10Dozen/ArmaTesqf` | CR-31, CR-40, CR-50 (in-mission test runner) | none; runs inside the mission | SP | CBA | 2021-05-19, 1 commit on main, code on `dev` | MIT | idea |
| CBA test scripts | CR-50 vocabulary | none; results to RPT | where run | CBA | live, part of CBA | GPL-2.0 | idea |

## Tools that run code in a live game

### `Neefay/arma3-llm-harness`

What it does: an MCP server and a CLI that boot a mission on an isolated local dedicated server with headless clients and an optional rendering client, or attach to a game the user started; run SQF on any machine; stream events; take screenshots; hot-reload mission files; and run YAML scenarios that return pass or fail[^neefay].

How it talks to the game:
- The host posts to an HTTP endpoint on 127.0.0.1 with an ephemeral port and a per-run token checked in constant time. The endpoint lives in Pythia, which runs Python inside the hub's game process[^neefay]. Pythia is MIT[^pythia].
- The SQF probe polls the bridge each server frame, resolves the target, and sends remote work with `CBA_fnc_ownerEvent`; results come back with a CBA server event[^neefay].
- Targets are `server`, `clients`, `hcs`, `all`, `interface`, `owner:N` and `player:<name>`[^neefay].
- Code runs unscheduled by default. The docs give a round trip of 1 to 2 server frames plus HTTP, about 20 to 60 ms locally, as their own measurement on 2.22[^neefay].
- A Rust extension hooks the Direct3D Present call to capture frames, because `RVExtensionGGetR2TTexture` returned nothing for scripted render-to-texture in its tests[^neefay].

How it handles the hard parts:
- **Errors.** A `ScriptError` mission event handler (2.14+) attributes each error to the request running now, or to a `private _brmhReq = "<id>";` tag compiled into every request, which shows up in the handler's content for code that request spawned. A request still marked current on the next frame is reported `aborted`[^neefay].
- **Events.** JSON envelopes carry machine, owner, `CBA_missionTime`, `diag_tickTime` and frame. The bridge keeps 50,000 events and long-polls with a cursor[^neefay].
- **Lifecycle.** The probe activates only for a game process whose PID has a session file the harness wrote. A mission restart begins a new epoch and fails pending requests with "mission restarted"[^neefay].
- **Isolation.** The server runs from its own root: a copy of `arma3server_x64.exe` and the Steam DLLs, with directory junctions to the install's data folders. Missions run as staged copies, because the engine locks every file it reads[^neefay].
- **Server config.** `loopback = 1`, `BattlEye = 0`, `kickDuplicate = 0`. The docs say `-ip=127.0.0.1` alone still registers with the Steam master list[^neefay].
- **Clients.** Headless clients are `arma3server_x64 -client`, and need a `HeadlessClient_F` slot or they hang silently. Rendering clients are `arma3_x64 -connect=127.0.0.1 ... -window -noPause -world=empty` with their own `-profiles` and `-name`[^neefay].
- **Process lifetime.** It spawns the game through WMI so the game outlives the MCP server, because MCP clients run servers in a kill-on-close job object[^neefay].

Machines: a dedicated server with clients and headless clients when it launches the run. Singleplayer, Eden and hosted games only by attaching to a game the user started[^neefay].

Platform: Windows 10/11, Arma 3 2.22+, CBA_A3 and Pythia from the Workshop, Rust, Python 3.11+, HEMTT 1.23+[^neefay].

License: the repository has no LICENSE file, and neither `host/pyproject.toml` nor `brmh_ext/Cargo.toml` declares one[^neefay]. No code may be taken.

Worth taking, as ideas:
- the per-PID session file that makes the probe inert unless the runner launched that process;
- error attribution by a request tag in compiled code;
- the epoch per mission start;
- the event envelope with both clocks;
- staged mission copies;
- `loopback = 1`;
- the warning that HCs hang without a slot.

### `headswe/arma3-mcp`

What it does: an MCP server that runs SQF in a running game and returns the result as JSON. It also has typed tools for units, vehicles, groups, markers, config, spawning, teleporting, time and weather, RPT reading, and Eden editing[^headswe].

How it talks to the game:
- The extension (`arma3_mcp_x64.dll` or `.so`) opens TCP on 127.0.0.1:9120 and speaks a line protocol: `AUTH`, `EXEC`, `STATUS`, `CANCEL`, `PING`. Any other line closes the connection, so a web page cannot smuggle an `EXEC` in[^headswe].
- A request raises the `ExtensionCallback` mission event, and the addon pulls it with `callExtension ["poll"]` and `["read", [id]]`. Requests and results travel in pieces sized to the 10 KB output buffer, so a result can be megabytes. A half-second poll covers dropped callbacks[^headswe].
- Results are serialized with `toJSON`, so it needs Arma 3 2.18 or newer[^headswe].
- Code runs unscheduled by default. `scheduled: true` spawns it. `throw` and compile errors come back as `ok: false`. Other script errors reach only the RPT, and come back as `rpt_errors` when the RPT is on the same machine[^headswe].
- `target` sends the code as code with `remoteExecCall` to the server, a client's owner id, or a player's name or UID, so clients need nothing installed. A mission with `CfgRemoteExec` in whitelist mode must allow it[^headswe].
- The token is optional. By default only the server or the singleplayer game listens (`allow_client = 0`)[^headswe].

Its own warning: an `ok` result only confirms the request completed, not that the intended effect happened[^headswe].

Machines: singleplayer, Eden preview, hosted, dedicated server. In the main menu nothing answers[^headswe].

Platform: Windows DLL and Linux `.so`, built with C++17 and CMake, no dependencies. A test loads the built extension into a simulated game[^headswe].

License: no LICENSE file[^headswe]. Ideas only.

Worth taking, as ideas:
- the line protocol that refuses anything else;
- chunking under the output buffer;
- `ExtensionCallback` plus a fallback poll;
- `remoteExecCall` so clients need no extension;
- a test that loads the real DLL without the game.

### `lepitinox/a3-bridge`

What it does: a mod that publishes game state ten times a second and events as they happen, and accepts commands (spawn, orders, terrain questions) over HTTP and WebSocket on 127.0.0.1:8765. It comes with a test web page, a replay recorder and a Node MCP server[^a3bridge].

How it talks to the game: a Rust extension runs the gateway in its own threads, and `A3B_fnc_tick` drains commands each frame. Events that fire only on players are relayed by a small script the server installs on each client with `remoteExec`[^a3bridge]. The gateway accepts browser requests only from its own test page, and the README says any local program can drive the game, including `exec_sqf`[^a3bridge]. There is no per-run token.

Machines: singleplayer and dedicated server. On one PC with Steam's Arma 3 Server install, the README says to start the game before the server, or Steam takes the server for the game and closes it[^a3bridge]. This is **unverified** for this framework's setup and belongs to the T5 spikes.

License: MIT, copyright 2026 Adrien Ducourthial[^a3bridge]. Code may be taken with the notice. The useful part is small: its Origin and Host checks in `ext/src/net.rs`[^a3bridge].

### `Zihuange/arma3-mcp`

What it does: an MCP server with an offline library of 5181 command signatures, "gates" that grade commands by risk, `exec`, config queries, screenshots, and input injection[^zihuange].

How it talks to the game: the MCP server listens on TCP 32132, and the extension in the game connects out to it, since the game can only poll[^zihuange].

Facts it measured:
- `exec` and the mod's `CfgFunctions` are not available in the main menu. CfgFunctions compile at preInit and postInit, which belong to a mission. `spawn` in `-init=` silently does nothing[^zihuange]. The framework therefore has no executor before a mission is loaded.
- Input goes through `SendInput`, and the extension refuses to inject unless the foreground window belongs to the game, because `SendInput` writes to the system input queue[^zihuange]. That breaks the project's rule never to take over the desktop. It is no model for CR-15.
- Screenshots use `PrintWindow` with `PW_RENDERFULLCONTENT`, falling back to `BitBlt`. This works while the game is in the background, but gives black frames in exclusive full screen[^zihuange].

License: MIT, "arma3-mcp contributors"[^zihuange]. The NOTICE says the command library comes from SQFLint (MIT), and that wiki descriptions were removed because the wiki's terms are not compatible with MIT[^zihuange].

Worth taking: the main-menu finding, and the reason why input injection by `SendInput` is out.

### `JTM-rootstorm/arma-mcp`

An Eden-editor bridge for Codex. A TypeScript sidecar talks to an HTTP bridge on 127.0.0.1 with a bearer token, and on to an extension. It deliberately exposes no raw SQF[^jtm]. No LICENSE file[^jtm]. It has nothing for a test framework.

### Remote executors from before MCP

- `realbadidas/Arma-Remote-Code-Executor` is a server mod that takes SQF in an HTTP(S) POST with a user and password, Windows only. It names known crashes on bad config, and is GPL-3.0[^rce]. GPL-3.0 code would force the whole project under GPL-3.0, so nothing is taken.
- `g0mb4/arma-ws` is a WebSocket extension from 2020 that crashes when the game is paused[^armaws]. There is no LICENSE file. The README says "Use, modify at your will, just please mention me"[^armaws]. That is an informal grant whose weight is the owner's call. Nothing in it is needed.

## Reference and lint servers

`a3ref/mcp`, `ScottTMConnors/arma3-wiki-mcp`, `Nighthawk42/arma3-mcp` and `habitualdev/IntelPackage` serve the BI wiki's command reference to agents, and some lint SQF[^a3ref][^wikimcp][^nighthawk][^intelpkg].

- None touches a running game.
- Each licenses its code under MIT and keeps wiki-derived data out of that license as non-commercial[^a3ref][^wikimcp][^nighthawk].
- For a test author writing SQF checks, `a3ref`'s `lint_sqf` reports made-up commands, missing operands, unknown `BIS_fnc_*` and deprecated commands[^a3ref]. It is worth recommending to test authors next to the framework, not building in.

## Debuggers and native bindings

- **ArmaDebugEngine** is a script debugger: breakpoints, call stack, variables in every namespace, stepping. It talks over a named pipe, `\\.\pipe\ArmaDebugEnginePipeIface`, and finds engine internals by pattern scanning (`findPattern` in `EngineHook.cpp`)[^ade]. Its VS Code client is tested only "lightly" on Eden mission scripts, and lists expression evaluation as able to crash the game[^vscodedbg].
  - Its LICENSE allows use only for Arma, non-commercial and non-monetized, and forbids sharing or adapting without written permission[^ade]. Nothing may be taken.
  - Its approach, patching engine internals, also runs into T11's reverse-engineering clauses.
- **Intercept** gives C++ plugins direct calls to the engine's SQF command functions, bypassing SQF[^intercept]. Its loader is tied to game builds: the last commits are "Update Loader for 2.22 Prof v9+" and "Update to v2.17.151987"[^intercept]. Every game update can break it until it is updated. It is MIT[^intercept]. The framework does not need native SQF bindings for the MVP, and a dependency that breaks with each game build works against G3.
- **arma-rs** is the Rust library most Rust extensions use: `#[arma]` entry points, command groups, callbacks through a `Context`, the call context (caller, source, mission, server, remote-exec owner) since 2.11, persistent state, and unit tests through `extension.call()`[^armars].
  - Its license disagrees with itself. The repository's LICENSE file is GPL-2.0, created 2023-04-18, while both crates declare `license = "MIT"` in `Cargo.toml`[^armars]. Every published version on crates.io, up to 1.13.0, carries MIT[^armarscrate].
  - Using the crate from crates.io under its declared MIT is a reasonable reading, but the conflict is for the owner to settle before the framework depends on it. Asking the author to align the two would settle it.

## HEMTT `launch`

`hemtt launch` builds a dev version of the mod with `hemtt dev` and starts Arma 3 with file patching on by default. It supports Workshop mods, DLC, Launcher presets, a mission, passthrough parameters, `-Q` to skip the build, and `-i/--instances` for several game instances[^hemttdoc].

Its code shows two limits[^hemttsrc]:
- the branch that would add `-connect=127.0.0.1` for a server is disabled (`if false`), so it starts clients only, never a dedicated server;
- with several instances it returns only the first child process.

HEMTT is GPL-2.0, at release v1.23.1[^hemttsrc]. The framework can call HEMTT as an external tool to build a mod. That is not code reuse. It cannot use HEMTT to run a session.

## Test frameworks inside the game

- **ArmaTesqf** (2021, MIT, needs CBA) runs test suites inside a mission.
  - Each test case is an SQF file with `__TESTCASE`, optional `__TAGS__` that a suite filters on, and `__BEFORE__`, `__TEST__` and `__AFTER__` blocks.
  - Each block is a `try`/`catch`: a `throw` in `__BEFORE__` marks the test "Crashed", and a failed `ASSERT_` macro marks it failed[^tesqf].
  - Tests run scheduled, so `sleep` is allowed[^tesqf].
- **CBA** ships in-game test scripts (`addons/*/test_*.sqf`), started by a per-addon `test.sqf` with `execVM` and `waitUntil`. They use `TEST_TRUE`, `TEST_FALSE`, `TEST_OP`, `TEST_SUCCESS` and `TEST_FAIL` macros that log to the RPT[^cbatest].
- **Neefay's scenarios** are YAML step lists: `wait_for`, `exec` with `expect`, `assert` with `no_errors` and others[^neefay].

All three give vocabulary for CR-50: a setup block, a test block, a teardown, tags for filtering, crashed as distinct from failed. None reports to a runner outside the game.

## Findings and what they force

1. **The channel is an extension plus an SQF addon.** Every working tool loads a native extension through `callExtension` and drains requests in SQF each frame or on `ExtensionCallback`[^headswe][^neefay][^a3bridge][^zihuange]. The BI wiki documents callbacks since 1.96 and Linux server extensions since 1.18[^wikiext]. This forces a framework mod with an extension and an addon. BattlEye must be off wherever the DLL loads, since non-whitelisted extensions are blocked[^wikiext][^headswe]. Touches CR-10, CR-27, CR-53, CR-61. T3 settles the details.
2. **One hub routes to machines.** Both multi-machine tools put the external endpoint on the server, or the singleplayer game, only, and reach other machines from SQF: CBA owner events[^neefay] or `remoteExecCall` of the code itself[^headswe]. The second way needs nothing installed on clients but meets `CfgRemoteExec` whitelists[^headswe]. This forces a hub-and-agent design with an owner registry and a "dispatched to N machines, M answered" result, so a missing machine reads as invalid, not failed. Touches CR-10, CR-22, CR-30, CR-31.
3. **There is no executor before a mission.** Mod functions and the scheduler are not there in the main menu[^zihuange], and nothing answers there in `headswe`[^headswe]. A session is only usable once a mission has loaded, and "connected" must mean "an executor answered". Touches CR-01, CR-27, CR-30.
4. **Errors can be traced to the request that caused them.** The `ScriptError` mission event handler plus a request tag compiled into the code attributes errors, including those of code the request spawned[^neefay]. This forces a request id on every step the framework sends. Touches CR-23. Whether every error kind on every machine is caught is **needs a spike**: pass when a deliberate type error, a compile error and an error in spawned code on a client each come back tagged with the step's id.
5. **Process lifetime conflicts.** Neefay spawns the game outside the MCP client's job object so it survives MCP restarts[^neefay]. CR-05 needs every process to die when the runner dies. The framework has to choose, and the spec must say which process owns the game: the runner, killed with it, or a long-lived session holder that the runner and the agent tools both talk to. Touches CR-01, CR-05, CR-53.
6. **Activation must be gated per run.** Per-PID session files with a random token keep the probe inert in any game the runner did not start[^neefay]. `headswe` has an optional token and a fixed port[^headswe], and `a3-bridge` has none[^a3bridge]. This forces a token per run and an ephemeral port published through a file. Touches CR-60.
7. **Key input by `SendInput` takes over the desktop.** The only tool that injects input needs the game in the foreground[^zihuange]. CR-15 needs another path, and T6 owns it.
8. **Screenshots need a renderer and have limits.** One tool hooks Direct3D Present[^neefay], the other uses `PrintWindow`, with black frames in exclusive full screen[^zihuange]. The Present hook changes how the game runs, which T11 weighs. Touches CR-25, a `could`. The engine's own `screenshot` command (T7) comes first.
9. **No existing tool is a base.** The tools closest to the need have no license[^neefay][^headswe], and the licensed ones are days old with a handful of commits[^ghmeta]. The framework is built from scratch, taking ideas, and code only from MIT sources with their notice kept. Touches the PRD's section 8 assumption.
10. **HEMTT builds, it does not run sessions.** `hemtt launch` starts clients only[^hemttsrc]. The framework owns starting the server, clients and HCs, and can call HEMTT as an external tool for builds with file patching. Touches CR-01, CR-03.

## Needs a spike

- **Round-trip time.** Pass when 100 calls of `1+1` to the server, and to one client through the hub, have a median under 100 ms and no call over 1 s on the test machine. Neefay's 20 to 60 ms is its own number on 2.22[^neefay].
- **Large results.** Pass when a 1 MB string result comes back intact through the 10 KB output buffer by chunking[^headswe].
- **`remoteExecCall` of code to a client with no framework mod.** Pass when the result and its errors come back, under a mission with no `CfgRemoteExec` and under one in whitelist mode.
- **Two rendering clients from one Steam account against a local server.** Pass when both join and stay connected for 10 minutes with `kickDuplicate = 0`. Neefay sets it but starts one rendering client[^neefay]. This belongs to T5.

## Could not verify

- Whether ArmaDebugEngine works on the current stable game build. Its last commit is 2025-02-01[^ade].
- The real-world use of any MCP tool beyond its stars and commits[^ghmeta].
- The arma-rs license conflict's intent[^armars][^armarscrate].

[^neefay]: https://github.com/Neefay/arma3-llm-harness at 21aac67e, README.md, docs/ARCHITECTURE.md, host/src/arma_harness/harness.py, host/src/arma_harness/serverroot.py.
[^headswe]: https://github.com/headswe/arma3-mcp at d778d75e, README.md and file tree.
[^a3bridge]: https://github.com/lepitinox/a3-bridge at cf42c739, README.md, LICENSE, ext/src/net.rs.
[^zihuange]: https://github.com/Zihuange/arma3-mcp at 380af427, README.md, LICENSE, NOTICE, docs/decisions.md (D20), game/extension/src/arma3_mcp_extension.cpp.
[^a3ref]: https://github.com/a3ref/mcp at 3109b0ef, README.md, LICENSE.
[^wikimcp]: https://github.com/ScottTMConnors/arma3-wiki-mcp at 90418253, README.md, LICENSE, NOTICE.md.
[^nighthawk]: https://github.com/Nighthawk42/arma3-mcp at 4c4bc777, README.md, LICENSE.
[^intelpkg]: https://github.com/habitualdev/IntelPackage at b365a260, README.md, LICENSE.
[^jtm]: https://github.com/JTM-rootstorm/arma-mcp at 1beb6061, README.md, file tree without a LICENSE file.
[^ade]: https://github.com/dedmen/ArmaDebugEngine at e01f80e4, README.md, LICENSE, NamedPipeServer.cpp, EngineHook.cpp.
[^vscodedbg]: https://github.com/Vindicta-Team/vscode-sqf-debugger at 854c32f7, readme.md, LICENSE.
[^rce]: https://github.com/realbadidas/Arma-Remote-Code-Executor at bb762ae4, README.md, LICENSE.
[^armaws]: https://github.com/g0mb4/arma-ws at 7958508b, README.md, no LICENSE file.
[^intercept]: https://github.com/intercept/intercept at 47ce89fc, README.md, LICENSE, commit history.
[^armars]: https://github.com/BrettMayson/arma-rs at 8e7b243c, README.md, LICENSE, arma-rs/Cargo.toml, arma-rs-proc/Cargo.toml.
[^armarscrate]: https://crates.io/api/v1/crates/arma-rs, versions 1.12.0 to 1.13.0 with license MIT.
[^hemttdoc]: https://hemtt.dev/commands/launch.html.
[^hemttsrc]: https://github.com/BrettMayson/HEMTT at 903589ed, bin/src/commands/launch/launcher.rs and platforms.rs; license GPL-2.0; release v1.23.1.
[^tesqf]: https://github.com/10Dozen/ArmaTesqf branch dev at dacfff1c, README.md, TestCase.sqf, LICENSE.
[^cbatest]: https://github.com/CBATeam/CBA_A3 at 8b15a04b, addons/common/test.sqf, addons/main/script_macros_common.hpp.
[^pythia]: https://github.com/overfl0/Pythia at b0b0061d, repository license MIT.
[^ghmeta]: GitHub API metadata, commit counts and repository searches, 2026-11-04.
[^wikiext]: https://community.bistudio.com/wiki/Extensions, revision 379582.
