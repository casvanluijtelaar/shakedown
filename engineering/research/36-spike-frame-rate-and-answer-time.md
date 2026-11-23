---
type: Research
title: "Spike SPK-22: the thresholds of the frame rate and answer time watch"
description: "Each machine's frame rate over 5 s and its answer time to the runner's ping over 50 passing checks in singleplayer and 50 on a dedicated session with two clients and a headless client, and how fast a client starved of CPU is flagged, on a Windows machine in M18: no healthy machine came under 44 frames a second or above 0.03 s, and a client suspended 480 ms of every 500 ms was flagged by its frame rate in 3.6 to 5.6 s, 5 of 5, once mission time that fell behind on a starved machine is named by its cause, so the floor is 10 frames a second and the ceiling 2 s."
status: stable
generated: { by: "claude-code/developer", at: 2026-11-21T20:20:48Z }
sources:
  - { id: tests, resource: "crates/shakedown/tests/spikes_m18.rs, spk_22_frame_rate_and_answer_time_over_50_passing_checks and spk_22_a_starved_client_is_flagged, runs of 2026-11-21 on a Windows machine, Arma 3 2.22 stable, the e2e setups sp-fixture and dedicated (a server, two clients and a headless client loading @skd_fixture)", read: 2026-11-21 }
  - { id: r17, resource: "engineering/research/17-health.md", read: 2026-11-21 }
  - { id: r28, resource: "engineering/research/28-spikes-health.md", read: 2026-11-21 }
---

# Spike SPK-22: the thresholds of the frame rate and answer time watch

M18 ran `SPK-22` (`engineering/spec/06-open-questions.md`) on a Windows machine before it set the thresholds of `HLT-01`, from the tests in `crates/shakedown/tests/spikes_m18.rs`[^tests]. Each test starts its sessions through the runner's own library, as `session start` does, and ends every process it started.

## What was measured

The two measures were built first, with no threshold yet trusted:

- **The frame rate** is a machine's frames over the last 5 s of real time, from the frame number and `diag_tickTime` of its heartbeats (`HLT-02`), which the agent sends once a second from an `EachFrame` handler. Research 17 names `diag_fps`, the engine's average over the last 16 frames[^r17]; the heartbeats' own frame numbers give the average over the same 5 s the clock watch judges, and need nothing new from the agent.
- **The answer time** is how long a machine takes to answer the runner's ping. With each heartbeat of a `ready` machine that is not held, the runner sends a ping, one at a time, as an order: the extension announces it through its callback, the agent fetches it in a scheduled script and answers through the extension's send, the path every `exec` and check order takes. The answer time is how long the last answered ping took, or how long the ping not yet answered has waited, whichever is longer.

## Over 50 passing checks

Each check waits 1.5 s and asserts that `time` is above 0, as `SPK-15`'s did[^r28]. The test sampled every machine's `health` from `status` every 100 ms[^tests]:

| Session | Machine | Samples | Smallest 5 s frame rate | Mean frame rate | Largest answer time | Mean answer time | Flagged |
|---|---|---|---|---|---|---|---|
| Singleplayer | `p1` | 1,409 | 155.1 | 158.1 | 0.026 s | 0.005 s | never |
| Dedicated | `server` | 1,737 | 44.5 | 47.0 | 0.023 s | 0.021 s | never |
| Dedicated | `p1` | 1,737 | 57.9 | 58.7 | 0.021 s | 0.017 s | never |
| Dedicated | `p2` | 1,737 | 57.8 | 58.7 | 0.019 s | 0.017 s | never |
| Dedicated | `hc1` | 1,737 | 45.2 | 47.9 | 0.022 s | 0.021 s | never |

All 100 checks passed. The rendering clients ran at about 58 frames a second while a remote desktop showed the machine. In an earlier run of the same day, with no remote desktop showing it, `p2` ran 17.7 to 19.7 frames a second with its mission time at the rate of real time: the about 19 a second that research 17 measured while a remote desktop connection was cut[^r17], without the stopped clock it also reported.

## A client starved of CPU

On one dedicated session with two clients and a headless client, `p2` was starved five times, each until it was flagged, by suspending its process with `NtSuspendProcess` by the id `status` gives and resuming it, in a loop, and then left to run freely until its flag cleared[^tests]:

| Starving | Frame rate while starved | Answer time while starved | Flagged | Cleared after it ran freely |
|---|---|---|---|---|
| Suspended 180 ms of every 200 ms, `p2` at about 19 frames a second | 17.7 to 19.7, unchanged | 0.002 to 0.005 s | 0 of 5 within 30 s | never flagged |
| Suspended 450 ms of every 500 ms, `p2` at about 58 frames a second | 6.0 to 6.5 | 0.017 to 0.048 s | `frame-rate`, 5 of 5, after 5.0 to 5.5 s | 4.6 to 5.1 s |
| Suspended 450 ms of every 500 ms, `p2` at about 155 frames a second (the first run of `m18.rs`) | 19.1 | under 0.01 s | not within 16 s, 0 of 1 | never flagged |
| Suspended 480 ms of every 500 ms, `p2` at about 58 frames a second, mission time named before the frame rate | 2.4 to 4 over the starved steps | up to 0.5 s | `frame-rate` once, then `mission-time` four times, after 4.6 s | 4.6 to 5.1 s |
| The same, mission time that fell behind on steps under the floor named `frame-rate` | 2.4 to 4 over the starved steps | up to 0.5 s | `frame-rate`, 5 of 5, after 3.6 to 5.6 s | 4.6 s |

The first starving hardly touched a client that rendered at 19 frames a second: the game waits for most of each frame there, so 20 ms of each 200 ms is enough for its frames. The game likewise caught up on 50 ms of each 500 ms when it ran at 155 frames a second, which a dedicated client did on some starts of the day. Only 20 ms of each 500 ms took the frame rate under the floor at every rate seen.

At 480 ms of each 500 ms a second effect showed: mission time fell under half of real time within the first starved seconds, while the 5 s window still held frames from before the starving and its frame rate stayed above 10, so the clock watch named `mission-time` first. The engine caps the simulation step of each frame, so a machine with few frames loses mission time; the cause is the frame rate. The watch therefore names `frame-rate` when mission time crossed and the steps since it crossed ran under the floor, with the frame rate over those steps; a held clock (`SPK-17`) or a cut remote desktop connection keeps its frames and stays `mission-time`. With that rule all five rounds were flagged `frame-rate`. The answer time rose to about 0.5 s, a ping waiting out one suspension, far under its ceiling. No other machine was flagged in any round.

## Thresholds

- **The frame rate floor is 10 frames a second, one for every kind of machine.** Every healthy machine ran at 44 or more, and a rendering client at 18 to 20 with no remote desktop showing it, so 10 leaves a margin under the slowest healthy rate and stays under the 19 a second of a cut remote desktop connection, which the clock watch names by its stopped mission time (`US-059`). A starved client fell to between 2 and 7. Research 17 suggests a floor against each machine's own baseline, since a server with no players limits itself to 30 frames a second[^r17]; 10 sits under that too, so no baseline is needed.
- **The answer time ceiling is 2 s, held at every heartbeat of 5 s.** No answer of a healthy machine took more than 0.03 s, and of a starved one no more than 0.5 s, the length of one suspension, so 2 s is four times the slowest measured and still far under the 10 s of `P-07`'s heartbeat gap and the `exec` ceiling. A ceiling held over the whole window keeps one slow answer, such as the scheduled environment busy for a moment, from flagging a machine. No run made a machine's answers lag while its frames went on, so `answer-time` is proven by the runner's own tests without the game.

**Forces:** `SPK-22` passes: a starved client was flagged within 15 s, 5 of 5, and no healthy machine crossed either threshold. `HLT-01` takes the floor, the ceiling, the order in which the measures are named, mission time that fell behind on steps under the floor named by the frame rate, and when a flag clears; `INS-04` and the `status` tool show both measures; `DM-05`'s `unhealthy` names the machine and the measure. The fallback is not needed.

[^tests]: `crates/shakedown/tests/spikes_m18.rs`, run with `cargo test -p shakedown --test spikes_m18 -- --ignored --nocapture --test-threads=1` after `python tools/install.py`, 2026-11-21. The starving duty is `SKD_STARVE`, `<off ms>,<on ms>`, by default 480 and 20; the rows of the second table ran with the duty each names, and the 155 frames a second row is the first run of `crates/shakedown/tests/m18.rs`.
[^r17]: engineering/research/17-health.md
[^r28]: engineering/research/28-spikes-health.md
