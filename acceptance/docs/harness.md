---
type: Documentation
title: How the harness is built
description: The parts of the acceptance harness, how a check is named, phased and judged, and what the first round learned about driving the framework.
status: draft
---

# How the harness is built

The harness follows the last section of `spec/07-tech-stack.md`: the checks that need the game are SQF files written against the check API, and Python drives the `shakedown` command line, reads the result files and judges them. It shares no code with the framework and opens none of its source.

## Parts

| Part | Holds |
|---|---|
| `run.py` | Options, the generated project with its setups, one session per setup around its in-session checks, cleanup by process id, the report. |
| `harness/core.py` | The check registry (`@check(name, phase, needs, setup)`), the batch registry (`@batch(setup)`), the setups in the order their sessions run, and the shared state of a run: the session, every run and its result file. |
| `harness/driver.py` | Calls `shakedown` with a ceiling, reads `--json` output, lists and stops processes by id with `tasklist` and `taskkill`, reads `runner.json`. |
| `harness/schema.py` | A validator for the JSON Schema 2020-12 keywords `result.schema.json` uses. It refuses a schema with a keyword it does not know, so a new keyword in the contract shows up as a harness error. Written here instead of taking `jsonschema` as a dependency. |
| `harness/m00.py` | The checks of M0 and their batch. |
| `harness/m01.py` | The checks of M1 and their batches. |
| `checks/` | The SQF checks: one folder per story, `m01/` for M1's requirements, `sel/` for the selection, `agh/` for the M1 demo against arma-grappling-hook. |
| `templates/` | SQF checks written into the project in numbered copies. |

## Checks

A check is a Python function with a name that starts with the story or requirement ID it proves. Its phase says when it runs:

- `offline`: once, before any game starts. Commands that need no runner, such as `list`.
- `session`: once per repeat, inside the session of its setup (`sp` or `cba`). Each names the SQF checks it reads (`needs`). Per repeat each module's batch for that setup makes its runs: the union of what the selected checks need, in a fixed order that puts every broken check before the check that must pass after them, plus the runs a check needs on its own (the 20 checks of `NFR-RL-02`, the four selections of `RUN-01`, the four selections with a repeated id and one text run of `m01-3`, the demo run). Then every selected check is judged against those result files.
- `end`: once, after the last session stopped.

A failed assertion fails only its own check; the others are still judged. A harness error is reported as a failure of the check it happened in.

## What the first round learned

- A singleplayer session starts in about 40 s on the test machine and a run of eleven checks takes a tenth of a second, so the flake check costs one start.
- `run --json` gives `result_file` as an absolute path; the run folder is its parent.
- A header mistake is reported with no `machine` in its reason and no steps, which is how the checks see that the game was not touched.
- Stopping the runner by id leaves its `runner.json` behind; the next command starts a new runner over it. The harness therefore waits for the runner to exit by itself first.

## What the M1 round learned

- Two sessions, `cba` and `sp`, cost about 40 s each to start; everything inside them takes about 40 s more, most of it the two `US-039` timeouts and the 20 checks of `NFR-RL-02`.
- `run` given an id more than once executes it once per time (`m01-3`), so the demo's ten passes are one run of the demo and the second check, given ten times each, in turn.
- The reset leaves variables on units and in `missionNamespace`, as `docs/check-api.md` says. A check that counts something on the player's unit, such as the fixture's `skdfx_signalDown`, registers a cleanup that puts it back; the `NFR-RL-02` checks keep the start state they compare against in `acc_m01_base` for the same reason.
- arma-grappling-hook's placed hook needs a lip: the demo check finds a container's south lip by casting rays down from above, places `p1` 0.6 m inside it facing out, and calls `agh_fnc_placeHook` from there.
