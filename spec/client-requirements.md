---
type: Specification
title: Client requirements
description: The product requirements for Shakedown, a test automation framework for Arma 3 mods - the problem, the goals, the users, the scope and what the first release must do.
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T21:37:06+01:00 }]
---

# Client requirements

This is the product requirements document (PRD). It says **what** the framework must do and **why**, not how it is built. The research (step 1) checks it against the domain, and the specification in `spec/` (step 2) fills its holes with requirement IDs, a domain model and user stories. Where this document and the research disagree on goals or scope, this document wins.

Each requirement has an ID of the form `CR-nn`, never reused, so the spec can trace back to it. Write what the user needs, not the screen or code that delivers it. Where a decision is still open, put it in section 10 rather than guessing.

## 1. Problem

Arma 3 addons and mods can only be verified by playing them. Someone starts the game, loads a mission, walks to the right spot, does the thing and watches what happens, in singleplayer and again on a dedicated server with several clients. Nothing makes the game do this on its own and report a verdict, the way Playwright does for the web and Maestro does for mobile apps.

That makes acceptance testing of a mod slow, manual and impossible to delegate to agents:

- **Every check is a full game start.** Changing one check means a fresh game start and a whole mission. Chasing one defect once took 12 starts of 3.5 minutes each. A single start of server, clients and headless client costs 20 to 35 seconds before any mission runs.
- **Nobody can look into a running game.** When a run stalls or misbehaves, the cause is guessed from a log after the fact. An agent cannot ask the game what is going on.
- **Setting up a situation is manual.** Placing objects, moving a player to an exact spot facing an exact way, and making the player use the mod go through debug-console pastes.
- **Results cannot be trusted.** A stalled game looks like a failed check. A check that passes may never have been able to fail. Flaky behaviour shows up only by repeating runs by hand.
- **Multiplayer is the hard part and the least tested.** Whether every machine agrees, whether the server keeps authority, what a late joiner or a cheating client sees: these need several machines driven at once.

### The first user

The first mod the framework serves is `arma-grappling-hook`: a CBA-based mod with optional ACE3 integration, where the server holds authority over ropes, hooks and players climbing them, and late joiners, disconnects and respawns all matter. Its acceptance checks today are handcrafted PowerShell scripts that start the game per run and are flaky. The framework replaces them. Its milestones demo against that mod, and nothing in the framework is specific to it.

## 2. Goals and success measures

| # | Goal | Measure | Target |
|---|---|---|---|
| G1 | A mod's behaviour in the game is checked by machine, not by a person playing it. | Share of a mod's user stories with an automated in-game acceptance check. | 80 percent. |
| G2 | The edit-and-check loop is fast. | Time from changing a check or the mod's code to that check's verdict, with the game already running, for a change that reloads inside the mission. | Under 30 seconds, in singleplayer and on a dedicated server. A change that needs a mission restart gets its target once a spike has measured it. |
| G3 | A verdict can be trusted. | Share of failed verdicts caused by the environment instead of the mod (a stall, a crash, a dead Steam). | None. Those runs are reported invalid, not failed. |
| G4 | Multiplayer behaviour is tested as routinely as singleplayer. | Checks that run on a dedicated server with several clients, in one command. | Every multiplayer check starts its dedicated session with one command. |

### How we build it

A working MVP early beats a careful product late. The plan puts a usable framework in a test author's hands as soon as possible and grows it from there. Each milestone delivers the shortest path to its demo. Edge cases, hardening and `should` and `could` requirements come after the MVP works end to end, and only where using it shows they matter.

## 3. Users

| User | Situation | Trying to get done |
|---|---|---|
| Test author | Writes acceptance checks for a mod from its user stories, without reading the mod's source. Often an AI agent (the test developer role), sometimes a person. | Set up a situation in the game, make players do things, wait for an outcome and assert it, on every machine that matters. |
| Mod developer | Changes the mod's code and wants to know at once whether it still does what it promised. Often an AI agent working in the mod's own repository. | Reload the changed code into a running game and rerun the checks a finding names, without a restart. |
| Manual tester | A person who does the visual and feel checks no script can judge. | Be put at the right spot in the right situation by the framework, with a list of what to look for, instead of pasting into the debug console. |

## 4. Scope

### In scope

- Arma 3, in singleplayer and in multiplayer on a dedicated server with clients and headless clients.
- Testing any addon or mod, loaded alone or with its dependencies, such as CBA and ACE. No part of the framework is specific to one mod.
- Starting, keeping alive and stopping the game processes a run needs, on the local machine.
- Running code on a chosen machine and reading the result back as data.
- Building a scene: placing objects, units and vehicles, setting time and weather, moving and facing players.
- Making a player act through the mod's real path: its actions and key handlers, and key input sent to the game where a check needs it.
- Observing what happens: waits on conditions, the mod's and the engine's events, script errors, logs and state on each machine.
- Writing checks, running a chosen subset, and reporting each verdict with its evidence.
- Use by AI agents as well as people.

### Out of scope

- Arma Reforger, Arma 4 and DayZ.
- CI/CD. The framework runs on a developer's machine. Running it on a CI runner is deferred.
- Unit testing of SQF functions outside the running game. Mods keep their own unit tests.
- Building, packing or signing mods. The framework loads a mod that some other tool built.
- Hosting public servers, or running with BattlEye on. Test runs are local and private.
- A screenshot or recording as a pass or fail criterion. Screenshots are output for a person or an agent to interpret.
- Taking over the machine's mouse and keyboard. Input goes to the game, not to the desktop.
- Reading or patching the mod's source to make a check pass. The framework drives the mod the way a player and the game do.

## 5. Requirements

`must` is in the MVP. `should` follows soon after it. `could` comes when use shows it is needed.

### Lifecycle

| ID | Requirement | For | Priority |
|---|---|---|---|
| CR-01 | Start a session once (singleplayer, or a dedicated server with a chosen number of clients and headless clients) and run many checks against it without restarting. | Test author | must |
| CR-02 | Reset the game state between checks in a session, so one check cannot affect the next. | Test author | must |
| CR-03 | Reload the mod's changed code and the checks into a running session at the lowest level the change allows: inside the mission, by a mission restart within the session, or by a game restart. Report the level used and why. Checks always reload inside the mission. | Mod developer | must |
| CR-04 | Make any machine join, leave or rejoin during a session, including a late joiner and a server restart. | Test author | should |
| CR-05 | Stop every process a run started, whether the run passed, failed or the runner itself crashed. | Test author | must |
| CR-06 | Check the environment before a run (game installed, Steam running and logged in, mods present in the right versions) and say exactly what is missing. | Test author | should |
| CR-07 | After a reboot or a crash of the game, resume a suite from the last finished check. | Test author | could |

### Control

| ID | Requirement | For | Priority |
|---|---|---|---|
| CR-10 | Run code on a chosen machine (the server, one player by name, all clients or a headless client) and get the result back as data. | Test author | must |
| CR-11 | Build a scene: place objects, units and vehicles, set terrain position, time and weather, and move a player to an exact spot facing an exact way. | Test author | must |
| CR-12 | Make a player act through the mod's real path: use its actions and fire its key handlers. | Test author | must |
| CR-15 | Send key input to a game client when a check's promise is about the key itself, without taking over the machine's mouse and keyboard. | Test author | should |
| CR-13 | Act as a cheating client: send forged, repeated or out-of-role requests and events from one machine. | Test author | could |
| CR-14 | Simulate failures: kill a client, add lag or packet loss, stall a frame, make a unit unconscious, kill and respawn it. | Test author | could |

### Observation

| ID | Requirement | For | Priority |
|---|---|---|---|
| CR-20 | Wait for a condition on any machine, with a ceiling, never a fixed sleep. | Test author | must |
| CR-21 | Collect the mod's events and the engine's events per machine, with game time and real time. | Test author | should |
| CR-22 | Read the same value on several machines and compare them, such as whether every client agrees with the server. | Test author | must |
| CR-23 | Report every script error, on every machine, traced to the step that caused it. | Test author, mod developer | must |
| CR-24 | Keep each machine's log with the run, filterable. | Test author | must |
| CR-25 | Offer a screenshot check: a check that takes screenshots at chosen steps and returns them as its output, with what it expected to show, for a person or an AI agent to interpret. It has no pass or fail of its own. | Test author, manual tester | could |
| CR-26 | Measure performance: frame time, and the cost of a mod function on the server and on a client. | Test author | could |
| CR-27 | Ask a running game what is going on, at any time, without stopping it: for a person or an agent diagnosing a stall. | Mod developer, test author | must |

### Reliability

| ID | Requirement | For | Priority |
|---|---|---|---|
| CR-31 | Give every check one of three outcomes, pass, fail or invalid, with the reason. | Test author | must |
| CR-30 | Watch the session's health (the game clock moves, the frame rate is sane, every machine answers) and report a run that fails it as invalid, not failed. | Test author | should |
| CR-32 | Repeat a check N times on request and report how often each outcome came up. | Test author | should |
| CR-33 | Prove a check can fail: run it against a build where its promise is broken and confirm it fails. | Test author | could |
| CR-34 | Measure a baseline of the game's own behaviour under the same conditions as the check that compares against it. | Test author | could |

### Speed

| ID | Requirement | For | Priority |
|---|---|---|---|
| CR-40 | Run a single check, a list of checks, or the checks that name a story or a finding. | Test author, mod developer | must |
| CR-41 | Choose the setup per check (singleplayer or dedicated, which mods loaded) and run each check only in the setups it needs. | Test author | should |
| CR-42 | Report the time of every step, so a slow check shows. | Test author | should |

### Authoring and reporting

| ID | Requirement | For | Priority |
|---|---|---|---|
| CR-50 | Write a check in SQF, the language mods are written in, against an API shaped like Playwright's: steps, waits and assertions, readable without knowing the framework's internals, named with the story or requirement ID it proves. A test author needs no other language. | Test author | must |
| CR-51 | Write one result file per run: the setup, the mod build, each check's outcome and timing, and the evidence behind it. | Test author | must |
| CR-52 | Put a manual tester at the start of a situation, set up by the framework, with a list of what to look for. | Manual tester | should |
| CR-53 | Let an AI agent use every capability above as tools, not only through files and logs. | Test author, mod developer | must |

### Safety

| ID | Requirement | For | Priority |
|---|---|---|---|
| CR-60 | Accept control only from the local machine, with a token per run. | Everyone | must |
| CR-61 | Refuse to run with BattlEye on or against a public server. | Everyone | should |

## 6. Key scenarios

### When things go as planned

1. A test author starts a dedicated server with two clients, places a crate in front of one player, makes the player open it through the mod's action, and asserts that the item lands in the player's inventory on the server and on both clients.
2. A mod developer changes one function, reloads it into the running session, and reruns the three checks a finding names. The verdict comes back in seconds.
3. A test author sets up a scenario (a squad on a slope at dusk) and checks that the mod's behaviour holds in singleplayer and on a dedicated server, from one check.
4. An AI agent, stuck on a failing check, asks the running game for the state of the player and the objects around it, and finds the cause without a restart.
5. A test author runs the full suite locally and walks away. It leaves one result file, its logs, and nothing running.

### When things go wrong

1. A check's condition is never met. The wait ends at its ceiling, and the check fails with the last observed value and the state on each machine.
2. The mod throws a script error on one client only. The report names the client, the error and the step that caused it.
3. The game crashes or stops answering in the middle of a check. The check is reported invalid, not failed, with the reason.
4. A required mod is missing or the wrong version. The run stops before the game starts and says which mod and which version.
5. A check passes once in three runs. Repeating it shows the spread, and it is reported as flaky, not as a pass.
6. A run is stopped by hand or the runner crashes. The next run finds no game process, port or file left behind from it.
7. A client drops while it holds an object the mod tracks. The check asserts what the server and the other clients do about it.

## 7. Constraints

- Arma 3 only, in its current version, on Windows. The dedicated server may run on Windows or Linux.
- Runs on a developer's machine that has Arma 3 and its mods installed through Steam, logged in to an account that owns the game.
- Clients render a window. A machine that runs clients needs a display, or a remote stream such as Shadow.
- The framework never takes over the machine's mouse and keyboard, so a person can keep working while a suite runs.
- BattlEye is off for every test run.
- Checks are written in SQF. The framework's own internals may use whatever the game requires to talk to it from outside, but a test author never needs another language.
- The game's EULA and Bohemia Interactive's rules for tools and mods apply.
- Checks must not depend on the mod's source. The framework may load the mod and call its public functions, events and actions.
- The framework is for non-commercial mod testing only. The Arma 3 EULA allows the game's use for entertainment and recreation, and Bohemia's tools license names testing non-commercial content as its purpose.
- The project is private. It may be open-sourced later, so code copied from another project must carry a license that allows that. Reading any project, whatever its license or lack of one, and writing our own implementation of its ideas is allowed.
- The project is named Shakedown, after a ship's first test voyage. Its name contains no Bohemia Interactive trademark; "Arma 3" appears only to say which game it tests. The repository carries the name before anything is published.
- The extension is written in Rust with `arma-rs`, the library Arma extensions are commonly built with.
- Every dependency version is looked up at install time, not recalled.

## 8. Assumptions and dependencies

- We build our own framework. Existing open-source Arma tools for remote code execution, test harnesses and MCP servers are a source of code and ideas, where their licenses allow it. Research lists them with their license and what each one is worth taking.
- The game can run code sent from outside it while a mission runs, through an extension or a debug channel. Earlier research found several tools that do this. Research confirms which approach holds up.
- CBA is available in every setup that tests a CBA-based mod, and its events and key handlers can be driven from code.
- Key input can be sent to one game window without taking over the desktop. Research confirms it.
- Screenshots can be taken from inside the game on a client that renders.

## 9. First release

The first release is the MVP. It is done when a test author can, on one Windows machine:

- start a singleplayer session and a dedicated server session with two clients, and keep each alive across many checks (CR-01, CR-02);
- write a check in SQF that builds a scene, makes a player act through a mod's real path, waits for the outcome and asserts it on every machine (CR-10, CR-11, CR-12, CR-20, CR-22, CR-50);
- reload a mod's changed code and rerun a check without a restart (CR-03);
- get pass, fail or invalid with the reason, script errors traced to their step, and one result file per run (CR-23, CR-31, CR-51);
- do all of this from an AI agent's session as well as from the command line (CR-27, CR-53);
- and the run leaves nothing running (CR-05).

The MVP is proven on `arma-grappling-hook`: a set of its existing acceptance checks, rewritten for the framework, runs in singleplayer and on a dedicated server with two clients, against one session per setup.

Everything marked `should` or `could` waits until the MVP works.

## 10. Open questions

None open.
