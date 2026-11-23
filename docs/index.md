---
type: Index
okf_version: "0.2"
title: Shakedown documentation
description: The guide to Shakedown, the test automation framework for Arma 3 mods - a quickstart, then every feature in detail with examples, read top to bottom.
---

# Shakedown documentation

Shakedown runs automated checks of an Arma 3 mod inside the live game, in singleplayer or on a dedicated server with game clients and headless clients, all on one Windows machine. You write checks in SQF. Shakedown starts the game, resets it between checks, runs each check on the machines it names, and tells you `pass`, `fail` or `invalid` with a reason, in one result file per run.

New to it? Read the quickstart, then the pages in order. Each page stands on its own, so come back to any of them as a reference.

## Getting started

* [Quickstart](quickstart.md) - From nothing to a first passing check in singleplayer and on a dedicated server - install Shakedown, write a project file and a check, run it, keep a session up, and read the result and the exit code.
* [Installing, upgrading and removing](install.md) - What Shakedown needs on the machine, how to install a release with winget, the one-line script or the zip by hand, how to upgrade and remove it, what a release holds and its licence, the files the runner keeps for itself, and how to build and install from source.
* [The project file](project-file.md) - Every key of shakedown.toml with examples - the game, the tests and library folders, base_port and slow_step, mods with CBA, ACE, pinned versions and the keys reload watches, and singleplayer and dedicated setups with clients, headless clients and late joiners - and what a mistake in the file looks like.

## Sessions

* [Sessions and the runner](sessions-and-the-runner.md) - What a session and the runner are, how to start, watch and stop a session, why only one runs at a time, how a session outlives the command or AI agent session that started it, what happens when the runner dies or cannot be reached, and how to end a runner that does not answer.
* [Starting a session, and when it fails](starting-and-failed-starts.md) - What session start checks before any game process starts and the message for every gap, how to pin a mod's version, what makes a started session fail, where the reason of a failed start is kept, and what happens to a client that code sends to another server.
* [Dedicated servers, clients and headless clients](dedicated-sessions.md) - How to declare and start a dedicated session on one machine, how a check names each machine and group, how code, waits and agreement reach other machines, how a check makes a client join late, leave and join again, and what the reset does across machines.

## Writing checks

* [Writing checks](writing-checks.md) - How a check is written in SQF - its file and id, the header with @check, @tags, @setups, @timeout and @reset, the body and the lead machine, steps, what code sent to another machine sees, shared helpers in the library folder, and the message each mistake gives.
* [The check API](check-api.md) - Every skd_fnc function a check can call, with its arguments, what it returns, an example and how it fails, and the rules every function keeps.
* [Waits and assertions](waits-and-assertions.md) - How a check waits for a condition and asserts a value on one machine, on several at once, or that machines agree, with no sleep - every operator, ceilings, and the message each failure gives.
* [Scenes, resets and cleanups](scenes-resets-and-cleanups.md) - How a check builds its scene, what the reset between checks restores and what it leaves, cleanups for state the reset cannot name, a check that starts in a fresh mission, resetting by hand, and what a failed reset does.
* [Acting through the mod](acting-through-the-mod.md) - How a check makes a player use a mod's scroll-menu action by its title and press a CBA keybind by its names, how each fails, and why a check cannot send a real key to the game.
* [ACE interaction actions](ace-interaction.md) - How a check makes a player use a mod's ACE interaction action, on an object or on itself, by its path of action names, how to find the path, how it fails, and the ACE version it is pinned to.
* [Recording the mod's events](recording-events.md) - How a check records a CBA event or an engine mission event on the machines it names, reads each with its arguments and clocks, and waits for one.
* [Script errors, logs and evidence](script-errors-and-logs.md) - How a script error on any machine fails a check and is traced to its machine and step, how a check expects an error, each machine's log in the run folder and how to read it, and the evidence a check keeps.
* [Handing over to a person](handing-over.md) - How a check sets up a situation and hands it over to a person with a list of what to look for, what they find on screen and in the output, and how the session stays up for them.

## Running checks

* [Running checks](running-checks.md) - How to list checks without the game, run them against a session or with one of their own, select them by id and tag and in which order they run, what the printed lines mean, the exit codes, and what a run leaves behind.
* [Every setup, repeats and slow steps](every-setup-and-repeats.md) - How one command runs a suite in every setup its checks name, how to repeat a check and read a flaky summary, and how slow steps and slow checks are flagged.
* [Running a suite unattended](unattended-runs.md) - How one command starts a session, runs the suite and leaves nothing running, how to stop a run by hand, the runner ceiling that stops a run that hangs, and the result of a stopped run.
* [Outcomes and a broken session](outcomes-and-broken-sessions.md) - What pass, fail and invalid mean and every reason kind, which cause decides, the health watch with frame rate and answer time, lost machines and a broken session, how to look into one, and how to make a failure on purpose.
* [Results and the result file](results.md) - What a run leaves in its run folder, and every field of result.json, with an example and how values from the game are written.

## A live session

* [Asking a running game](asking-a-running-game.md) - How to run SQF on a machine or a group of a running session, wait for a condition, try an assertion live, inspect a machine, read its logs and list its script errors.
* [Reloading changed mod files](reloading.md) - How reload brings changed checks, library files and mod files into a running session at the levels none, mission-restart and game-restart, what each costs, how to set a mod up for it, and what a failed reload does.

## AI agents

* [AI agents and the MCP tools](agent-tools.md) - How an AI agent drives Shakedown through the shakedown MCP server - the project's .mcp.json, every tool and its command line twin, the rules every tool keeps, the error codes, the agent's workflow, and the text the guide tool returns.

## Reference and help

* [The command line](command-line.md) - Every shakedown command and option with its defaults and limits, its MCP twin, the common options, how Ctrl+C stops a run, and the exit codes of every command.
* [Porting an existing suite](porting-a-suite.md) - How to move a mod's checks from PowerShell scripts that start the game and read its RPT to Shakedown - what each part becomes, the order of the work, how to rewrite one check, and how to show the port reaches the same verdicts.
* [Security and privacy](security-and-privacy.md) - How Shakedown keeps control of a session to the local user, records every piece of code sent to the game, keeps the Steam account out of everything it writes, and what it does not yet cover.
* [Troubleshooting](troubleshooting.md) - What to do when a session does not start, a session is broken, checks end invalid or fail with errors you did not cause, the runner does not answer or the install fails, and the known limits.
