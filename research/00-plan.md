---
type: Research
title: Research plan
description: The topics the research covers before the spec is written, the questions each must answer, its output file, its verification gate and whether the MVP waits on it.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T21:37:06+01:00 }
sources: []
---

# Research plan

The research gathers what is known about driving Arma 3 from outside for acceptance testing, so the spec of the framework has something to stand on. It starts from the PRD, `spec/client-requirements.md`, and reads every topic for what it forces into the framework: a component, a state, a limit, a requirement, an error case or a thing we cannot do.

The project is MVP first. Topics marked **MVP** must be done before the spec is written. Topics marked **later** get a short first pass now, enough to keep the spec from closing a door, and a fuller one when their requirements come up.

## Rules for every topic

- One OKF `Research` concept per topic, in the output file the topic names, listed in `research/index.md`.
- Every claim carries a footnote whose label is the `id` of an entry in the file's `sources` list: `{ id, resource, read }`. A claim that could not be checked at its gate is marked **unverified**.
- Facts are looked up, never answered from memory. Engine commands, tool features, versions and licenses change.
- Every finding ends in what it forces into the framework, and which `CR-nn` it touches.
- Claims about how the game behaves that only a run can settle (timings, reliability, what survives a reload) are marked **needs a spike**, with the pass condition a spike would use. The plan in step 3 turns them into spikes.
- Each topic is checked against its gate before the next starts, and marked done in the table below.
- An engine command or event is checked against the stable branch. One that needs the profiling or development branch says so.

## Topics

| # | Topic | Output file | MVP | Done |
|---|---|---|---|---|
| T1 | The harness of `arma-grappling-hook` | `10-prior-harness.md` | MVP | 2026-11-04 |
| T2 | Existing Arma automation, MCP and debug tools | `11-arma-tools.md` | MVP | 2026-11-04 |
| T3 | Running code in the game from outside | `12-remote-execution.md` | MVP | 2026-11-04 |
| T4 | Hot reload of mod code and checks | `13-hot-reload.md` | MVP | 2026-11-04 |
| T5 | Sessions: starting, keeping and resetting the game | `14-sessions.md` | MVP | 2026-11-04 |
| T6 | Making a player act: actions, key handlers and key input | `15-player-input.md` | MVP | 2026-11-04 |
| T7 | Observing the game: waits, events, errors, logs, screenshots | `16-observation.md` | MVP | 2026-11-04 |
| T8 | Health, stalls and clean stops | `17-health.md` | MVP | 2026-11-04 |
| T9 | Test automation tools for the web, mobile and games | `18-test-tools.md` | MVP | 2026-11-04 |
| T10 | Agent tools: MCP | `19-agent-tools.md` | MVP | 2026-11-04 |
| T11 | Licenses, the EULA and Bohemia's rules | `20-licenses-and-rules.md` | MVP | 2026-11-04 |
| T12 | What mod developers say about testing | `21-mod-developer-pain.md` | later | 2026-11-04 |

### T1 The harness of `arma-grappling-hook`

The only harness we have run. It is prior art and the first thing the framework replaces.

Questions:
- What does `harness/run.ps1` do, step by step: build, mission generation, setups, late joiners, several missions per start, the result file, the stall detector?
- What does `acceptance/run.ps1` add: suite files, cases, filters, setups per check?
- What do its checks look like in SQF, and which helpers (`acc_lib.sqf`) does every check need? That is the first draft of the check API's vocabulary.
- What has it measured: start times, mission change times, stall thresholds, start options tried?
- Where has it failed or been flaky, from its findings, docs and coordinator playbook: the Shadow stream and the stopped clock, clients kept across a mission change, kicks with one Steam ID, ACE's random falls?
- What should the framework keep, what replace, and what does it make unnecessary?

Gate: the files in `../arma-grappling-hook` at commit `3f4f7bd`, cited by path. Its measurements count as measured on the test machine, with their date.

### T2 Existing Arma automation, MCP and debug tools

Earlier research named these: `Neefay/arma3-llm-harness`, `headswe/arma3-mcp`, `lepitinox/a3-bridge`, `Zihuange/arma3-mcp`, `a3ref/mcp`, `ScottTMConnors/arma3-wiki-mcp`, `dedmen/ArmaDebugEngine`, `Vindicta-Team/vscode-sqf-debugger`, `realbadidas/arma-remote-code-executor`, `g0mb4/arma-ws`, Intercept, `arma-rs`, HEMTT's `launch` command. Search for any newer ones.

Questions, for each tool:
- What does it do, and which `CR-nn` does it cover?
- How does it talk to the game: extension, debug console, files, network? On which machines (singleplayer, server, clients, headless clients)?
- Platform and game version it needs. Dependencies (CBA, Pythia, its own addon).
- How mature is it: last commit, commits, issues, users?
- Its license, read from the LICENSE file. Can we take its code and still open-source this project later?
- What is worth taking: code, an idea, nothing?

Gate: the tool's repository at a named commit, its LICENSE file, and its own documentation. A feature counts only when its code or its docs show it. No license file means no reuse of code.

### T3 Running code in the game from outside

The core of the framework: `CR-10`, `CR-22`, `CR-27`, `CR-53`.

Questions:
- Which channels exist to send SQF into a running game and get a result back: `callExtension` and the `ExtensionCallback` event, Intercept, the debug console, `serverCommand`, files read with file patching, others?
- Which work in singleplayer, on a dedicated server, on a client and on a headless client, and which on a Linux server?
- How does code reach a chosen machine: `remoteExec`, `remoteExecCall`, `publicVariable`, a channel per machine?
- Limits: size of a call and its answer, calls per frame, scheduled and unscheduled code, what blocks the game's frame.
- How are extensions written today (Rust with `arma-rs`, C++, others), signed and loaded, and what does BattlEye off change?
- Which language does each channel force on the framework's internals, given that a test author writes only SQF?

Gate: the Bohemia Interactive Community Wiki page of each command and event, and the engine's changelog for when it was added. Library claims: its own docs at a named version.

### T4 Hot reload of mod code and checks

`CR-03`.

Questions:
- What can be reloaded into a running game without a restart: functions from `CfgFunctions`, CBA `PREP` functions, scripts, mission files, config, string tables, models?
- What does file patching allow, and what must be set on the server and clients for it?
- How do CBA and ACE developers reload today (`CBA` compile cache settings, `ACE` dev builds, HEMTT `launch`, `HEMTT` file patching builds)?
- What cannot be reloaded, so a check that needs it must restart, and how do we tell the user?
- How does a reload reach every machine of a dedicated session?

Gate: the BI wiki, CBA's and ACE's source and docs at a named commit, HEMTT's docs at a named version.

### T5 Sessions: starting, keeping and resetting the game

`CR-01`, `CR-02`, `CR-04`, `CR-05`, `CR-41`.

Questions:
- How do singleplayer, a dedicated server, clients and headless clients start from the command line, and what does each start option do (`-autotest`, `-autoInit`, `-filePatching`, `-world`, `-noPause`, profiles)?
- How can several checks run in one session without a mission restart, and how is the game state reset between them: deleting what a check created, a fresh mission, a fresh scenario in the same mission?
- What does the engine do when a client leaves and joins in progress, and when the mission changes with clients connected? What broke in T1?
- Running several clients on one machine: Steam IDs, `kickDuplicate`, ports, profiles, memory.
- How fast is each path: a full start, a mission change, a reset in place? T1's measurements first.

Gate: the BI wiki pages on startup parameters and server configuration, and T1's measurements. Timings not measured here are **needs a spike**.

### T6 Making a player act: actions, key handlers and key input

`CR-11`, `CR-12`, `CR-15`.

Questions:
- How does a check move a player, turn them, aim and look, from code: commands for position, direction, animation, and walking a unit to a point?
- How does a check use a mod's actions: `addAction` entries, ACE interaction menu entries, CBA keybinds and their handlers?
- How can key input reach one game window without taking over the desktop: window messages, input injection per process, DirectInput and raw input limits, what Arma actually reads? Does it work while the window is in the background, and over Shadow?
- Building a scene: creating objects, units and vehicles, setting time, weather and terrain position, and removing them again.

Gate: the BI wiki, CBA's and ACE's docs, Microsoft's Win32 documentation for any input API. Input claims about Arma are **needs a spike** unless a tool in T2 shows them working.

### T7 Observing the game: waits, events, errors, logs, screenshots

`CR-20`, `CR-21`, `CR-23`, `CR-24`, `CR-25`, `CR-26`.

Questions:
- How is a condition polled with a ceiling, in game time and in real time, on any machine?
- Which engine events and CBA events can a check subscribe to, and how are they collected per machine with timestamps?
- How are script errors caught as they happen (a mission event handler, `-showScriptErrors`, the RPT log) and tied to the step that caused them?
- Where are the logs of each machine, how do they rotate, and how are they read while the game runs?
- How are screenshots taken from inside the game (`screenshot` and its limits, where files go), and can a client record video?
- Which performance probes exist (`diag_fps`, `diag_codePerformance`, others)?

Gate: the BI wiki and CBA's docs.

### T8 Health, stalls and clean stops

`CR-05`, `CR-06`, `CR-30`, `CR-31`.

Questions:
- How do we tell a healthy session from a stalled one: game time advancing, frame rate, every machine answering? What did the Shadow stream do in T1?
- What happens to the game when its window loses focus, is minimised or is not shown?
- How does the runner stop every process it started, even when the runner itself crashes (Windows job objects, process trees)?
- How is the environment checked before a run: Steam running and logged in, the game and mods installed, versions?
- Where do crash dumps go, and how do we tell a crash from a stop?

Gate: the BI wiki, Microsoft's documentation for process APIs, T1's findings.

### T9 Test automation tools for the web, mobile and games

What the check API, the runner and the report should feel like. `CR-31`, `CR-32`, `CR-40`, `CR-42`, `CR-50`, `CR-51`.

Questions:
- Playwright and Maestro: how a test reads, auto-waiting, locators, fixtures and setup, retries and flake reporting, filters, trace and report output, how agents use them.
- Game test tools: Unity's Test Framework, Unreal's Automation and Gauntlet, AltTester, GameDriver, and any others. How do they drive a running game and wait on it?
- What of each fits a check written in SQF?

Gate: each tool's official documentation at its current version.

### T10 Agent tools: MCP

`CR-53`, `CR-60`.

Questions:
- The current version of the Model Context Protocol: tools, transports (stdio, HTTP), long-running calls, progress, and how a server reports errors.
- How Playwright's MCP server and the Arma MCP servers from T2 shape their tools.
- Running a local MCP server safely: binding to the local machine, a token per run.
- How Claude Code loads a project's MCP server.

Gate: the specification on `modelcontextprotocol.io` at its current version, and Claude Code's documentation.

### T11 Licenses, the EULA and Bohemia's rules

The constraints of PRD section 7.

Questions:
- What do the Arma 3 EULA and Bohemia's terms say about tools that drive the game, extensions (DLLs), automation and running several copies on one machine?
- Under which licenses are CBA, ACE and the tools of T2 published, and what does each mean for reusing their code in a project that may be open-sourced later?
- Do test runs store anything personal (Steam names, IDs in logs) that a privacy rule touches?

Gate: the text of the EULA and each license itself, not a summary of it.

### T12 What mod developers say about testing (later)

Questions:
- What do Arma mod developers complain about when testing mods: forums, Reddit, CBA and ACE issues and discussions?
- What workarounds do they use, and which failure modes keep coming up?

Gate: the post or issue itself, quoted with its link.

## After the topics

- `research/01-best-of-breed.md`: a table of capabilities. Each row says which tool or idea we take, which we reject, and why, with its license.
- `research/02-gaps.md`: the holes, conflicts and open questions of the PRD against the research, each with a recommendation.

## Decisions for this research

- **No measurements on the test machine.** The research is desk research. Every timing and every claim about game behaviour that only a run can settle is marked **needs a spike**, with its pass condition, for the plan.
- **The stable branch.** Engine features are checked against Arma 3's current stable branch. A feature that exists only on the profiling or development branch is recorded with its branch, and recommended only where it makes the framework much better or simpler. `01-best-of-breed.md` names each one.
- **Discord through the owner.** No tool reads Discord: automating a user account breaks Discord's terms, and a bot needs the server's admins. The owner runs the searches below in the ACE Discord and pastes what is useful into the session. The research cites such a source as `discord:ace/<channel>/<date>`, with no usernames, and marks every claim from it **unverified** until a public source or a spike confirms it. T12 also uses forums, Reddit and GitHub issues and discussions.
- **One session.** The MVP topics run in one session. T12 gets a short pass.

## Searches for the ACE Discord

The owner runs these in Discord's search on the ACE server, and pastes the threads that answer a question of a topic, with the channel and date of each.

| Search | For |
|---|---|
| `unit test` and `automated test` | T2, T9, T12: how ACE and CBA test today |
| `callExtension` and `extension` | T3: extensions in use, and their pitfalls |
| `filePatching` and `hot reload` and `recompile` | T4: how developers reload code |
| `headless` with `test` | T5: multiplayer test setups |
| `dedicated` with `test` | T5: running a local server for tests |
| `CI` and `github actions` and `hemtt launch` | T5, T8: anything run unattended |
| `keybind` with `test` | T6: driving key handlers from code |
| `screenshot` | T7: taking screenshots from code |
| `flaky` and `random` with `test` | T8, T12: what makes tests unreliable |
| `ScriptError` and `showScriptErrors` | T7: catching script errors |
