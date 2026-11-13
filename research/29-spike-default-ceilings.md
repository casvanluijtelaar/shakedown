---
type: Research
title: "Spike SPK-14: the default ceilings"
description: "How long an action's effect takes to reach every machine of a dedicated session, measured 100 times, and how near the passing checks of the ported arma-grappling-hook suite and of the framework's own end-to-end runs came to the wait ceiling, the check timeout, the reset ceiling and the start ceiling, on the test machine in M10."
status: stable
generated: { by: "claude-code/developer", at: 2026-11-13T08:13:00-08:00 }
sources:
  - { id: tests, resource: "crates/shakedown/tests/m10.rs, spk_14_an_actions_effect_reaches_every_machine_within_the_default_wait_ceiling, cfg_01_rst_01_dm_14_m10_findings_in_singleplayer and cfg_01_rst_01_dm_14_m10_findings_on_a_dedicated_server", read: 2026-11-13 }
  - { id: port, resource: "arma-grappling-hook, branch shakedown/m10, acceptance/shakedown/ (its shakedown.toml and ten ported checks), run with the framework built from this repository's branch developer/m10-b", read: 2026-11-13 }
  - { id: r18, resource: "research/18-test-tools.md", read: 2026-11-13 }
---

# Spike SPK-14: the default ceilings

M10 ran `SPK-14` (`spec/06-open-questions.md`) on the Windows test machine, Arma 3 with CBA_A3 and ACE3 from the Steam Workshop, after the port of arma-grappling-hook's M00 checks. Its pass condition: the 10 s wait ceiling covers the slowest common wait, an action's effect replicated to every client of a dedicated session, in 95 of 100 runs, and no passing run of 50 checks comes within 5 s of a runner ceiling. It passed, so the defaults of `NFR-PF-06` stand as written: wait ceiling 10 s, check timeout 120 s, the runner's grace 10 s, start ceiling 180 s, reset ceiling 15 s. The defaults are those `research/18-test-tools.md` drew from other test tools[^r18]; this is the first measurement of them on real checks.

## The slowest common wait

The test `spk_14_an_actions_effect_reaches_every_machine_within_the_default_wait_ceiling`[^tests] starts a dedicated session with two clients and a headless client loading the fixture `@skd_fixture`, and runs one check 100 times, in two runs of 50. The check places `p1`, creates the fixture's `skdfx_Crate` 2 m in front of it, waits until `p1` has the crate, fires its `Open crate` action as `p1`'s own choice with `skd_fnc_useAction`, and then asserts with `skd_fnc_expectEach` on `"all"` that every machine reads the crate's public variable `skdfx_open` as `true`. The firing goes from `p1` to the server, which opens the crate and sets the variable, and the variable reaches the second client and the headless client from there: the round trip the pass condition names. The wait measured is that last step's duration in the result file.

| Measure | Value |
|---|---|
| Waits within 10 s | 100 of 100 |
| Median | 0.107 s |
| 95th percentile | 0.128 s |
| Longest | 0.140 s |
| Check duration, reset included | 2.1 to 2.3 s, of a 120 s timeout |
| Reset before each check | median 0.449 s, longest 0.461 s, of a 15 s ceiling |
| Session start | 50.8 s, of a 180 s ceiling |

The 10 s default is about 70 times the longest wait. It is not lowered: a wait on a mod's own effect, such as a unit that has to fall and settle, is far longer than a replicated variable, and the default is the ceiling a check gets without thinking about it.

## The ported suite

arma-grappling-hook's ten ported checks[^port] ran with their own `shakedown.toml`, one session per setup, each setup's checks given several times in one `shakedown run`:

| Setup | Executions | Outcomes | Check durations | Least margin to the check's `@timeout` | Resets | Session start |
|---|---|---|---|---|---|---|
| `sp-cba` | 54 (9 checks, 6 times) | 54 pass | 1.5 to 51.4 s | 28.4 s (`CTL-01-defaults`, 1.6 s of 30 s) | median 0.375 s, longest 0.413 s | 46.7 s |
| `sp-cba-ace`, first session | 15 (3 checks, 5 times) | 2 pass, 13 `invalid` / `unhealthy` | 2.0 to 158.8 s | 141.2 s | longest 0.590 s | 33.8 s |
| `sp-cba-ace`, second session | 9 (3 checks, 3 times) | 9 pass | 10.8 to 49.2 s | 288.2 s | longest 0.382 s | 33.9 s |
| `dedicated-cba` | 16 (2 checks, 8 times) | 16 pass | 13.2 to 28.1 s | 271.9 s | median 0.494 s, longest 0.506 s | 46.0 s |

Every check of the port sets its own `@timeout`, from 30 s to 600 s, so the margin that matters for the default is the longest check, 51.4 s (`US-009-1`), 68.6 s short of the 120 s default. No reset failed, no machine was lost, and no order needed the runner's grace.

In the first `sp-cba-ace` session, during the third check (`US-064-2`, in a step whose own code waits for a fall to settle up to ten times), `p1`'s mission time fell below half of real time: "over 5.0 s of real time its mission time advanced 2.12 s". The clock watch (`HLT-02`) flagged it, and that check and the twelve after it ended `invalid` / `unhealthy` with that reason, as `DM-05` says; the clock did not recover before the run ended. The RPT shows no error or engine message for it. The same checks in a second session, on the same build, passed 9 of 9, and the port's own three runs of that setup before this round passed 3 of 3. It is counted as a slow environment, which is what the framework reported, and it is no run within 5 s of a ceiling.

## The framework's own end-to-end runs

The two tests of the M10 findings[^tests] start a singleplayer session and a dedicated session with two clients and a headless client and run three checks each, one failing on purpose; each passing check took under 3 s. The 100 checks of the wait measurement above are a run of 50 twice, every check passing with 117.7 s to spare.

## What the spec takes from it

- `NFR-PF-06` keeps its numbers and names `SPK-14` as their source.
- No ceiling changes, so no check, document or default in the code changes.

[^tests]: `crates/shakedown/tests/m10.rs`, run on 2026-11-13 with `cargo test -p shakedown --test m10 <name> -- --ignored --nocapture --test-threads=1` after `python tools/install.py`.
[^port]: arma-grappling-hook's `acceptance/shakedown/` on its branch `shakedown/m10`, run with `shakedown run` and its own `shakedown.toml` on 2026-11-13.
[^r18]: `research/18-test-tools.md`, the default ceilings of other test tools.
