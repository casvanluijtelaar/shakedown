---
type: Plan
title: Implementation plan
description: The phases of Shakedown's build, the order of its milestones and the rules behind that order, where each spike lands, which milestone completes each must and delivers each should requirement of the PRD, where the could requirements go, and which deferred work a milestone takes up.
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# Implementation plan

This plan turns the specification and the user stories into milestones. Each milestone is one Milestone concept in [milestones/](milestones/), with its goal, demo, stories and exit criteria; its state lives there, not here. This file holds the phases, the order of the milestones and the rules behind the order.

## Phases

| Phase | What it holds | Milestones |
|---|---|---|
| 1 | The MVP of PRD section 9 and nothing more: every `must` requirement and every user story, each story in exactly one milestone. It ends with release 0.1.0. | M0 to M10 |
| 2 | The `should` requirements: `MCH-01` (`CR-04`), `ENV-01` (`CR-06`), `ACT-04` (`CR-12`), `ACT-05` with spike `SPK-11` (`CR-15`), `EVT-01` (`CR-21`), `HLT-01` (`CR-30`), `OUT-02` (`CR-32`), `RUN-07` (`CR-41`), `TIM-01` (`CR-42`), `MAN-01` (`CR-52`) and `SEC-05` (`CR-61`); and the deferred work the M10 port and the test rounds showed matters: late joiners (`DEF-04`), shared helper code (`DEF-27`, `API-06`), a reload without a game restart (`DEF-21`), a hung client and a failed start's reason (`DEF-28`, `DEF-29`, `SES-10`). It ends with the first public release, 0.2.0 (`CR-70`, `DEF-30`). | M11 to M19 |
| 3 | The `could` requirements: `RSM-01` (`CR-07`), `CHT-01` (`CR-13`), `FLT-01` (`CR-14`), `SHT-01` (`CR-25`), `PRF-01` (`CR-26`), `OUT-03` (`CR-33`) and `OUT-04` (`CR-34`). | Unplanned; placed coarsely below |

Phase 2 is ordered by what use on arma-grappling-hook shows matters (PRD section 2, "How we build it"): the M10 port, its findings, the deferred work they raised, and what that repository's own plan asks of the framework next. Its stories are `US-066` to `US-098` (no `US-079`). Phase 3's stories are written when a milestone takes a `could` up.

## The order

| # | Milestone | Demo |
|---|---|---|
| M0 | [Walking skeleton](milestones/m00.md) | `shakedown run --setup sp smoke` starts Arma 3 in singleplayer, runs one SQF check with steps that passes, writes `result.json` and stops the game. |
| M1 | [A grappling-hook check in singleplayer](milestones/m01.md) | arma-grappling-hook's `US-009`, rewritten, passes in singleplayer: a placed rope, `p1` on it through the mod's action, the let-go keybind pressed, `p1` off the rope. |
| M2 | [Dedicated sessions](milestones/m02.md) | arma-grappling-hook's freed-rope check passes on a dedicated server with two clients, which agree with the server that the rope is free. |
| M3 | [Script errors and logs](milestones/m03.md) | A grappling-hook check made to fail shows its step, each machine's last value, log excerpt and script errors, and its run folder holds no Steam ID or name. |
| M4 | [Agent tools](milestones/m04.md) | An AI agent in Claude Code starts a session, runs the grappling-hook check, reads its result and inspects `p1`, all through MCP tools. |
| M5 | [A session that outlives its client](milestones/m05.md) | A dedicated session started from one Claude Code session is used by the next, and a killed runner leaves no game process within 10 s. |
| M6 | [Reload by restart](milestones/m06.md) | A changed grappling-hook function is reloaded by a game restart in one step into a running dedicated session, and the rerun check sees it. |
| M7 | [Reload inside the mission](milestones/m07.md) | A reload asked during a run is refused, and a check tagged with a finding sees a reloaded fixture function on every machine; a reload inside the mission waits for `DEF-21`. |
| M8 | [Walk away](milestones/m08.md) | One command runs the whole grappling-hook suite in its own dedicated session, and a run stopped with Ctrl+C still leaves a result and nothing running. |
| M9 | [Invalid, not failed](milestones/m09.md) | A client suspended or ended during a check makes it `invalid` naming the client, and `inspect` and the result still say what happened. |
| M10 | [The grappling-hook suite](milestones/m10.md) | arma-grappling-hook's rewritten M00 checks run in singleplayer and on a dedicated server with two clients, one command and one session per setup. |
| M11 | [Late joiners](milestones/m11.md) | arma-grappling-hook's `US-009-1-freed` and `MP-01-drawn` run with their late-joiner cases, `p3` joining after the rope is up, with the PowerShell suite's verdicts. |
| M12 | [Shared helpers and mod events](milestones/m12.md) | arma-grappling-hook's rope checks run with the rope kit in the library folder and their event lists from recorded events. |
| M13 | [Every setup, repeated](milestones/m13.md) | `run --each-setup --repeat 5` runs arma-grappling-hook's suite in all three setups and shows `US-064-1`'s spread in `sp-cba-ace`. |
| M14 | [Reload without a game restart](milestones/m14.md) | A changed arma-grappling-hook function reaches a running session by a mission restart, the rerun check's verdict under 30 s from the save in singleplayer, and every machine of a dedicated session with the server kept, as `SPK-19` found. |
| M15 | [Hand over to the manual tester](milestones/m15.md) | A check puts `p1` on a rope 10 m up and hands over, with three things to look for in `p1`'s window, and the session stays up. |
| M16 | [A start that explains itself](milestones/m16.md) | Every gap of a start named at once, a wrong ACE version refused, a hung client failing the start at the start ceiling, its reason kept. |
| M17 | [ACE interaction and real keys](milestones/m17.md) | `p1` gets on a rope through ACE interaction, and a real let-go key sent to `p1`'s window in the background takes it off. |
| M18 | [The full health watch](milestones/m18.md) | A client starved of CPU makes its check `invalid` / `unhealthy` naming its frame rate. |
| M19 | [The first public release](milestones/m19.md) | On a Windows machine with no build tools, `winget install` gives `shakedown` 0.2.0, which runs a passing check, and the one-line script refuses an upgrade while its runner runs. |

## The rules behind the order

1. **MVP first.** Phase 1 is PRD section 9 and nothing more. Every milestone takes the shortest path to its demo; what no story of the milestone promises waits, and goes into [deferred/](deferred/) when it matters.
2. **A walking skeleton first.** M0 builds every part of the chain thin, so every later milestone grows a part that already works instead of joining parts late.
3. **Singleplayer before dedicated.** A singleplayer session is one process and the fastest start, so the check API and the reset are proven there (M1) before machines are added (M2).
4. **A real grappling-hook check as soon as the API allows.** M1's demo is an arma-grappling-hook check, and M1 builds exactly what it needs: the scene, the mod's action, its CBA keybind, waits, assertions and the reset. Every later demo runs on arma-grappling-hook where a story lets it, and on a fixture mod where only a fixture can show the behaviour.
5. **Multiplayer first among the three that follow.** arma-grappling-hook's promises are about server authority, and its existing checks run on a dedicated server with clients, so dedicated sessions (M2) come before the MCP server and the restart reloads.
6. **A verdict explains itself before more ways to get one.** A pass means no script error on any machine (`DM-02`), so script errors and logs (M3) come before anything that runs more checks or runs them faster.
7. **Tools before reload.** arma-grappling-hook's checks are written and run by AI agents in its repository. From M4 they drive the framework as tools, and every later command lands with its tool, so no milestone waits on a tool layer. Without a reload the loop still works by stopping and starting the session; the reloads only shorten it. M5 follows M4 at once, because agents end their sessions and the game session has to survive that.
8. **A restart before a reload inside the mission.** arma-grappling-hook's functions are CBA `PREP` functions, final once compiled, and the game holds every mod file it has read (`SPK-08`), so a change to them reloads by a game restart. The restarts (M6) therefore come before level A (M7), which needs a changed file to reach a running game (`DEF-21`).
9. **Walking away, then trust, then the proof.** A suite run unattended needs a run that ends whatever happens (M8). The environment's failures must be `invalid`, not `fail` (M9), before the port (M10), whose runs `NFR-RL-01` judges.
10. **A story lands where its last criterion can be met.** A story whose criteria name an MCP tool lands in M4 or later, one that names a dedicated session in M2 or later, one that names a reload in M6 or later. A failure story lands with its feature story where both can; the pairs that split are listed below.
11. **Infrastructure in the first milestone that needs it, never ahead.** `skd_fnc_useAction` is built in M1 for its demo, while its stories land in M2; `status` comes in M2 for `US-004`; the `logs` command in M3 and its tool in M4; the session state `broken` in M6; each fixture behaviour with the first story or demo that uses it.
12. **Spikes before the work that depends on them.** Each spike of `spec/06-open-questions.md` runs in the milestone that first needs it, before the work that depends on it, or ahead of it as rule 18 says, and its result goes into `research/` with the spec edited in place. Where its pass condition spans a setup or feature that comes later, that part runs in the milestone that brings it, which names it.
13. **Each milestone ends on the test machine.** Its last exit criterion is the owner running its demo on the Windows test machine. Where a demo needs a look in the game, the session sets up the situation and tells the owner what to look for.
14. **After the MVP, use orders the work.** Each place in phase 2 is grounded in the M10 report, its findings, a deferred concept, or what arma-grappling-hook's own plan needs next, never in taste. What blocks retiring arma-grappling-hook's PowerShell suite comes first (M11), then what the port had to copy or measure by hand (M12, M13), then the gaps against a PRD goal (M14 for G2), then what makes the owner's and the test rounds' work cheaper (M15, M16), then what arma-grappling-hook needs only in its later milestones (M17), then what no use has shown yet (M18), and last the release that lets a test author install all of it without building it (M19), which changes none of the framework's code and so is specified beside M16 to M18. A requirement whose use does not show moves down; one that use shows matters moves up.
15. **One failure story per two feature stories, after the MVP.** A phase 2 milestone has at least one story of another kind for every two `feature` stories it holds, rounded up, each for the way a feature of the milestone most plausibly fails (the owner's decision of 2026-11-13). `malicious intent` stories still wait for the `CR-13` work.
16. **Deferred work rides with the milestone whose path it lies on.** A deferred concept is taken up by a phase 2 milestone when it is that milestone's demo (`DEF-04`, `DEF-30`), removes what the demo works around (`DEF-27`), or lies in the code the milestone changes (`DEF-28`, `DEF-29`). The rest stay deferred until use shows they matter.
17. **A patch version per milestone, a minor version per phase.** As in phase 1, each milestone of a phase raises the patch version and the phase's last milestone raises the minor: M11 to M18 are 0.1.1 to 0.1.8, and M19 is release 0.2.0 (the owner's decisions of 2026-11-13 and 2026-11-21).
18. **A spike that decides whether a milestone is built runs as a research round ahead of it.** `SPK-19` runs beside M11, so M14 is built against its result or closed without work before its place comes (the owner's decision of 2026-11-13).

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
| `SPK-08` | M6 | Yes, if a changed file does not reach a running machine: every `.sqf` change is level `game-restart` (`RLD-02`, `P-08`, `US-054`, `US-055`, `US-057`, `FIX-03`). Its measurement sets `NFR-PF-02`. |
| `SPK-09` | With `DEF-32`, if it is taken up | No: mod code keeps no level inside the mission, and a changed `.sqf` reloads by a mission restart (`SPK-19`). |
| `SPK-10` | M1 | No. |
| `SPK-11` | M17 | `ACT-05`, a `should`, is dropped. |
| `SPK-12` | M1 | Yes: in singleplayer a net id in a result is the agent's own id (`DM-13`). |
| `SPK-13` | M3 | Yes: `logs` asked during a check returns lines only up to that check's start. |
| `SPK-14` | M10 | Only the default numbers of `NFR-PF-06`. |
| `SPK-15` | M9 | Yes: the clock watch reports without changing outcomes, and `US-059` waits. |
| `SPK-16` | M9 | The fixture contract's crash changes; the stories hold. |
| `SPK-17` | M9 | Yes: `FIX-02` is dropped, and `US-059` is accepted by the owner cutting the Shadow stream. |
| `SPK-18` | M11 | Only if a client cannot rejoin under its name: rejoining waits, and `MCH-01` and `US-067` say so. Without a clean leave, a leave ends the process; the promise holds. |
| `SPK-19` | A research round beside M11, ahead of M14; passed (research `31-spike-reload-release.md`) | Yes: M14 builds nothing, every mod `.sqf` stays level `game-restart`, and `DEF-21`'s third option goes to the owner. |
| `SPK-20` | M16 | Yes: `SES-10`'s connect ceiling is dropped and `DEF-28` stays open; the reason in `session.json` holds. |
| `SPK-21` | M17 | No: the framework runs the ACE action's condition and statement itself. |
| `SPK-22` | M18 | Yes: frame rate and answer time show in `status` and flag no check. |

## Which milestone completes each must requirement of the PRD

A requirement of the PRD is complete in the milestone that delivers the last story its `must` requirements need. Parts of it work earlier, as the note says.

| CR | Requirement | Complete in | Note |
|---|---|---|---|
| `CR-01` | Start a session once, run many checks | M5 | Singleplayer in M0 and M1, dedicated in M2; surviving its client in M5. |
| `CR-02` | Reset between checks | M9 | In place in M1 and M2, by mission restart in M6; a reset that cannot complete in M9. |
| `CR-03` | Reload at the lowest level | M7 | Game restart in M6; a changed `.sqf` by a mission restart in M14, the game made to let go of the file (`SPK-19`, `DEF-21`); inside the mission waits with `DEF-32`. |
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

## Which milestone delivers each should requirement

| CR | Requirement | Delivered in | Note |
|---|---|---|---|
| `CR-04` | Join, leave, rejoin | M11 | Game clients only; restarting the server waits (`DEF-31`). |
| `CR-06` | Check the environment before a run | M16 | `SPK-20` failed, so `SES-10` keeps the reason of a failed start without a connect ceiling (`DEF-28`). |
| `CR-12` | ACE interaction actions (`ACT-04`) | M17 | |
| `CR-15` | Real key input | M17 | Only if `SPK-11` passes; otherwise dropped. |
| `CR-21` | The mod's and the engine's events | M12 | CBA events and mission events. |
| `CR-30` | The rest of the health watch | M18 | The clock watch since M9. |
| `CR-32` | Repeat a check | M13 | |
| `CR-41` | Setups per check, one command for all | M13 | The header's skipping since M0. |
| `CR-42` | The time of every step | M13 | Recorded since M0; flagged in M13. |
| `CR-52` | Hand over to a manual tester | M15 | |
| `CR-61` | No BattlEye, no other server | M16 | By construction since M0 (`NFR-LG-02`); a check that sends a client elsewhere in M16. |
| `CR-70` | Install a release without building it | M19 | Windows only, by winget, the one-line script or the zip by hand; macOS and Linux build with cargo (`NFR-PT-02`). |

## Where the could requirements go

Phase 3 is not planned yet; its stories are written when a milestone takes a requirement up. What arma-grappling-hook's plan shows today:

- **`FLT-01` failure simulation** comes first, after M17: arma-grappling-hook promises that a climber who falls unconscious or dies is released (its `CLB-05`), that a rope whose holder crashes is freed (its `MP-02`) and that actions work after a respawn (its `CTL-03`), and its ACE milestone demos a climber knocked out by ACE. Lag and packet loss wait inside it until a check needs them.
- **`PRF-01` performance with `OUT-04` baseline** come next, before arma-grappling-hook's first release on a server: its M06 demo holds 20 ropes "at the frame rate of the same mission without ropes", a performance measure against a baseline, and its fall checks already compare against the game's own fall in the same check.
- **`CHT-01`, `SHT-01`, `OUT-03` and `RSM-01` wait for use.** No check of the port asked for a cheating client, a screenshot, a proof that a check can fail, or a resumed run; `MAN-01` (M15) covers the owner's look, and the framework's own tests prove their checks can fail by hand.

## Deferred work

Work found while planning that no story promises is in [deferred/](deferred/). Phase 2 takes up `DEF-04` (M11), `DEF-27` (M12), `DEF-21` (M14), `DEF-29` (M16) and `DEF-30` (M19); `DEF-28` stays open after `SPK-20` failed in M16. The rest stay deferred:

- `DEF-26` (`skd_fnc_pressKey` returns no handler value) waits for a second mod that needs it; M17's real keys read the pass-through from the game.
- `DEF-31` (restarting the server during a session) waits for a mod's story that needs it.
- `DEF-24` (ceilings count a paused runner), `DEF-14`, `DEF-22`, `DEF-23` and `DEF-25` (the reset around a player's death and a stop), `DEF-01`, `DEF-02`, `DEF-03`, `DEF-07`, `DEF-08`, `DEF-11`, `DEF-13`, `DEF-15` to `DEF-20`: no run of the port met them; each moves into a milestone when use shows it matters.
