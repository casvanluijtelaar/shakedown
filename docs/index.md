---
type: Index
okf_version: "0.2"
title: Documentation
description: How the code realises the spec, and what each finished milestone built.
---

# Documentation

* [Installing Shakedown](install.md) - What Shakedown needs on the machine, how to build and install it from this repository, what the install folder holds, and the files the runner keeps for itself.
* [The project file](project-file.md) - The keys of shakedown.toml for singleplayer and dedicated setups and for a mod that reload watches, with an example, how relative paths and defaults work, and what a mistake in the file looks like.
* [Dedicated sessions](dedicated-sessions.md) - How to declare and start a dedicated server with game clients and headless clients on one machine, how a check names each machine and group, how code, waits and agreement reach other machines, what the reset does across machines, and how to see a session's machines with status.
* [Writing and running a check](writing-and-running-checks.md) - How to write a check in SQF - its file, header and body - how to list and run checks in a singleplayer session, the reset between checks, and what the run leaves behind.
* [The check API by example](check-api.md) - Every skd_fnc function a check can call, with an example of each - the context, code on a machine, waits, assertions, the scene, actions and keybinds, cleanups and ending a check - and what each does when it fails.
* [Script errors, logs and evidence](script-errors-and-logs.md) - How a script error on any machine fails a check and is traced to its machine and step, how a check expects an error, what each machine's log in the run folder holds and how to read it with logs, the evidence a failed check keeps, and how the Steam account is kept out of all of it.
* [Asking a running game](asking-a-running-game.md) - How to run code on a machine of a running session, wait for a condition, try an assertion live, inspect a machine and list the script errors it caught, from the command line, and what status shows of a start that failed.
* [Reloading and the mission restart](reloading.md) - How reload classifies a changed file and applies its level, why every change of mod code is a game restart made in one step with the mod's rebuild, how to set a mod up for it, why a reload during a run is refused, how to rerun the checks a finding names, what reloads inside the mission and what partial means, how a check asks for a mission restart with @reset mission, how a broken session comes back, and what the reload result holds.
* [Agent tools](agent-tools.md) - How an AI agent drives Shakedown through the shakedown MCP server - the project's .mcp.json, every tool and its command line twin, the rules every tool keeps, the agent's workflow for a session and a run, the error codes, and the text the guide tool returns.
* [When a client or the runner ends](sessions-and-the-runner.md) - What happens to a session when the command or the AI agent session that started it ends, when the runner itself dies, and when a command cannot reach the runner, how to end a runner that cannot be reached, and how the runner keeps control to the local user and records every piece of code sent to the game.
* [Milestone reports](milestones/) - One Milestone Report per finished milestone.
