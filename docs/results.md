---
type: Documentation
title: Results and the result file
description: What a run leaves in its run folder, and every field of result.json - the run, the session and each machine's versions, the summary, each check execution with its outcome, reason, steps, script errors, events and evidence, the checks not run, repeats and reloads - with an example and how values from the game are written.
status: draft
---

# Results and the result file

Every run writes exactly one result file, also when it was stopped or its session broke. It is the full record of the run, for a person and for a program; `run --json` and the MCP tool `run_result` give a summary and its path. Its shape is the JSON Schema [result.schema.json](../spec/contract/result.schema.json), and the contract is [spec/contract/result-file.md](../spec/contract/result-file.md).

## The run folder

```
<project>\.shakedown\runs\<run id>\
  result.json                 the result file
  logs\<machine>.log          each machine's log lines for the run's time span, redacted
  checks\<n>\<machine>.log    each machine's log lines for check execution n (1-based, in run order), redacted
  dumps\<machine>\<file>      each crash dump a machine lost during the run left in its profile folder
```

Every path in `result.json` is relative to the run folder, with `/` as separator. The folder is under `.shakedown\`, which git ignores. Nothing in it holds a Steam ID, a Steam name or a token, except the crash dumps under `dumps\`, which are copied as the engine wrote them ([Security and privacy](security-and-privacy.md)).

## An example

A run of one check on a dedicated session, which failed:

```json
{
  "schema": "shakedown/result/1",
  "framework_version": "0.2.0",
  "run": {
    "id": "r-20261104-213706-1", "state": "finished",
    "started_at": "2026-11-04T21:37:06.123Z", "ended_at": "2026-11-04T21:37:19.880Z", "duration_s": 13.757,
    "selection": { "ids": [], "tags": ["US-012"] }, "repeat": 1, "slow_step_s": 10, "stopped_reason": null
  },
  "session": {
    "id": "s-20261104-213512",
    "setup": { "name": "dedicated-2", "mode": "dedicated", "clients": 2, "headless": 0, "world": "VR", "mods": ["cba", "agh"] },
    "machines": [
      { "name": "server", "kind": "server", "start": 1, "game_version": "2.22.154089 stable",
        "mods": [{ "mod": "cba", "patch": "cba_main", "version": "3.19.0.260811" }], "end_state": "ready" },
      { "name": "p1", "kind": "client", "start": 1, "game_version": "2.22.154089 stable",
        "mods": [{ "mod": "cba", "patch": "cba_main", "version": "3.19.0.260811" }], "end_state": "ready" },
      { "name": "p2", "kind": "client", "start": 1, "game_version": "2.22.154089 stable",
        "mods": [{ "mod": "cba", "patch": "cba_main", "version": "3.19.0.260811" }], "end_state": "ready" }
    ]
  },
  "summary": { "total": 1, "pass": 0, "fail": 1, "invalid": 0, "not_run": 0 },
  "checks": [
    {
      "check_id": "rope/US-012-hook-holds", "name": "US-012 A hook thrown at a wall holds a player's weight",
      "tags": ["US-012", "smoke"], "repeat": 1, "outcome": "fail",
      "reason": { "kind": "assertion", "message": "ropes attached to the wall: expected >= 1 on every machine within 10 s; last read server 1, p1 1, p2 0; not met on p2.", "machine": "p2", "at": "2026-11-04T21:37:19.650Z" },
      "started_at": "2026-11-04T21:37:06.200Z", "ended_at": "2026-11-04T21:37:19.700Z", "duration_s": 13.5,
      "reload": "full", "slow": false,
      "steps": [
        { "path": ["every machine sees the rope"], "machine": "server", "state": "failed",
          "start": { "utc": "2026-11-04T21:37:09.600Z", "tick": 412.5, "time": 301.2, "frame": 20311 },
          "end": { "utc": "2026-11-04T21:37:19.650Z", "tick": 422.6, "time": 311.3, "frame": 20817 },
          "duration_s": 10.05, "error": "ropes attached to the wall: p2 read 0", "slow": true }
      ],
      "errors": [],
      "events": [],
      "evidence": {
        "failed_step": ["every machine sees the rope"],
        "observed": { "server": 1, "p1": 1, "p2": 0 },
        "attached": [],
        "logs": { "server": "checks/1/server.log", "p1": "checks/1/p1.log", "p2": "checks/1/p2.log" }
      }
    }
  ],
  "not_run": [],
  "reloads": [],
  "logs": { "server": "logs/server.log", "p1": "logs/p1.log", "p2": "logs/p2.log" }
}
```

## The top level

| Field | Meaning |
|---|---|
| `schema` | Always `"shakedown/result/1"`. A change that breaks a reader raises the number. |
| `framework_version` | The version of the `shakedown` binary that wrote the file. |
| `run` | The run's `id`; its end `state`, `finished` or `stopped` with `stopped_reason` (`by-hand`, `run-ceiling` or `session-stopped`, else `null`); its wall-clock `started_at`, `ended_at` and `duration_s`; the `selection` it was given, `ids` as given, a repeated id each time, and `tags`; its `repeat` count, 1 when not repeated; and `slow_step_s`, the project's `slow_step` it flagged steps against. A run of `--each-setup` whose session did not start has state `finished`, its start and end both the moment it was given up. |
| `session.id` | The session's id; `null` when the run's session start was refused before a session existed. |
| `session.setup` | The setup the session ran: its `name`, `mode`, `clients` and `headless` counts, `world` and `mods`. |
| `session.machines` | Every machine start that took part in the run (below). Empty when the run's session did not start. |
| `summary` | The count of check executions per outcome, and of checks not run. `total` is `pass + fail + invalid + not_run`. |
| `checks` | One entry per check execution, in run order (below): a check given twice in the selection has two entries. |
| `not_run` | Selected checks that were not executed (below). |
| `repeats` | Only when `run.repeat` is above 1: one entry per selected check, in the order first selected, with its summary `outcome` (`pass`, `fail`, `flaky`, `invalid` or `not-run`) and `counts` of its executions per outcome and of its executions not run ([Repeating a check](every-setup-and-repeats.md#repeating-a-check)). |
| `reloads` | Every reload done in the session between the run's start and end, each as `shakedown reload --json` gives it ([Reloading changed mod files](reloading.md)). |
| `logs` | Each machine's log excerpt for the whole run, by machine. |

## Machines

Each entry of `session.machines` is one start of a machine: `name`, `kind` (`server`, `client` or `headless`), `start` number, `game_version` (such as `2.22.154089 stable`), `mods` (the version of each mod with a `patch` key as that machine loaded it, as `{"mod", "patch", "version"}`) and `end_state` (`ready`, `lost` or `stopped`).

- A machine that was **lost** has a `lost` object: its `reason`, `exited` (its process ended unasked), `no-answer` (it stopped answering) or `foreign-server` (the runner ended it because it joined a server that is not the session's own); `at`, the wall-clock time; `exit_code`, `null` when it stopped answering; and `dumps`, the paths of its crash dumps under `dumps/` ([Outcomes and a broken session](outcomes-and-broken-sessions.md#what-the-result-file-keeps-of-a-lost-machine)).
- A machine that restarted during the run, through a game restart, a `@reset mission` check or a join, is listed once per start, each with its own start number.
- A client that left, or was made to leave by the reset, ends `stopped`.

## A check execution

| Field | Meaning |
|---|---|
| `check_id`, `name`, `tags` | From the check file as it was read for this execution. |
| `repeat` | Which time the run executes this check, counting from 1, so `check_id` and `repeat` name one execution. |
| `outcome` | `pass`, `fail` or `invalid`. |
| `reason` | `null` for `pass`. Otherwise its `kind`, a `message` that says what happened in one or two sentences, the `machine` it concerns when there is one, and `at`, the wall-clock time of the cause that decided the outcome ([The reason kinds](outcomes-and-broken-sessions.md#the-reason-kinds)). |
| `started_at`, `ended_at`, `duration_s` | Wall clock, from the start of the check's preparation, its reset included, to the outcome. |
| `reload` | `partial` when the check ran after a reload that had a machine recompile a `CfgFunctions` function, until the next game restart, otherwise `full` ([What `partial` means](reloading.md#what-reloads-inside-the-mission-and-partial)). |
| `slow` | `true` when `duration_s` is more than half the check's `@timeout`. |
| `steps` | Every step in the order it started, nested steps after their parent (below). |
| `errors` | Every script error caught on any machine during the execution, from the start of its reset to the end of the wait for errors in flight, in the order of their wall-clock time ([Script errors, logs and evidence](script-errors-and-logs.md)). |
| `events` | Every event the check recorded, on every machine, in the order of their wall-clock time: `machine`, `name`, `kind` (`cba` or `mission`), `args` and `at`. Empty when the check recorded none. |
| `handed_over` | Only on the execution that handed the session over to a person, which is `pass`: `machine` and `items`. |
| `evidence.failed_step` | The path of the step the outcome was decided in; `null` for a pass. |
| `evidence.observed` | The last value read on each machine by the wait or assertion that failed; `null` when none. |
| `evidence.attached` | The values the check attached with `skd_fnc_evidence`, as `{"label", "value"}`, kept for every outcome. |
| `evidence.logs` | The path of each machine's log lines for this check, `checks/<n>/<machine>.log`, kept for every outcome. |

### Steps

Each step has its `path` (the titles from the outermost step in), the `machine` it ran on, its `state` (`passed`, `failed`, or `running` for a step still running when the check ended, which then has no `end`), its `start` and `end` on every clock, its `duration_s`, its `error` (`null` for a step that passed) and `slow`, `true` when its duration is above `run.slow_step_s`.

The clocks of a step, an error and an event are `utc`, the machine's wall clock; `tick`, its `diag_tickTime`; `time`, its mission time; and `frame`, its frame number. Two clocks of different machines line up by `utc`.

### Checks not run

Each entry of `not_run` is one execution the run would have made: `check_id`, the `repeat` it would have had, and its `reason`:

| Reason | When |
|---|---|
| `setup` | The check's `@setups` excludes the session's setup. |
| `run-stopped` | The run stopped first. |
| `session-failed` | The session of a `--each-setup` setup was refused or failed to start; `message` holds the start's reason. |
| `session-broken` | The session broke before a repeat started, in a run with `--repeat` above 1. |
| `handed-over` | An earlier check of the run handed the session over to a person. |

## Values from the game

Values the game gives, in `evidence.observed`, `evidence.attached`, an event's `args` and the output of `exec`, are written as JSON:

| SQF value | JSON |
|---|---|
| Number, String, Boolean, Array | as is |
| HashMap with only String keys | an object |
| HashMap with any other key | `{"$type": "hashmap", "pairs": [[key, value], ...]}` |
| `nil` | `null` |
| Object | `{"$type": "object", "netId": "<netId>", "class": "<typeOf>", "alive": <bool>}`; a null object has `"netId": ""` |
| Group | `{"$type": "group", "netId": "<netId>"}` |
| Side | `{"$type": "side", "name": "<str side>"}` |
| Code | `{"$type": "code", "text": "<the code as text>"}` |
| Any other type | `{"$type": "<typeName>", "text": "<str value>"}` |

## Reading it from a script

```powershell
$r = Get-Content .shakedown\runs\r-20261104-213706-1\result.json | ConvertFrom-Json
$r.checks | Where-Object outcome -ne 'pass' | ForEach-Object { "$($_.check_id): $($_.reason.kind): $($_.reason.message)" }
```

`shakedown run --json` prints the run's `run_result` with the `result_file` path, so a script can take the path from there.
