---
type: Index
title: Milestones
description: One Milestone concept per milestone, file mNN.md, id MN.
---

# Milestones

* [M0 Walking skeleton](m00.md) - Every part of the chain exists and answers the next, in singleplayer: one command starts Arma 3, runs one SQF check that passes, writes the result file and leaves nothing running, behind the GitHub Actions gates.
* [M1 A grappling-hook check in singleplayer](m01.md) - The check API a real check needs, the reset between checks and the first fixture mods, in singleplayer: a rewritten arma-grappling-hook check gets on a rope through the mod's action, lets go with its CBA keybind and passes.
* [M2 Dedicated sessions](m02.md) - A dedicated server with game clients and headless clients on the one machine, each machine addressed by name, with code, waits, agreement and the reset across machines: a rewritten arma-grappling-hook check passes with two clients.
* [M3 Script errors and logs](m03.md) - Every script error on every machine traced to its step, each machine's log kept with the run and per check, the evidence of a failed wait or assertion, and no Steam ID or Steam name in any of it.
* [M4 Agent tools](m04.md) - The shakedown MCP server gives an AI agent every capability built so far as tools with the same JSON as the command line, and the command line gains exec, wait, expect, inspect and errors for a running session.
* [M5 A session that outlives its client](m05.md) - The runner, not the client, owns the session: it survives the end of the command or AI agent session that started it, takes its game down when it dies, recovers at its next start, tells a client it cannot reach what to do, and accepts control only with its tokens.
* [M6 Reload by restart](m06.md) - A changed mod file is reloaded into the running session by a game restart in one step, with the level and the reason reported; a check can ask for a mission restart before it; and a reload that fails leaves the session broken.
* [M7 Reload inside the mission](m07.md) - A reload asked during a run is refused and the checks a finding names are rerun after a reload; recompiling a function inside the running mission waits for DEF-21, since the game holds the files it read.
* [M8 Walk away](m08.md) - One command runs the whole suite, starting and stopping its own session, and every way a run ends, finished, stopped by hand or at its ceiling, leaves one result file, its logs and nothing running.
* [M9 Invalid, not failed](m09.md) - When the environment fails, a machine that stops answering, a process that crashes, a reset that cannot complete, a clock that stops, the check is invalid with its reason instead of failed, and every machine that still answers can be asked what is going on.
* [M10 The grappling-hook suite](m10.md) - The MVP of PRD section 9, proven on arma-grappling-hook: a set of its existing acceptance checks, rewritten for Shakedown, runs in singleplayer and on a dedicated server with two clients, from the command line and from an AI agent, and the default ceilings are set from measurement.
* [M11 Late joiners](m11.md) - A game client joins a running dedicated session late, leaves it and joins again, driven by a check, and the reset brings the session back to the machines it started with: arma-grappling-hook's late-joiner cases run on Shakedown.
* [M12 Shared helpers and mod events](m12.md) - Checks share helper code from a library folder compiled on every machine, and record a mod's CBA events and the engine's mission events on each machine with a wait on them: arma-grappling-hook's rope checks lose their copied kit.
* [M13 Every setup, repeated](m13.md) - One command runs a suite in every setup it needs, one session per setup; a check can be repeated with the spread of its outcomes reported, flaky included; and slow steps are flagged.
* [M14 Reload without a game restart](m14.md) - A changed mod function reaches every machine of a running session by a mission restart instead of a game restart, the game made to let go of the changed file by reading the framework's release files as spike SPK-19 found, and a reload that cannot free a file falls back to a game restart and says why.
* [M15 Hand over to the manual tester](m15.md) - A check builds a situation and hands it over: the manual tester finds a client's window at its start with a list of what to look for, and the session stays up until they are done.
* [M16 A start that explains itself](m16.md) - Before a start the environment is checked in full and every gap named at once, a mod in another version than the project names stops the start, a client that hangs fails the start naming it, every failed start keeps its reason, and no client can be sent to another server.
* [M17 ACE interaction and real keys](m17.md) - A check uses a mod's ACE interaction actions by their path, through ACE's own path, and, if spike SPK-11 passes, sends a real key to one client's window in the background.
* [M18 The full health watch](m18.md) - Each machine's frame rate and answer time are watched beside its clocks, shown in status, and a check that overlaps a machine starved of them is invalid, completing CR-30.
