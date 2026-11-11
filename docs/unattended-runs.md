---
type: Documentation
title: Running a suite unattended
description: How to run the whole suite with one command that starts and stops its own session, how to stop a run by hand from the console, the command line or an AI agent, the runner ceiling that stops a run that hangs, and how to read the result file of a stopped run.
status: stable
---

# Running a suite unattended

One command runs every check of a setup and leaves nothing behind, whatever happens to the run (`RUN-04`, `RUN-05`, `RUN-06`, `STP-02`).

## One command, then walk away

```
shakedown run --setup dedicated-2
```

With no session running, `run --setup` starts a session of that setup, runs every check, and stops the session, whatever the outcomes: passed, failed, stopped by hand or at the runner ceiling. Give ids or `--tag` to run fewer. When it ends:

- the exit code says how it went: 0 every check passed, 1 a check failed, 2 none failed and one was `invalid` or the run was stopped;
- the run folder `.shakedown\runs\<run id>\` holds `result.json`, `logs\<machine>.log` for each machine and `checks\<n>\<machine>.log` for each check execution ([What a run leaves behind](writing-and-running-checks.md#what-a-run-leaves-behind)); every path in `result.json` is relative to that folder;
- no `arma3_x64` or `arma3server_x64` process of the session runs, no port from `base_port` to `base_port` plus 4 is held, the mission copy in the game's `MPMissions` folder is gone, and `shakedown status` shows no session.

The framework writes nothing in the project but `.shakedown\`, which holds a `.gitignore` of `*`, so git never sees a run.

With a session already running, `run` without `--setup` uses it and leaves it `ready` afterwards; `run --setup` naming another setup is refused.

## Stopping a run by hand

Any of these stop a run alike (`RUN-05`):

| How | Where |
|---|---|
| Ctrl+C or Ctrl+Break | In the console where `shakedown run` runs. A program that drives `shakedown run` sends Ctrl+Break to its process group. |
| `shakedown run stop` | From any other console, for the active run. With no run active it exits 4. |
| `run_stop {"run_id": "..."}` | From an AI agent ([Agent tools](agent-tools.md)). |

What happens:

1. The check being reset or executing ends `invalid` with reason kind `run-stopped`; its body is terminated in the game. A check that had already ended keeps its outcome.
2. Every check not started is listed in `not_run` with reason `run-stopped`.
3. The result file is written with `run.state` `stopped` and `run.stopped_reason` `by-hand`, and the logs are cut out as for any run.
4. A run that started its own session stops it; a run against a session you started leaves it `ready`, with every machine on the same start, so you can run again at once.

`run stop` and `run_stop` return once the result file is written, with the run's status: its state, how many checks executed and each outcome. `shakedown run` prints its usual lines and summary, then exits 2 unless a check failed:

```
pass    first  (2.0 s)
Stopping run r-20261111-222122-4; its result file comes next.
invalid slow  (1.6 s)
        run-stopped: The run was stopped by hand (`run stop`, `run_stop`, Ctrl+C or Ctrl+Break) before this check reached a verdict.
Run stopped: by-hand.
1 passed, 0 failed, 1 invalid, 1 not run. Result: C:\...\.shakedown\runs\r-20261111-222122-4\result.json
```

A Ctrl+C while `run --setup` is still starting its session lets the start finish, then ends the run at once with every check not run, and stops the session. A mission restart before a `@reset mission` check also finishes before the stop takes the check, so no machine is left half restarted.

`shakedown session stop` during a run stops the run first, with `stopped_reason` `session-stopped`, then the session.

## The runner ceiling

A suite you walked away from must end even when something hangs. Every run has a runner ceiling: the sum of the `@timeout` of the checks it executes, plus 60 s, counted from the start of its first check, after any session it starts is ready (`RUN-06`). Resets, mission restarts and cleanups count against it. A check whose `@setups` leaves out the session's setup adds nothing.

At the ceiling the run stops as by hand, with `stopped_reason` `run-ceiling`, and the check it was on says so:

```
invalid c6  (4.1 s)
        run-stopped: The run reached its runner ceiling of 68 s, the sum of its checks' timeouts plus 60 s, before this check reached a verdict. ...
```

Eight checks of `@timeout 1` have a ceiling of 68 s. If each registers a cleanup that runs 14 s, the cleanups alone outlast it, and the sixth check is stopped about 69 s after the first started. A run that reaches its ceiling usually means a check's `@timeout` is too short for what it, its reset or its cleanups really take: raise it, or find what runs long.

## Reading a stopped run's result

The result file of a stopped run is a whole result file, valid against [result.schema.json](../spec/contract/result.schema.json):

| Field | In a stopped run |
|---|---|
| `run.state` | `stopped` (a run that ran every check is `finished`). |
| `run.stopped_reason` | `by-hand`, `run-ceiling` or `session-stopped`; `null` for a finished run. |
| `checks` | Every check that started, the last one `invalid` / `run-stopped` when the stop came during it, with its script errors and its logs; it lists no steps, since the game's report of them never came. |
| `not_run` | Every check that did not start, reason `run-stopped`, one entry per time it was given. |
| `summary` | `total` counts the checks not run too: `pass + fail + invalid + not_run`. |

`run_status` and `run_result` give the same summary from the file. To finish what a stopped run left, run the ids in `not_run` again.
