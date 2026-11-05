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
| `run.py` | Options, the generated project, the session around the in-session checks, cleanup by process id, the report. |
| `harness/core.py` | The check registry (`@check(name, phase, needs)`) and the shared state of a run: the session, every run and its result file. |
| `harness/driver.py` | Calls `shakedown` with a ceiling, reads `--json` output, lists and stops processes by id with `tasklist` and `taskkill`, reads `runner.json`. |
| `harness/schema.py` | A validator for the JSON Schema 2020-12 keywords `result.schema.json` uses. It refuses a schema with a keyword it does not know, so a new keyword in the contract shows up as a harness error. Written here instead of taking `jsonschema` as a dependency. |
| `harness/m00.py` | The checks of M0. |
| `checks/` | The SQF checks, one folder per story. |

## Checks

A check is a Python function with a name that starts with the story or requirement ID it proves. Its phase says when it runs:

- `offline`: once, before any game starts. Commands that need no runner, such as `list`.
- `session`: once per repeat, inside one game session. Each names the SQF checks it reads (`needs`). Per repeat the harness runs one `shakedown run` with the union of those, in a fixed order that puts every broken check before the check that must pass after them, then judges every selected check against that run's result file.
- `end`: once, after `session stop` returned.

A failed assertion fails only its own check; the others are still judged. A harness error is reported as a failure of the check it happened in.

## What the first round learned

- A singleplayer session starts in about 40 s on the test machine and a run of eleven checks takes a tenth of a second, so the flake check costs one start.
- `run --json` gives `result_file` as an absolute path; the run folder is its parent.
- A header mistake is reported with no `machine` in its reason and no steps, which is how the checks see that the game was not touched.
- Stopping the runner by id leaves its `runner.json` behind; the next command starts a new runner over it. The harness therefore waits for the runner to exit by itself first.
