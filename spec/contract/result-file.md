---
type: Specification
title: Result file
description: The contract for what a run leaves behind - the run folder, the result file and what each of its fields means; the JSON Schema is result.schema.json.
status: draft
---

# Result file

Every run writes exactly one result file, also when it was stopped or its session broke (`RES-01`). Its shape is the JSON Schema [result.schema.json](result.schema.json) (2020-12); this file says what the fields mean. Where the two disagree, the schema is fixed to match this file.

## The run folder

```
<project>/.shakedown/runs/<run id>/
  result.json            the result file
  logs/<machine>.log     each machine's log lines for the run's time span, redacted
  checks/<n>/<machine>.log   each machine's log lines for check execution n (1-based, in run order), redacted
  dumps/<machine>/<file>     each crash dump a machine lost during the run left in its profile folder
```

Every path in `result.json` is relative to the run folder (`RES-04`). The folder is under `.shakedown/`, which git ignores (`CFG-02`). Nothing in it holds a Steam ID or a token (`NFR-PV-01`, `NFR-SC-03`).

## Fields

| Field | Meaning |
|---|---|
| `schema` | Always `"shakedown/result/1"`. A change that breaks a reader raises the number. |
| `framework_version` | The version of the `shakedown` binary that wrote the file. |
| `run` | The run's id, its end state (`finished`, or `stopped` with `stopped_reason`), its wall-clock start, end and duration, and the selection it was given. |
| `session.setup` | The setup the session ran: its name, mode, client and headless client counts, world and mod names. |
| `session.machines` | Every machine start that took part in the run: name, kind, start number, the game version and the version of each mod with a `patch` key as that machine loaded it, and its state at the end of the run. A machine that was `lost` says whether it exited (with its exit code and the paths of its crash dumps under `dumps/`) or stopped answering, and the wall-clock time it was lost (`RES-05`). Its crash dumps are the `.mdmp` and `.bidmp` files written to its profile folder after its process started (`P-09`). |
| `summary` | The count of check executions per outcome, and of checks not run. `total` is `pass + fail + invalid + not_run`. |
| `checks` | One entry per check execution, in run order (below). |
| `not_run` | Selected checks that were not executed: `setup` when the check's `@setups` excludes the session's setup, `run-stopped` when the run stopped first. |
| `reloads` | Every reload done in the session between the run's start and end. |
| `logs` | Each machine's log excerpt for the whole run. |

### A check execution

| Field | Meaning |
|---|---|
| `check_id`, `name`, `tags` | From the check file as it was read for this execution. |
| `repeat` | 1, or the repeat index when the run repeats checks (`OUT-02`). |
| `outcome` | `pass`, `fail` or `invalid` (`DM-01`). |
| `reason` | `null` for `pass`. Otherwise its `kind` (`DM-03`, `DM-04`, `DM-05`), a `message` that says what happened in one or two sentences, the `machine` it concerns when there is one, and `at`, the wall-clock time of the cause that decided the outcome, so the order of `DM-06` can be read from the file (`RES-05`). |
| `started_at`, `ended_at`, `duration_s` | Wall clock, from the start of `preparing` to the outcome. |
| `reload` | `partial` when the check ran after a `mission`-level reload of mod code (`RLD-05`), otherwise `full`. |
| `steps` | Every step in the order it started, nested steps after their parent, each with its path, the machine it ran on, its state, its start and end on the clocks of `OBS-05`, its duration and its error. A step that was running when the check ended has state `running` and no end. |
| `errors` | Every script error caught on any machine during the execution, with its machine, text, file, line, stack, the check and step it was stamped with, whether the handler or the RPT scan caught it, whether an `expectError` block matched it, and its clocks. |
| `evidence.failed_step` | The path of the step the outcome was decided in. |
| `evidence.observed` | The last value read on each machine by the wait or assertion that failed. |
| `evidence.attached` | The values the check attached with `skd_fnc_evidence`, kept for every outcome. |
| `evidence.logs` | Each machine's log lines for this check. |

## Example

```json
{
  "schema": "shakedown/result/1",
  "framework_version": "0.1.0",
  "run": {
    "id": "r-20261104-213706-1", "state": "finished",
    "started_at": "2026-11-04T21:37:06.123Z", "ended_at": "2026-11-04T21:37:19.880Z", "duration_s": 13.757,
    "selection": { "ids": [], "tags": ["US-012"] }, "stopped_reason": null
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
      "reason": { "kind": "assertion", "message": "ropes attached to the wall: expected >= 1 on every machine; p2 read 0 for 10 s.", "machine": "p2", "at": "2026-11-04T21:37:19.650Z" },
      "started_at": "2026-11-04T21:37:06.200Z", "ended_at": "2026-11-04T21:37:19.700Z", "duration_s": 13.5,
      "reload": "full",
      "steps": [
        { "path": ["every machine sees the rope"], "machine": "server", "state": "failed",
          "start": { "utc": "2026-11-04T21:37:09.600Z", "tick": 412.5, "time": 301.2, "frame": 20311 },
          "end": { "utc": "2026-11-04T21:37:19.650Z", "tick": 422.6, "time": 311.3, "frame": 20817 },
          "duration_s": 10.05, "error": "ropes attached to the wall: p2 read 0" }
      ],
      "errors": [],
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
