---
type: Index
okf_version: "0.2"
title: Documentation
description: How the code realises the spec, and what each finished milestone built.
---

# Documentation

* [Installing Shakedown](install.md) - What Shakedown needs on the machine, how to build and install it from this repository, what the install folder holds, and the files the runner keeps for itself.
* [The project file](project-file.md) - The keys of shakedown.toml for singleplayer and dedicated setups, with an example, how relative paths and defaults work, and what a mistake in the file looks like.
* [Dedicated sessions](dedicated-sessions.md) - How to declare and start a dedicated server with game clients and headless clients on one machine, how a check names each machine and group, how code, waits and agreement reach other machines, what the reset does across machines, and how to see a session's machines with status.
* [Writing and running a check](writing-and-running-checks.md) - How to write a check in SQF - its file, header and body - how to list and run checks in a singleplayer session, the reset between checks, and what the run leaves behind.
* [The check API by example](check-api.md) - Every skd_fnc function a check can call, with an example of each - the context, code on a machine, waits, assertions, the scene, actions and keybinds, cleanups and ending a check - and what each does when it fails.
* [Milestone reports](milestones/) - One Milestone Report per finished milestone.
