---
type: Plan
title: Implementation plan
description: The phases of Shakedown's build, the order of its milestones and the rules behind that order, where each spike lands, and which milestone completes each must requirement of the PRD.
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# Implementation plan

This plan turns the specification and the user stories into milestones. Each milestone is one Milestone concept in [milestones/](milestones/), with its goal, demo, stories and exit criteria; its state lives there, not here. This file holds the phases, the order of the milestones and the rules behind the order.

## Phases

| Phase | What it holds | Milestones |
|---|---|---|
| 1 | The MVP of PRD section 9 and nothing more: every `must` requirement and every user story, each story in exactly one milestone. It ends with release 0.1.0. | M0 to M10 |
| 2 | The `should` requirements: `MCH-01` (`CR-04`), `ENV-01` (`CR-06`), `ACT-04` (`CR-12`), `ACT-05` with spike `SPK-11` (`CR-15`), `EVT-01` (`CR-21`), `HLT-01` (`CR-30`), `OUT-02` (`CR-32`), `RUN-07` (`CR-41`), `TIM-01` (`CR-42`), `MAN-01` (`CR-52`) and `SEC-05` (`CR-61`). | Unplanned |
| 3 | The `could` requirements: `RSM-01` (`CR-07`), `CHT-01` (`CR-13`), `FLT-01` (`CR-14`), `SHT-01` (`CR-25`), `PRF-01` (`CR-26`), `OUT-03` (`CR-33`) and `OUT-04` (`CR-34`). | Unplanned |

Phases 2 and 3 have no stories yet. They are planned once the MVP works end to end, in the order that use on arma-grappling-hook shows they matter (PRD section 2, "How we build it"): their stories are written then, and each becomes a milestone of its phase. The port in M10 already names the first candidate, late joiners ([deferred 04](deferred/04-late-joiners.md)).

## The order

| # | Milestone | Demo |
|---|---|---|
| M0 | [Walking skeleton](milestones/m00.md) | `shakedown run --setup sp smoke` starts Arma 3 in singleplayer, runs one SQF check with steps that passes, writes `result.json` and stops the game. |
| M1 | [A grappling-hook check in singleplayer](milestones/m01.md) | arma-grappling-hook's `US-009`, rewritten, passes in singleplayer: a placed rope, `p1` on it through the mod's action, the let-go keybind pressed, `p1` off the rope. |
| M2 | [Dedicated sessions](milestones/m02.md) | arma-grappling-hook's freed-rope check passes on a dedicated server with two clients, which agree with the server that the rope is free. |
| M3 | [Script errors and logs](milestones/m03.md) | A grappling-hook check made to fail shows its step, each machine's last value, log excerpt and script errors, and its run folder holds no Steam ID or name. |
| M4 | [Agent tools](milestones/m04.md) | An AI agent in Claude Code starts a session, runs the grappling-hook check, reads its result and inspects `p1`, all through MCP tools. |
| M5 | [A session that outlives its client](milestones/m05.md) | A dedicated session started from one Claude Code session is used by the next, and a killed runner leaves no game process within 10 s. |
| M6 | [Reload by restart](milestones/m06.md) | A changed grappling-hook function is reloaded by a mission restart into a running dedicated session, and the rerun check sees it. |
| M7 | [Reload inside the mission](milestones/m07.md) | A changed fixture function is recompiled in the running mission, and a check tagged with a finding sees it on every machine under 30 s later. |
| M8 | [Walk away](milestones/m08.md) | One command runs the whole grappling-hook suite in its own dedicated session, and a run stopped with Ctrl+C still leaves a result and nothing running. |
| M9 | [Invalid, not failed](milestones/m09.md) | A client suspended or ended during a check makes it `invalid` naming the client, and `inspect` and the result still say what happened. |
| M10 | [The grappling-hook suite](milestones/m10.md) | arma-grappling-hook's rewritten M00 checks run in singleplayer and on a dedicated server with two clients, one command and one session per setup. |

## The rules behind the order

1. **MVP first.** Phase 1 is PRD section 9 and nothing more. Every milestone takes the shortest path to its demo; what no story of the milestone promises waits, and goes into [deferred/](deferred/) when it matters.
2. **A walking skeleton first.** M0 builds every part of the chain thin, so every later milestone grows a part that already works instead of joining parts late.
3. **Singleplayer before dedicated.** A singleplayer session is one process and the fastest start, so the check API and the reset are proven there (M1) before machines are added (M2).
4. **A real grappling-hook check as soon as the API allows.** M1's demo is an arma-grappling-hook check, and M1 builds exactly what it needs: the scene, the mod's action, its CBA keybind, waits, assertions and the reset. Every later demo runs on arma-grappling-hook where a story lets it, and on a fixture mod where only a fixture can show the behaviour.
5. **Multiplayer first among the three that follow.** arma-grappling-hook's promises are about server authority, and its existing checks run on a dedicated server with clients, so dedicated sessions (M2) come before the MCP server and the restart reloads.
6. **A verdict explains itself before more ways to get one.** A pass means no script error on any machine (`DM-02`), so script errors and logs (M3) come before anything that runs more checks or runs them faster.
7. **Tools before reload.** arma-grappling-hook's checks are written and run by AI agents in its repository. From M4 they drive the framework as tools, and every later command lands with its tool, so no milestone waits on a tool layer. Without a reload the loop still works by stopping and starting the session; the reloads only shorten it. M5 follows M4 at once, because agents end their sessions and the game session has to survive that.
8. **A mission restart before a reload inside the mission.** arma-grappling-hook's functions are CBA `PREP` functions, final once compiled, so a change to them reloads only by a mission restart. Levels B and C (M6) therefore come before level A (M7), which serves functions that are not final.
9. **Walking away, then trust, then the proof.** A suite run unattended needs a run that ends whatever happens (M8). The environment's failures must be `invalid`, not `fail` (M9), before the port (M10), whose runs `NFR-RL-01` judges.
10. **A story lands where its last criterion can be met.** A story whose criteria name an MCP tool lands in M4 or later, one that names a dedicated session in M2 or later, one that names a reload in M6 or later. A failure story lands with its feature story where both can; the pairs that split are listed below.
11. **Infrastructure in the first milestone that needs it, never ahead.** `skd_fnc_useAction` is built in M1 for its demo, while its stories land in M2; `status` comes in M2 for `US-004`; the `logs` command in M3 and its tool in M4; the session state `broken` in M6; each fixture behaviour with the first story or demo that uses it.
12. **Spikes before the work that depends on them.** Each spike of `spec/06-open-questions.md` runs in the milestone that first needs it, before the work that depends on it, and its result goes into `research/` with the spec edited in place. Where its pass condition spans a setup or feature that comes later, that part runs in the milestone that brings it, which names it.
13. **Each milestone ends on the test machine.** Its last exit criterion is the owner running its demo on the Windows test machine. Where a demo needs a look in the game, the session sets up the situation and tells the owner what to look for.

## Where each spike lands

| Spike | Milestone | If it fails, does a promise change? |
|---|---|---|
| `SPK-01` | M0 (`p1`), M2 (server, client, headless client), M6 (across a mission restart) | No for a client or headless client: orders take another route. For `p1` there is no fallback, and M0 stops. |
| `SPK-02` | M0 | No. |
| `SPK-03` | M0 (`p1`), M2 (the other kinds) | Only where the token lives (`SEC-02`); no token reaches a command line or output either way. |
| `SPK-04` | M1 (singleplayer), M2 (dedicated) | Yes: what the reset cannot restore needs `@reset mission`, written into the check API contract. |
| `SPK-05` | M1 | Its second fallback, yes: a level-B reload in singleplayer becomes a game restart (`RLD-02`, `P-08`, `US-055`). |
| `SPK-06` | M5 | No. |
| `SPK-07` | M3 | Yes: on a machine kind where the handler is silent, errors come from the RPT, attributed by time. |
| `SPK-08` | M6 | No. Its measurement sets `NFR-PF-02`. |
| `SPK-09` | M7 | Yes: mod code gets no level A; `RLD-02`, `US-054` and `NFR-PF-01` are edited, with the owner's confirmation. |
| `SPK-10` | M1 | No. |
| `SPK-11` | Phase 2, with `ACT-05` | `ACT-05`, a `should`, is dropped. |
| `SPK-12` | M1 | Yes: in singleplayer a net id in a result is the agent's own id (`DM-13`). |
| `SPK-13` | M3 | Yes: `logs` asked during a check returns lines only up to that check's start. |
| `SPK-14` | M10 | Only the default numbers of `NFR-PF-06`. |
| `SPK-15` | M9 | Yes: the clock watch reports without changing outcomes, and `US-059` waits. |
| `SPK-16` | M9 | The fixture contract's crash changes; the stories hold. |
| `SPK-17` | M9 | Yes: `FIX-02` is dropped, and `US-059` is accepted by the owner cutting the Shadow stream. |

## Which milestone completes each must requirement of the PRD

A requirement of the PRD is complete in the milestone that delivers the last story its `must` requirements need. Parts of it work earlier, as the note says.

| CR | Requirement | Complete in | Note |
|---|---|---|---|
| `CR-01` | Start a session once, run many checks | M5 | Singleplayer in M0 and M1, dedicated in M2; surviving its client in M5. |
| `CR-02` | Reset between checks | M9 | In place in M1 and M2, by mission restart in M6; a reset that cannot complete in M9. |
| `CR-03` | Reload at the lowest level | M7 | Mission and game restart in M6, inside the mission in M7. |
| `CR-05` | Stop every process a run started | M8 | Job object in M0, recovery after the runner dies in M5. |
| `CR-10` | Run code on a chosen machine | M4 | Inside a check from M1, across machines in M2, `exec` in M4. |
| `CR-11` | Build a scene | M2 | Singleplayer in M1. |
| `CR-12` | Act through the mod's real path | M2 | The keybind and the action in M1. |
| `CR-20` | Wait with a ceiling | M9 | In-game ceilings in M1, `wait` in M4, the run's ceiling in M8, the runner's ceiling on a machine that stops answering in M9. |
| `CR-22` | Compare a value across machines | M4 | Agreement in M2, group results of `exec` in M4. |
| `CR-23` | Every script error, traced to its step | M4 | Inside checks in M3, from `exec` and outside checks in M4. |
| `CR-24` | Each machine's log with the run, filterable | M9 | Logs and redaction in M3, the tool in M4, a crashed machine's log in M9. |
| `CR-27` | Ask a running game what is going on | M9 | `inspect` and `status` in M4, on a broken session and a lost machine in M9. |
| `CR-31` | Pass, fail or invalid with the reason | M9 | `fail` from M0, `invalid` for a failed reset or reload in M6, a stopped run in M8, every environment failure in M9. |
| `CR-40` | Run one check, a list or a tag | M8 | Ids from M0, tags in M1, the full suite in its own session in M8. |
| `CR-50` | Checks in SQF, shaped like Playwright | M6 | Header and steps in M0, the check API in M1 and M2, `@reset mission` in M6. |
| `CR-51` | One result file per run | M9 | From M0; the time of each outcome's cause in M9. |
| `CR-53` | Every capability as AI agent tools | M9 | The MCP server in M4; each later capability lands with its tool; machine health in `status` in M9. |
| `CR-60` | Control only from the local machine, with a token | M5 | The token in M0, the process check and the audit in M5. |

`CR-30` is a `should`, but its clock watch (`HLT-02`) is a `must` and lands in M9.

## Story pairs that land apart

A feature story's failure stories land with it, except these, whose failure story lands where its own last criterion can be met:

- `US-017` (M2) and `US-018` (M1): the argument checks come with the API in singleplayer.
- `US-036` (M8) and `US-035` (M5): the runner's death belongs with its detached start.
- `US-040` (M6) and `US-038` (M0): a broken check is part of the skeleton; the edit loop needs `reload`.
- `US-042` (M4) and `US-043` (M9): a crashed machine's log needs the crash handling of M9.

## Deferred work

Work found while planning that no story of phase 1 promises is in [deferred/](deferred/).
