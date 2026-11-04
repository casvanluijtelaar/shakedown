---
type: Research
title: Running code in the game from outside
description: Which channels carry SQF from a process outside Arma 3 into a running game and a result back, on which machines they work, their limits, how code reaches a chosen machine, and which language each one forces on the framework's internals.
status: stable
generated: { by: "claude-code/research", at: "2026-11-04T21:55:00+01:00" }
sources:
  - id: wiki-callext
    resource: "https://community.bistudio.com/wiki/callExtension (revision 378445)"
    read: 2026-11-04
  - id: wiki-ext
    resource: "https://community.bistudio.com/wiki/Extensions (wikitext, action=raw)"
    read: 2026-11-04
  - id: wiki-meh
    resource: "https://community.bistudio.com/wiki/Arma_3:_Mission_Event_Handlers#ExtensionCallback"
    read: 2026-11-04
  - id: wiki-freeext
    resource: "https://community.bistudio.com/wiki/freeExtension"
    read: 2026-11-04
  - id: wiki-remoteexec
    resource: "https://community.bistudio.com/wiki/remoteExec"
    read: 2026-11-04
  - id: wiki-re-framework
    resource: "https://community.bistudio.com/wiki/Arma_3:_Remote_Execution"
    read: 2026-11-04
  - id: wiki-servercommand
    resource: "https://community.bistudio.com/wiki/serverCommand"
    read: 2026-11-04
  - id: wiki-startup
    resource: "https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters"
    read: 2026-11-04
  - id: wiki-pipe
    resource: "https://community.bistudio.com/wiki/Arma_3:_Named_Pipe"
    read: 2026-11-04
  - id: wiki-servercfg
    resource: "https://community.bistudio.com/wiki/Arma_3:_Server_Config_File"
    read: 2026-11-04
  - id: wiki-debugconsole
    resource: "https://community.bistudio.com/wiki/Arma_3_Debug_Console (via web search summary only)"
    read: 2026-11-04
  - id: spotrep-120
    resource: "https://dev.arma3.com/post/spotrep-00120 (via web search summary: Game Update 2.22, 2026-08-11)"
    read: 2026-11-04
  - id: armars
    resource: "https://github.com/BrettMayson/arma-rs at commit 8e7b243ccf6fcf98fbe83a9dd86982336ac660d1 (crate 1.13.0): README.md, arma-rs/src/lib.rs, arma-rs-proc/src/lib.rs, .github/workflows/test.yaml, LICENSE"
    read: 2026-11-04
  - id: intercept
    resource: "https://github.com/intercept/intercept at commit 47ce89fc5fa78b3a04fb35d9672277c46f7787ea: README.md, LICENSE, tree (linux64/), commit log"
    read: 2026-11-04
---

# Running code in the game from outside

This is the core of the framework: `CR-10` (run code on a chosen machine and get the result back as data), `CR-22` (read a value on several machines), `CR-27` (ask a running game what is going on) and `CR-53` (agent tools). The question is which channel carries SQF from the runner, a process outside the game, into each game process, and the result back.

The current stable branch is Arma 3 2.22, released on 2026-08-11[^spotrep-120]. Every feature below is on stable unless it says otherwise.

## The channels

| Channel | Carries arbitrary SQF and a result? | Machines | Verdict |
|---|---|---|---|
| Extension: `callExtension` plus the `ExtensionCallback` event | Yes, both ways | Client, server (Windows and Linux), singleplayer | **The channel.** |
| Intercept (C++ host on top of an extension) | Yes, and native calls | Windows, a Linux 64-bit loader exists | Rejected: tied to game builds. |
| Named pipe (`-command=`) | No, five fixed commands | Client and hosting client only | A health probe at most. |
| `serverCommand` | No, server commands such as `#kick` | Server | Not a code channel. |
| Debug console | Yes, but through a UI a person clicks | Any, with access | Not a programmatic channel. |
| Files read with file patching | One way only (game reads, cannot answer except through the RPT log) | Any with `-filePatching` | A fallback, no result path. |

### Extensions are the only two-way programmatic channel

An extension is a DLL (or `.so` on Linux) that SQF calls with `callExtension`. The game looks for `name_x64.dll` on the 64-bit executable[^wiki-callext], in the root folder of every loaded mod and in the game's installation folder[^wiki-ext]. A Linux server loads `name_x64.so`; on Linux the name is case-sensitive[^wiki-callext][^wiki-ext]. Linux server support arrived in 1.18, the array syntax (`RVExtensionArgs`) in 1.68, callbacks in 1.96, call context in 2.12, feature flags and game functions in 2.18[^wiki-ext].

The extension cannot start code in the game on its own initiative with a plain call: `callExtension` is a call from SQF into the DLL. What makes it two-way is the callback. The engine hands the extension a function pointer through `RVExtensionRegisterCallback`; calling it fires the `ExtensionCallback` mission event handler with three strings, `name`, `function` and `data`[^wiki-ext][^wiki-meh]. The wiki dates callbacks to 1.96 on the Extensions page[^wiki-ext] and tags the event handler with 2.04[^wiki-meh]; both are long on stable.

So the shape is: the extension runs a background thread that listens for the runner (on a local socket, for example), and when an order arrives it calls the callback. An SQF agent in the game, registered as an `ExtensionCallback` handler, compiles the code in `data`, runs it, and hands the result back with `callExtension`. The wiki's own example does exactly this from a thread[^wiki-ext].

**Forces:** a small native extension in the framework, loaded on every machine the framework talks to, plus an SQF agent addon that registers the handler. `CR-10`, `CR-27`, `CR-53`.

### Intercept

Intercept is a C/C++ binding to the engine's SQF commands. A host DLL loads client DLLs that call engine functions directly, bypassing SQF, and it allows inline SQF for commands without a wrapper[^intercept]. It is MIT licensed since commit `f9fe4d5`[^intercept]. Its loader is patched per game build: the latest commit is "Update Loader for 2.22 Prof v9+" (2025-08-08), earlier ones are "Update to v2.17.151987" and "Fix loader to work with diag exe"[^intercept]. A Linux 64-bit loader exists, with an open issue asking for an update after a profiling-branch change[^intercept]. Whether it works on the current stable 2.22 build is **unverified**.

**Forces:** nothing. The framework needs to run SQF, not to bypass it, and a dependency that breaks on game updates conflicts with "Arma 3 only, in its current version" (PRD section 7). Rejected for the MVP. `CR-10`.

### The named pipe

With `-command=name` the game creates `\\.\pipe\name` and accepts five commands: `shutdown`, `message` (to `systemChat`), `reply` (echo), `session` (JSON with player id, island, mission, host) and `connect` (join a server)[^wiki-pipe][^wiki-startup]. It works on a client and a hosting client, not on a dedicated server, and runs no SQF[^wiki-pipe].

**Forces:** nothing for code. It is a cheap "is this client's main loop answering" probe and a clean `shutdown` for T8. `CR-30`, `CR-05`.

### `serverCommand`, the debug console, files

`serverCommand` executes server commands such as `#kick` or `#lock`, from a logged-in admin client in UI context or from the server with `serverCommandPassword`[^wiki-servercommand]. It runs no SQF. It is useful for T5 (mission restarts), not here.

The debug console runs code locally, globally or on the server, and itself uses remote execution[^wiki-debugconsole]. It is a dialog for a person; the framework has no way to drive it from outside without input to the window, which the PRD rules out.

With file patching on, the game can read loose files with `loadFile` and `preprocessFileLineNumbers`, limited to the game folder and below and by `allowedLoadFileExtensions` on a server[^wiki-servercfg]. A polling agent could read an order file, but it has no path back except writing to the RPT log. A fallback, not a channel.

## Which machines

| Machine | Extension loads | Notes |
|---|---|---|
| Singleplayer client | Yes | |
| Client in multiplayer | Yes | With BattlEye on, unwhitelisted client extensions are blocked; BattlEye is off for every run.[^wiki-ext] |
| Dedicated server, Windows | Yes | Extensions load only from the server root and its subfolders.[^wiki-servercfg] |
| Dedicated server, Linux | Yes, `.so` | Since 1.18.[^wiki-ext] |
| Headless client | **needs a spike** | A headless client is the client executable started with `-client`[^wiki-startup], so it should load extensions like any client. |

An extension only has to be installed on the hosts that call it[^wiki-ext]. The `ExtensionCallback` event is a mission event handler, so it needs a running mission on that machine; on a dedicated server or headless client, which has no interface, that is **needs a spike**.

**Forces:** the framework's mod, with its extension in the mod root, sits inside the server's folder tree on a dedicated server. One extension build per platform: `_x64.dll` for Windows, `_x64.so` for a Linux server. `CR-01`, `CR-10`.

## How code reaches a chosen machine

There are two ways, and the framework should use both.

**A channel per machine.** Every game process loads the extension, and every extension connects to the runner on its own. On one test machine all processes run locally, so each connects to `127.0.0.1`. The runner then addresses the server, a client by player name or a headless client directly, and a stall on one machine does not block orders to the others. The call context (2.12) tells the extension the local Steam id, the mission and the server name[^wiki-callext][^wiki-ext], and arma-rs also exposes the `remoteExec` owner[^armars], which together let a connection say which machine it is.

**Remote execution inside the game.** `remoteExec` and `remoteExecCall` ask the server to run a function or command on targets: `0` everyone, `2` the server, a `clientOwner` number, a negative number to exclude one, an object (where it is local), a side or a group[^wiki-remoteexec]. Both also work in singleplayer[^wiki-remoteexec][^wiki-re-framework]. Orders are queued and run in the order sent[^wiki-remoteexec]. With `remoteExec`, functions run scheduled and commands unscheduled; with `remoteExecCall` both run unscheduled[^wiki-re-framework]. They return a JIP id or `nil`, never the result[^wiki-remoteexec], and `CfgRemoteExec` can block a function or command[^wiki-re-framework]. The function only needs to exist on the target machine[^wiki-remoteexec].

**Forces:**
- The runner's primary path is the per-machine channel: `CR-10`'s targets (server, one player, all clients, a headless client) map to connections, not to `remoteExec` targets.
- Inside a check, the SQF API offers "run on machine X" through `remoteExec` of a framework function that sends its result back with a correlation id, either through `remoteExecCall` to `remoteExecutedOwner` or through the target's own extension. `CR-22` compares results gathered that way.
- The framework's mission or addon declares `CfgRemoteExec` entries for its own functions so a strict mission does not block them.
- `remoteExec` of `call` with code should be avoided where `CfgRemoteExec` may block `call`[^wiki-remoteexec]; the agent remote-executes its own named function.

## Limits

| Limit | Value | Source |
|---|---|---|
| Result of one `callExtension` | 10240 bytes (passed to the extension as `outputSize`) | [^wiki-callext] |
| Data sent to the extension | No limit; up to 2048 arguments in the array syntax | [^wiki-callext] |
| Blocking | `callExtension` blocks the game until the extension returns; a slow call drops frames | [^wiki-callext][^wiki-ext] |
| Slow-call warning | 1000 ms on clients, fixed; `callExtReportLimit` on a server; error code 301 | [^wiki-callext][^wiki-servercfg] |
| Callback slots | 100 per frame; the callback returns the slots left (99 to -1); a negative value means the call was dropped and must be retried | [^wiki-ext] |
| Callback timing | Calls from another thread are processed each frame; from the calling thread, on the next frame | [^wiki-ext] |
| Callback buffer | Kept until an `ExtensionCallback` handler exists; cleared when all handlers are removed and on mission restart | [^wiki-ext] |
| Version string | 32 bytes, printed to the RPT on load | [^wiki-callext] |
| Unloading | `freeExtension` works only on the development branch; on stable it does nothing | [^wiki-freeext] |

Since 2.22 the array syntax returns error codes for a missing extension (404), one blocked by BattlEye (403), blocked by script (412) or the wrong architecture (415)[^wiki-callext]. `-debugCallExtension` logs every extension call to the RPT[^wiki-startup]. Antivirus scanning can stall the first call to a new extension; the wiki suggests a dummy call at init[^wiki-callext].

**Forces:**
- Results larger than 10 KB are split by the extension and fetched in parts by the agent, or sent back through several callbacks. `CR-10`, `CR-22`.
- The extension never does work inside `callExtension`: it queues and returns at once. All socket work runs on its own thread. `CR-30` (a slow call would look like a stall).
- The extension keeps its own queue of orders and retries the callback on a negative return. arma-rs already does this: its callback thread retries every millisecond until the engine accepts the call[^armars].
- The agent registers the `ExtensionCallback` handler at every mission start, and the extension resends orders that were in flight across a mission restart. `CR-01`, `CR-02`.
- The extension cannot be replaced while the game runs on stable. Keep it thin and stable, and put the logic in the runner and in SQF, which can be reloaded (see `13-hot-reload.md`). `CR-03`.
- The agent does a dummy call at init and reports load errors by code. `CR-06`.

### Scheduled and unscheduled code

Code from a callback runs inside an event handler. Whether mission event handlers run unscheduled is not stated on the pages read here, so it is **unverified**; the remote execution page confirms that `remoteExecCall` runs unscheduled[^wiki-re-framework]. Unscheduled code cannot `sleep` or `waitUntil`[^wiki-remoteexec].

**Forces:** the agent runs a quick order ("read this value") inline and returns at once, and runs a check or a wait with `spawn`, reporting its progress and result later. `CR-20` waits live in scheduled code with a ceiling. Long inline orders stall the frame, so the agent bounds what it runs inline.

## Writing, signing and loading an extension

- **Rust with arma-rs** (crate 1.13.0, GPL-2.0)[^armars]. A `#[arma]` init function builds an extension with named commands and command groups (`group:command`), typed arguments and results, persistent state, callbacks from any thread through `Context`, call context, and a test harness that calls the extension and catches callbacks without the game[^armars]. It sets the `RVExtensionFeatureFlags` export and generates `RVExtensionVersion`, `RVExtension`, `RVExtensionArgs`, `RVExtensionRegisterCallback` and `RVExtensionContext`[^armars]. Its CI runs on Ubuntu and Windows[^armars]. Its license matters if the framework is open-sourced: linking it makes the extension GPL-2.0 (see T11).
- **C or C++** with the wiki's utility header, which covers Windows and Linux[^wiki-ext]. The wiki also shows C# with an export tool[^wiki-ext].
- **Signing:** extensions are not PBOs and have no `.bisign`. With BattlEye on, a client extension must be whitelisted by BattlEye since 1.46, or it is blocked[^wiki-ext]. With BattlEye off this does not apply; only the framework's PBOs are subject to signature checks, and a local test server can set `verifySignatures = 0`[^wiki-servercfg].
- **Feature flags (2.18):** `RVFeature_ContextNoDefaultCall` skips the context call when it is not needed; `RVFeature_ArgumentNoEscapeString` (2.22) passes string arguments unescaped, which is faster[^wiki-ext].
- **Newer interfaces:** `RVExtensionLoadFile` is tagged 2.24[^wiki-ext], above the current stable 2.22[^spotrep-120], so it is development branch only for now. The framework does not need it.

**Forces:** the extension is the one native component. arma-rs gives the shortest path to it (callbacks with retry, typed commands, unit tests without the game). C++ with the wiki header avoids the GPL-2.0 question at the cost of writing the plumbing. The choice goes to `spec/07-tech-stack.md` with T11's license reading.

## Which language each channel forces

The test author writes only SQF (`CR-50`). The framework's internals may use what the game requires (PRD section 7).

| Part | Language forced |
|---|---|
| Agent inside the game (handler, check API, `remoteExec` routing) | SQF, in an addon |
| Extension | Native: Rust (arma-rs), C or C++, or C# with an export tool |
| Runner, CLI, MCP server | Free; it talks to the extension over a local socket |
| Intercept, if used | C++ |
| Named pipe client | Free; Windows named pipe API |

So the framework has three parts in up to three languages: SQF in the game, a native extension, and a runner. If the extension and the runner share a language (Rust, for example), the socket protocol is the only seam.

## Security of the channel

The extension accepts code from outside and runs it with full SQF rights. `CR-60` asks for control from the local machine only, with a token per run.

**Forces:** the extension connects out to the runner on `127.0.0.1` (or listens only there) and presents a token the runner made for this run. How the token reaches the extension is **needs a spike**: an environment variable inherited from the runner, a file next to the profile, or a startup parameter read by the agent and passed in its first call. `CR-60`.

## Needs a spike

| # | Question | Pass condition |
|---|---|---|
| RX-1 | Round trip through the extension and callback on each kind of machine | The runner sends `1 + 1` to a singleplayer client, a dedicated server, a client and a headless client; each returns `2`. 100 sequential round trips per machine, none lost, no 301 warning in the RPT; the median time is recorded. |
| RX-2 | `ExtensionCallback` on a machine with no interface | On a dedicated server and a headless client the handler fires within 2 frames of the callback. |
| RX-3 | Large results | A 1 MB string result reaches the runner intact through chunking, with no frame over 100 ms on the game side. |
| RX-4 | Mission restart | Orders sent while a mission restarts are delivered once after the agent re-registers its handler; none lost, none run twice. |
| RX-5 | Token delivery | The extension in a game process started by the runner reads the run token through the chosen path, on Windows and on a Linux server. |
| RX-6 | Linux server | The `.so` build loads on a Linux dedicated server and passes RX-1. |
| RX-7 | Headless client loads the extension | A headless client started with `-client` logs `CallExtension loaded` for the framework's extension in its RPT. |

## Unverified

- Whether mission event handlers, `ExtensionCallback` included, run unscheduled.
- Whether Intercept works on stable 2.22.
- The debug console's details come from a search summary of its wiki page, not the page.

[^wiki-callext]: Bohemia Interactive Community Wiki, `callExtension`.
[^wiki-ext]: Bohemia Interactive Community Wiki, Extensions.
[^wiki-meh]: Bohemia Interactive Community Wiki, Arma 3 Mission Event Handlers, `ExtensionCallback`.
[^wiki-freeext]: Bohemia Interactive Community Wiki, `freeExtension`.
[^wiki-remoteexec]: Bohemia Interactive Community Wiki, `remoteExec`.
[^wiki-re-framework]: Bohemia Interactive Community Wiki, Arma 3: Remote Execution.
[^wiki-servercommand]: Bohemia Interactive Community Wiki, `serverCommand`.
[^wiki-startup]: Bohemia Interactive Community Wiki, Arma 3: Startup Parameters.
[^wiki-pipe]: Bohemia Interactive Community Wiki, Arma 3: Named Pipe.
[^wiki-servercfg]: Bohemia Interactive Community Wiki, Arma 3: Server Config File.
[^wiki-debugconsole]: Bohemia Interactive Community Wiki, Arma 3 Debug Console, as summarised by a web search.
[^spotrep-120]: Arma 3 SPOTREP #00120, Game Update 2.22, as summarised by a web search.
[^armars]: arma-rs repository at commit `8e7b243`, crate version 1.13.0.
[^intercept]: Intercept repository at commit `47ce89f`.
