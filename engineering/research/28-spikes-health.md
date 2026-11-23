---
type: Research
title: "Spikes SPK-16, SPK-17 and SPK-15: crashes, a held clock and the clock watch"
description: "What a crash on demand through the fixture's extension does to each kind of game process, how singleplayer mission time is held while frames go on, and what each machine's heartbeat and clocks measured over 50 passing checks in singleplayer and on a dedicated session, with how fast a suspended and a killed client are seen, on the test machine in M9."
status: stable
generated: { by: "claude-code/developer", at: 2026-11-12T12:40:00+01:00 }
sources:
  - { id: tests, resource: "crates/shakedown/tests/spikes_m09.rs, spk_16_a_crash_on_demand_on_each_kind_of_game_process, spk_16_the_headless_client_after_its_crash, spk_17_mission_time_held_while_frames_go_on, spk_15_heartbeats_over_50_passing_checks and spk_15_a_suspended_and_a_killed_client_are_flagged, runs of 2026-11-12 on the Windows test machine, Arma 3 2.22 stable, the e2e setups sp-fixture and dedicated (a server, two clients and a headless client loading @skd_fixture)", read: 2026-11-12 }
  - { id: r17, resource: "research/17-health.md", read: 2026-11-12 }
---

# Spikes SPK-16, SPK-17 and SPK-15: crashes, a held clock and the clock watch

M9 ran `SPK-16`, `SPK-17` and `SPK-15` (`spec/06-open-questions.md`) on the Windows test machine, each before the work that depends on it, from the tests in `crates/shakedown/tests/spikes_m09.rs`[^tests]. Each test starts its sessions through the runner's own library, as `session start` does, and ends every process it started.

## SPK-16: a crash on demand

The fixture's extension `skdfx_x64.dll` (`FIX-01`) answers `"skdfx" callExtension ["crash", [delay]]` by starting a thread that sleeps `delay` seconds and then writes to address `0x10`, an access violation in native code that nothing in the extension handles. The test called `[0.5] call skdfx_fnc_crash` on singleplayer `p1` in one session, and on `p1` (a client), `hc1` and then `server` of one dedicated session, 5 rounds each, and watched the process handle for 10 s.

| Kind | Ended within 10 s | Exit code | Time from the call to the exit | Dumps in the profile folder |
|---|---|---|---|---|
| Singleplayer `p1` | 5 of 5 | 1 | 1.5 to 2.5 s | `.mdmp` (about 73 MB) and `.bidmp` (about 30 KB), 5 of 5 |
| Dedicated `server` | 5 of 5 | `0xC0000005` | 1.3 to 1.4 s | `.mdmp` (about 33 MB) and `.bidmp`, 5 of 5 |
| Client | 5 of 5 | `0xC0000005` | 1.7 to 1.8 s | `.mdmp` (about 73 MB) and `.bidmp`, 5 of 5 |
| Headless client | 1 of 5 | `0xC0000005` when it ended | 1.6 s in round 4; not within 10 s in the other four | `.mdmp` and `.bidmp`, 5 of 5, written within the 10 s |

The engine's crash handling ran on every kind: each process wrote both dumps into its own profile folder, named after the process's start time (`arma3_x64_<date>_<time>.mdmp`), and no dialog was seen. The job object's die-on-unhandled-exception limit (`STP-01`) lets no Windows error report hold a process.

A headless client sometimes stays up after its dumps are written. Two follow-ups crashed it with nothing crashed before it, four times: twice it ended after 1.7 s, and twice it stayed up for 61 s, its window answering and three of its threads gone after about 30 s, then ended with `0xC0000005`; crashed 2 s after a client, once more, it ended after 1.6 s. One further round is left out: its runner, built before the heartbeat reached the installed addon, took the headless client for lost and sent it nothing. What the engine waits for in those 60 s is not known.

So the pass condition held for three kinds and failed for the headless client. The fallback `SPK-16` names, ending the process with `ExitProcess` and a dump written by the harness with `MiniDumpWriteDump`, would replace the engine's own dumps, which every kind wrote, with dumps the engine never makes. The extension instead keeps the real fault and adds a watchdog: the faulting thread first starts a second thread that, 5 s after the fault, ends the process with `TerminateProcess` and exit code `0xC0000005` if it still runs. With the watchdog the whole spike ran again[^tests]:

| Kind | Ended within 10 s | Exit code | Time from the call to the exit |
|---|---|---|---|
| Singleplayer `p1` | 5 of 5 | 1 | 1.5 to 1.6 s |
| Dedicated `server` | 5 of 5 | `0xC0000005` | 1.3 s |
| Client | 5 of 5 | `0xC0000005` | 1.7 s |
| Headless client | 5 of 5 | `0xC0000005` | 1.6 to 1.8 s twice; 5.7 s three times, ended by the watchdog |

Every process left both dumps, 5 of 5 per kind. The 5 s leave the engine more than twice the time it took to write its dumps on every kind that ended by itself.

**Forces:** `FIX-01` and the fixture contract's `skdfx_fnc_crash` say what the engine does on each kind, and that the extension ends a process its crash handling leaves running 5 s after the fault. `P-09` takes the dumps as `.mdmp` and `.bidmp` in the profile folder, as it said. A crash is told from a stop by the runner: a process that ends without the runner having asked is `exited`, whatever its code.

## SPK-17: mission time held while frames go on

Two candidates held `time` on singleplayer `p1` for 8 s, each ended from an `EachFrame` handler gated on `diag_tickTime`: the engine's time acceleration set to 0 with `setAccTime 0` and back to 1, and the engine's pause, by opening the interrupt display `RscDisplayInterrupt` over the mission display and closing it. During each hold an `EachFrame` handler recorded a beat every second, as the heartbeat of `HLT-02` does, and the runner sent `exec` of `1 + 1` 2 s into the hold. Ten holds each[^tests]:

| Candidate | Mission time during 8 s of hold | Frames | `exec` of `1 + 1` | Beats | Rate after the hold | Pass |
|---|---|---|---|---|---|---|
| `setAccTime 0` | under 0.04 s | advanced, about 1,000 over the hold | answered in 0.01 to 0.02 s | 7 or 8, gap at most 1.01 s | 1.001 to 1.010 of real time | 10 of 10 |
| The engine's pause | 0.08 to 0.11 s | advanced | answered in 0.00 to 0.02 s | 7 or 8, gap at most 1.01 s | 1.000 to 1.006 of real time | 10 of 10 |

The mission time figures are what the test measured from before the hold to 0.5 s after its end, less the 0.5 s after. `accTime` read 0 after `setAccTime 0`, and `isGamePaused` was true under the interrupt display. Under both, the scheduled environment went on: the `exec` was spawned and answered at once.

**Forces:** `skdfx_fnc_holdClock` uses `setAccTime 0`, which held the clock tightest and touches no display. `FIX-02` stays, and says how the clock is held.

## SPK-15: the thresholds of the clock watch

The heartbeat of `HLT-02` was built first: from `ready` on, each machine's agent sends its clocks (`OBS-05`) from an `EachFrame` handler once a second of real time has passed, and the runner keeps the last 15 s of them per machine. The first test ran 50 passing checks (each waits 1.5 s and asserts that `time` is above 0) in a singleplayer session and 50 on a dedicated session with two clients and a headless client, and sampled every machine's last heartbeat from `status` every 100 ms[^tests]:

| Session | Machine | Heartbeats | Largest gap on arrival | Largest gap in the machine's real time | Smallest 5 s ratio of mission time to real time | Smallest 5 s frame advance | Flagged |
|---|---|---|---|---|---|---|---|
| Singleplayer | `p1` | 153 | 1.09 s | 1.01 s | 1.000 | 778 | never |
| Dedicated | `server` | 175 | 1.12 s | 1.02 s | 1.000 | 237 | never |
| Dedicated | `p1` | 175 | 1.02 s | 1.02 s | 0.997 | 291 | never |
| Dedicated | `p2` | 176 | 1.02 s | 1.01 s | 0.995 | 294 | never |
| Dedicated | `hc1` | 175 | 1.11 s | 1.02 s | 1.000 | 240 | never |

All 100 checks passed. The dedicated server and the headless client ran at about 47 frames a second, the rendering clients at about 58, singleplayer at about 155. The thresholds `HLT-02` set before the spike, a 5 s window, mission time under half of real time, a frame number that does not move, leave a wide margin: no window came near half, and no frame count near zero.

A suspended process sends no heartbeat at all, so the clock watch cannot judge its clocks; it is a stall (`P-07`), not an unhealthy machine that still answers. The runner therefore also takes a machine whose heartbeat has not come for 10 s as `lost` with reason `no-answer`, nine times the largest gap measured. The second test, 5 rounds on a dedicated session with two clients and a headless client, suspended `p2` with `NtSuspendProcess` by the process id `status` gives, and then ended `p1` with `taskkill /F`, with no run active[^tests]:

| What | Shown `lost` in `status` after | Within 15 s |
|---|---|---|
| `p2` suspended | 9.3 to 10.1 s, as `no-answer`, its last answer time in the message | 5 of 5 |
| `p1` ended | 0.4 to 0.6 s, as `exited`, counted from the start of `taskkill` | 5 of 5 |

Each time the session went `broken` with the machine's message, and every other process went on running.

The Shadow stream cut was not driven: nobody could cut the owner's stream during the spike. What research `17-health.md` reports of it, mission time standing still while frames go on at about 19 a second[^r17], is what `SPK-17`'s held clock reproduces, and the watch flags that in about 5 s; that the cut itself is flagged waits for the owner's run of `US-059`.

**Forces:** `HLT-02` keeps its thresholds, says when a flag clears and from when a clock counts as stopped, and adds the 10 s heartbeat gap; `P-07` takes the gap as its third way to see a stall. The clock watch changes outcomes, as `HLT-02` says, so the fallback is not needed.

[^tests]: `crates/shakedown/tests/spikes_m09.rs`, run with `cargo test -p shakedown --test spikes_m09 -- --ignored --nocapture --test-threads=1` after `python tools/install.py`, 2026-11-12.
[^r17]: research/17-health.md
