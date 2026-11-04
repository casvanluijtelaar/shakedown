---
type: Specification
title: Overview
description: What Shakedown is for, who uses it, what is in and out of its scope, its parts, and the words it uses with a fixed meaning.
status: draft
---

# Overview

Shakedown checks an Arma 3 mod's behaviour inside the running game, by machine, the way Playwright checks a web page. A test author writes a check in SQF. Shakedown starts the game (singleplayer, or a dedicated server with clients and headless clients), keeps it running across many checks, runs each check inside it, and reports each check as `pass`, `fail` or `invalid` with the evidence. A mod developer reloads changed code into the running game and reruns the checks a finding names. An AI agent does all of this through tools as well as a person does it through the command line.

The product requirements are in [client-requirements.md](client-requirements.md) (the PRD). This file and the files after it are the specification: they settle what the PRD leaves open, give every requirement an ID, and are what the framework is built and tested against. Where a requirement cites a `CR-nn`, it is the PRD's.

## Who uses it

| User | What they do with Shakedown |
|---|---|
| Test author | Writes checks in SQF against the check API, from a mod's user stories, without reading the mod's source. Runs them, reads the verdicts. Often an AI agent. |
| Mod developer | Changes the mod, reloads it into a running session, reruns the checks a finding or story names. Often an AI agent in the mod's own repository. |
| Manual tester | Is put at the start of a situation by the framework (`CR-52`, after the MVP). |

## Goals

The PRD's goals G1 to G4 hold as written, with the owner's answers in `research/02-gaps.md`: G2's 30 seconds applies to a change that reloads inside the mission, in singleplayer and on a dedicated server; the target for a change that needs a mission restart is set once spike `SPK-08` has measured it.

## Scope

In scope, for the first release (the MVP, PRD section 9):

- Arma 3 on Windows, the current stable branch.
- Two kinds of session: singleplayer, and a dedicated server with clients and headless clients, every process on the one local machine.
- Any mod loaded alone or with its dependencies, such as CBA and ACE3. Nothing in the framework is specific to one mod.
- Starting, keeping and stopping the game processes; running checks against a running session; resetting state between checks; reloading changed code.
- Running SQF on a named machine and reading the result back as data; building a scene; making a player use a mod's `addAction` actions and CBA keybinds; waiting and asserting on one or every machine; catching script errors per machine and step; keeping each machine's log.
- A command line and an MCP server for AI agents, with the same results.

Out of scope, as the PRD says: Arma Reforger, Arma 4 and DayZ; CI/CD; unit tests of SQF outside the game; building, packing or signing mods; public servers and BattlEye; screenshots as a verdict; the machine's mouse and keyboard; reading or patching the mod's source.

Also out, by the owner's answers in `research/02-gaps.md`:

- A player-hosted (non-dedicated host) session, until a spike shows an unattended way to start one (H9).
- A Linux dedicated server, deferred; the extension is written so it builds for Linux (H10).
- ACE interaction menu actions, a `should` after the MVP (H7, Q6).

## The parts

| Part | What it is | Where it runs |
|---|---|---|
| **Runner** | A long-lived process that owns every game process of a session in a Windows job object, holds the session's token, routes orders to machines, enforces the runner ceilings, decides `invalid`, and writes the result file. | The local machine, outside any AI agent's process tree. |
| **Command line** | The `shakedown` command. A thin client of the runner. | The local machine. |
| **MCP server** | `shakedown mcp`, an MCP server over stdio that an AI agent's client starts. A thin client of the runner with the same results as the command line. | The local machine. |
| **Extension** | `shakedown_x64.dll`, a thin native extension that connects its game process to the runner over loopback. | Inside every game process of a session. |
| **In-game agent** | SQF in the framework's addon. It receives orders from the extension, runs them, sends results back, catches script errors, records created entities and runs the check API. | Inside every game process of a session. |
| **Check API** | The SQF functions a check is written with (`skd_fnc_*`). | Inside the game, as part of the in-game agent. |

[07-tech-stack.md](07-tech-stack.md) says what each part is built from. The contract a test author and the test developer work from is listed there.

## Words with a fixed meaning

Each word below means this and only this in the spec, the contract, the docs and the code. "AI agent" always means a model such as Claude Code; "in-game agent" always means the framework's SQF inside the game.

| Word | Meaning |
|---|---|
| **Project** | A folder with a `shakedown.toml` file: the mod repository that uses Shakedown. It names the game, the mods, the setups and the checks folder. |
| **Setup** | A named, declared shape of a session in the project file: its mode (`singleplayer` or `dedicated`), the number of game clients and headless clients, the mods loaded and the world. Examples: `sp`, `dedicated-2`. |
| **Session** | The set of game processes started for one setup and kept running while many checks run against it, with the one long-running session mission they play. A session has an id, a token and a folder. At most one session runs on the machine at a time. |
| **Session mission** | The mission the runner generates for a session. It places the playable units and headless client slots of the setup, loads the in-game agent, and runs until the session stops or restarts it. |
| **Machine** | One game process of a session, named by its role: `server`, `p1` to `pN` for game clients, `hc1` to `hcN` for headless clients. A singleplayer session has one machine, `p1`, which `server` also names. A machine's name is also its profile name (`-name`). |
| **Lead machine** | The machine a check's body runs on: `server` on a dedicated session, `p1` in singleplayer. |
| **Check** | One SQF file in the project's checks folder with a header that names it and its tags. Its body runs on the lead machine and ends with one outcome. A check proves one promise, named with the story or requirement ID it proves. |
| **Check id** | A check's file path relative to the checks folder, with `/` as separator and without the `.sqf` extension, such as `rope/US-012-hook-holds`. |
| **Step** | A named, timed part of a check, made with `skd_fnc_step`. Steps nest. Every error, wait and assertion is recorded against the innermost step that was running. |
| **Tag** | A word in a check's header used to select it, such as a story ID (`US-012`), a finding ID (`m01-3`) or `smoke`. |
| **Run** | One execution of a selected list of checks against one session, started by one command or tool call. A run has an id and writes one result file. A session serves many runs. |
| **Outcome** | The verdict of one check in one run: `pass`, `fail` or `invalid`, always with a reason when it is not `pass`. Defined in [01-domain-model.md](01-domain-model.md). |
| **Wait** | A poll of a condition on a machine until it holds or a ceiling passes. Never a fixed sleep. |
| **Ceiling** | The longest a wait, a check or a call may take. Each has two: the in-game ceiling on real time (`diag_tickTime`) inside the game, and the runner ceiling on the wall clock in the runner. |
| **Reset** | What the framework does between two checks in one session so the second sees the session's start state: deletes what the first created and restores players, time and weather. |
| **Reload** | Bringing changed mod code or checks into a running session. |
| **Reload level** | How deep a reload goes: `mission` (A: inside the running mission), `mission-restart` (B: the session mission restarts in the running session) or `game-restart` (C: every game process of the session restarts). The lowest level a change allows is used. |
| **Evidence** | What a result carries to explain an outcome: the failed step, the last observed values, the script errors, the attached values and the paths to each machine's log excerpt. |
| **Run folder** | `.shakedown/runs/<run id>/` in the project: the result file and the log excerpts of one run. |
| **Session folder** | `.shakedown/sessions/<session id>/` in the project: the generated mission and configuration and one profile folder per machine start. |
