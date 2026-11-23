---
type: Documentation
title: Every setup, repeats and slow steps
description: How one command runs a suite in every setup its checks name, one session after the other, what happens when a setup's session does not start, how to repeat a check and read its spread of outcomes and a flaky summary, what becomes of repeats when the session breaks, and how slow steps and slow checks are flagged.
status: draft
---

# Every setup, repeats and slow steps

A suite that runs in singleplayer and on a dedicated server, with and without an optional mod, is one command you walk away from. A check that passes only sometimes shows as flaky instead of as a pass. A step that takes long is pointed out without reading every step's times.

## One command for every setup

```
shakedown run --each-setup
```

With no session running, this runs the selection once for each setup that a selected check names, in the order `shakedown.toml` declares the setups. A check names the setups of its `// @setups` header; a check with no `@setups`, or whose header cannot be read, names every setup. Each setup gets what `run --setup` gives one ([Running a suite unattended](unattended-runs.md)): its own session, started; a run of the selected checks that name it; a result file in its own run folder; and the session stopped before the next setup's starts. Only one game runs at a time.

A selection works as for any run: `shakedown run --each-setup --tag smoke` starts only the setups the checks tagged `smoke` name. `--repeat` goes with it.

The command prints a line per check execution as it ends, and a line per setup when it ends:

```
pass    rope/US-012-hook-holds  (13.5 s)
setup sp-cba: 1 passed, 0 failed, 0 invalid, 0 not run. Result: C:\mods\agh\.shakedown\runs\r-20261116-101500-1\result.json
pass    rope/US-012-hook-holds  (14.1 s)
setup dedicated-cba: 1 passed, 0 failed, 0 invalid, 0 not run. Result: C:\mods\agh\.shakedown\runs\r-20261116-101632-2\result.json
2 setups: 2 passed, 0 failed, 0 invalid, 0 not run.
```

It exits with the highest exit code of its setups' runs: 0 when every check passed in every setup, 1 when one failed anywhere, 2 when none failed but one was invalid, was not run because its session did not start, or a run was stopped. With `--json` it prints one document, the `run_result` of the whole command: the setups' counts added up, every outcome (each with its `setup`), and `runs`, the `run_result` of each setup's run with its `setup` and `result_file`.

With a session running, `--each-setup` is refused with exit code 4, since it starts a session of its own for each setup:

```
Session s-20261116-100012 of setup sp runs, and a run over every setup starts a session of its own for each. Stop it with `shakedown session stop`, or run against it without --each-setup.
```

A Ctrl+C or Ctrl+Break stops the setup's run that is going, as for any run ([Stopping a run by hand](unattended-runs.md#stopping-a-run-by-hand)), stops its session, prints `Run stopped by hand: no further setup started.` and starts no further setup. A check that hands over to a person keeps that setup's session up and starts no further setup ([Handing over to a person](handing-over.md)).

### When a setup's session does not start

A setup whose session start is refused, such as for a mod folder that does not exist, or fails, because a machine exits or the start ceiling passes, costs only its own checks. Its result file lists each of them in `not_run` with reason `session-failed` and the start's reason in `message`, and the next setup still runs:

```
setup sp-cba-ace: did not run, the session start was refused: The session cannot start; missing: the folder C:\mods\ace of mod ace. Provide what is missing and start the session again.. 0 passed, 0 failed, 0 invalid, 3 not run. Result: ...
```

The command then exits 2 if no check failed elsewhere. The result file of a refused start has `session.id` `null` and no machines, and its run is `finished` with its start and end both the moment it was given up; that of a failed start names the session that failed, whose folder holds each machine's RPT.

### From an AI agent

`run_start` with `"each_setup": true` does the same, and returns at once with a handle, `e-<time>-<n>`, as its `run_id`, and `setups`, the setups in the order they will run. `run_status` with that handle gives `setup`, the setup it is in, and `runs`, each setup's run id and state (`waiting`, `starting-session`, `running`, `finished`, `stopped` or `session-failed`). `run_result` gives the whole of it as `run --each-setup --json` prints it, with each setup's `session_reason` for a session that did not start, and `run_stop` stops the setup's run that is going and starts no further one ([AI agents and the MCP tools](agent-tools.md)). `each_setup` with `setup` is `bad-arguments`; with a session running, `session-exists`.

## Repeating a check

```
shakedown run rope/US-064-1 --repeat 10
```

`--repeat n`, from 1 to 100, runs each selected check `n` times in a row, each time its own execution with its own reset. Each execution prints its line, the second and later with `#<repeat>`, and after the run a line per check gives its summary and the count of each outcome:

```
pass    rope/US-064-1  (9.8 s)
fail    rope/US-064-1 #2  (10.1 s)
        assertion: the leg damage after the fall: expected < 0.5 on p1 within 10 s, last read 0.62.
...
flaky   rope/US-064-1 in sp-cba-ace: 8 pass, 2 fail, 0 invalid, 0 not run
8 passed, 2 failed, 0 invalid, 0 not run. Result: ...
```

The summary outcome is read from the repeats that ran:

| Summary | When |
|---|---|
| `pass` | Every repeat that was not `invalid` passed. |
| `fail` | Every repeat that was not `invalid` failed. |
| `flaky` | The repeats include both a `pass` and a `fail`. |
| `invalid` | Every repeat that ran was `invalid`. |
| `not-run` | No repeat ran. |

An `invalid` repeat says nothing about the mod, so it neither makes a check flaky nor hides a failure; its count stands beside the others. `flaky` is a summary only: each execution keeps its own outcome in the result file. A run with a flaky check exits 1, as for a failed one.

The result file gives `run.repeat`, every execution in `checks` with its `repeat` (1 to `n`), and `repeats`, one entry per selected check with its summary `outcome` and `counts` of `pass`, `fail`, `invalid` and `not_run`. Through MCP, `run_start` takes `"repeat": n`, and `run_result` has the same `repeats`, each with the `setup` it ran in.

Repeats are how to find a flaky check: run it 20 times, and a `flaky` summary with the failed executions' evidence says what differs.

### When the session breaks during repeats

A machine that is lost during a repeat ends that repeat `invalid` / `machine-lost`, as it ends any check ([Outcomes and a broken session](outcomes-and-broken-sessions.md)). The repeats not yet started are not run: they are in `not_run` with reason `session-broken` and the repeat each would have had, so the summary counts only the repeats that ran. Two passes, then a lost machine in the third of five repeats, give the summary `pass` with counts 2 pass, 1 invalid and 2 not run, and the run exits 2. When every repeat that ran was invalid, the summary is `invalid`.

In a run without `--repeat`, the checks after a broken session are `invalid` / `session-broken` instead.

## Slow steps and slow checks

The time of every step is in the result file, on every clock. Two kinds of time are flagged `slow`, in the result file and in the line printed for the check:

- a step whose duration is above the project's `slow_step`, 10 s unless `shakedown.toml` sets it ([The project file](project-file.md)): the step has `"slow": true`, and the line names it with its duration;
- a check execution whose `duration_s` is more than half its `@timeout`: the execution has `"slow": true`, and its line says `slow` after the duration.

```
pass    rope/US-078-climb  (14.2 s)
        slow step: climb to the top (12.1 s)
pass    rope/US-078-long  (11.3 s, slow)
```

A nested step that holds a slow step is slow too, since its time holds the inner one's; its line names it by its whole path, as `slow step: climb > reach the top (12.1 s)`. `run.slow_step_s` in the result file is the threshold the run used. Through MCP, each outcome line has `slow`, `true` when the execution or one of its steps was flagged. A slow flag changes no outcome.

`slow_step` is a number of seconds above 0, a whole number or a decimal. Any other value, such as `-1`, `0` or `"10"`, is refused by `list` and `run` with exit code 3, naming `slow_step` and its line, and nothing starts:

```
shakedown.toml, line 3: slow_step must be a number of seconds above 0, got -1.
```
