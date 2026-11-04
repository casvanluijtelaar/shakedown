---
type: Research
title: Gaps between the PRD and the research
description: The holes the PRD leaves that the research shows matter, the places where a requirement conflicts with what the engine allows, and the open questions for the owner, each with a recommendation.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T23:45:00+01:00 }
sources:
  - { id: prd, resource: "spec/client-requirements.md", read: 2026-11-04 }
  - { id: harness, resource: "research/10-prior-harness.md", read: 2026-11-04 }
  - { id: tools, resource: "research/11-arma-tools.md", read: 2026-11-04 }
  - { id: remote, resource: "research/12-remote-execution.md", read: 2026-11-04 }
  - { id: reload, resource: "research/13-hot-reload.md", read: 2026-11-04 }
  - { id: sessions, resource: "research/14-sessions.md", read: 2026-11-04 }
  - { id: input, resource: "research/15-player-input.md", read: 2026-11-04 }
  - { id: observe, resource: "research/16-observation.md", read: 2026-11-04 }
  - { id: health, resource: "research/17-health.md", read: 2026-11-04 }
  - { id: testtools, resource: "research/18-test-tools.md", read: 2026-11-04 }
  - { id: agent, resource: "research/19-agent-tools.md", read: 2026-11-04 }
  - { id: legal, resource: "research/20-licenses-and-rules.md", read: 2026-11-04 }
---

# Gaps between the PRD and the research

This file is the bridge to the specification. It compares the PRD[^prd] with the research and lists what the spec has to settle. The spec is written from the PRD, this file and the owner's answers to section 3.

## 1. Holes

Places where the PRD leaves a decision open that the research shows matters.

| # | Hole | What the research shows | Recommendation |
|---|---|---|---|
| H1 | **What a session is.** CR-01 says "start a session once" but not what keeps it alive. | `-autotest` fixes its mission list at start and exits after it. Mission changes with clients connected broke about one time in two. Nothing runs before a mission is loaded[^sessions][^harness][^tools]. | A session is one long framework mission per setup, which takes checks while it runs. It is ready when the framework agent answers on every machine of the setup. |
| H2 | **What a reset guarantees.** CR-02 says one check cannot affect the next, but not how. | No engine command resets a mission. `EntityCreated` can track what a check creates. `deleteVehicle` completes on the next frame and cannot delete players[^sessions][^input]. | The framework deletes everything a check created, and restores the players' position, health, loadout, time and weather. A check that changes anything else, such as global variables or mod settings, is tagged to run in a fresh mission. |
| H3 | **Which machine is which.** CR-10 names "one player by name". | Clients share one Steam account; the prior harness tells them apart by profile name, `p1` to `pN`[^harness][^sessions]. | Machines are named `server`, `p1`..`pN` and `hc1`..`hcN` in every setup, singleplayer included (`p1`). Checks address them by that name. |
| H4 | **The default ceilings.** CR-20 asks for a ceiling on every wait, but names no default. | Playwright waits 5 s per assertion and 30 s per test. A crashed client shows on the server only after about 15 s. A late joiner's `time` reads 0 for about 2.5 s[^testtools][^sessions]. | 10 s per wait, 120 s per check, both settable per call. Spikes set the final values. |
| H5 | **Two clocks.** The PRD speaks of a ceiling, not of which clock. | Mission time stops when the game pauses or a rendering client's stream is cut; `diag_tickTime` keeps counting[^observe][^health]. | Two ceilings: real time inside the game, wall clock in the runner. A wait that ends on the runner's ceiling is `invalid`, not `fail`. |
| H6 | **Who owns the game processes.** CR-05 and CR-53 together. | Claude Code ends an MCP server's whole process tree when the session stops[^agent]. | A long-lived runner owns every game process in a job object. The command line and the MCP server are thin clients of it. A session survives the agent session that started it. |
| H7 | **ACE interactions.** CR-12 says "use its actions". | ACE has no public way to perform an interaction by its path; doing it means repeating what ACE's own key-up code does through its internals[^input]. | Support `addAction` actions and CBA keybinds in the MVP. ACE interactions are a `should`, pinned to the ACE version a run reports. |
| H8 | **Personal data in logs.** CR-24 keeps every machine's log. | RPTs and extension calls carry Steam IDs and names, which count as personal data[^legal]. | Every process runs with a neutral `-name` and its own profile folder. The result file and stored logs are redacted. Run folders stay out of git. |
| H9 | **The player-hosted setup.** The PRD names singleplayer and dedicated only. | No source shows how to start a player-hosted game unattended[^sessions]. | Leave it out of the framework until a spike shows a way. `arma-grappling-hook` keeps its own plan for it. |
| H10 | **A Linux server.** Section 7 allows a Linux dedicated server. | The extension can build for Linux, but the test machine is Windows[^remote]. | Windows only for the MVP. A Linux server is deferred, with the extension written so it builds for both. |
| H11 | **Language of the runner.** The PRD fixes SQF for checks and Rust for the extension, not the runner. | The runner owns processes, job objects, sockets and the MCP server on Windows[^health][^agent]. | Rust, the same as the extension: one toolchain, one binary for the runner, the command line and the MCP server. Test authors still write only SQF. |

## 2. Conflicts

Places where a requirement conflicts with what the engine allows, or with another requirement.

| # | Conflict | Recommendation |
|---|---|---|
| C1 | **CR-03 against CBA's function cache.** CR-03 asks to reload the mod's changed code "without a restart". CBA's `PREP` functions, which `arma-grappling-hook` uses, are cached final and change only at a mission restart. New files, config and the extension itself need a game restart[^reload]. | Reword CR-03: the framework reloads a change at the lowest level it allows (in the mission, a mission restart inside the running session, or a game restart) and reports the level and why. A mission restart is still not a game restart. |
| C2 | **G2's 30 seconds against multiplayer.** A mission restart with clients kept connected is not reliable, and restarting the clients costs about 25 s[^harness][^sessions]. | Set G2 at 30 s for an in-mission reload in singleplayer and on a dedicated server, and measure a mission-restart reload by spike (HR-5) before setting its target. |
| C3 | **CR-15 against "never take over the desktop".** The one tool that sends real keys to Arma uses `SendInput`, which goes to the foreground window. No public source shows key input reaching a background game window[^input][^tools]. | Keep CR-15 a `should`, with a spike on posted window messages and in-process injection. If both fail, CR-15 is dropped: handlers cover every key path. |
| C4 | **Scenario 6.7 against CR-14.** The scenario of a client dropping while it holds an object needs failure simulation, which is a `could`. | Keep the scenario, and mark it as the target of CR-14, after the MVP. Killing a client process is cheap and may be enough for it. |
| C5 | **CR-25 against `-profiles`.** A per-machine profile folder (H8) may make `screenshot` fail silently[^observe]. | Spike it. The fallback is a window capture from inside the game process, which works in the background. |
| C6 | **Rendering clients against the Shadow stream.** A cut stream stops the clock on rendering clients, and Shadow shuts the machine down without input[^health]. | A check that needs a player but no rendering runs on a dedicated server with headless clients where it can. The health watchdog reports a cut stream as `invalid`. |

## 3. Open questions for the owner

| # | Question | Options | Recommendation | Answer |
|---|---|---|---|---|
| Q1 | The new name of the project, without "Arma". | Any name. | Pick one before anything is published. The MVP does not wait on it. | Brainstorm a name; decide before publishing. |
| Q2 | The goal targets in PRD section 2. | The proposals: G1 80 percent of stories, G2 under 30 s, G3 no environment failures reported as `fail`, G4 one command for a dedicated session. | Accept them, with G2 as in C2. | Accepted. |
| Q3 | The default ceilings (H4). | 10 s per wait and 120 s per check, or other values. | 10 s and 120 s until spikes measure them. | Accepted. |
| Q4 | The runner's language (H11). | Rust, or another language. | Rust. | Rust. |
| Q5 | The reset contract (H2): is a tag for "needs a fresh mission" enough for checks whose changes the framework cannot see? | A tag; or a fresh mission per check by default. | A tag. A fresh mission per check costs the restart the framework exists to avoid. | A tag. |
| Q6 | ACE interactions (H7). | MVP, or `should` after it. | `should` after the MVP. | `should`, after the MVP. |

[^prd]: spec/client-requirements.md
[^harness]: research/10-prior-harness.md
[^tools]: research/11-arma-tools.md
[^remote]: research/12-remote-execution.md
[^reload]: research/13-hot-reload.md
[^sessions]: research/14-sessions.md
[^input]: research/15-player-input.md
[^observe]: research/16-observation.md
[^health]: research/17-health.md
[^testtools]: research/18-test-tools.md
[^agent]: research/19-agent-tools.md
[^legal]: research/20-licenses-and-rules.md
