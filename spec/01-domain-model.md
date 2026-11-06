---
type: Specification
title: Domain model
description: The entities Shakedown works with, their attributes and relations, the status machine of each one that changes state, and the rules that decide a check's outcome.
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# Domain model

The words used here are defined in [00-overview.md](00-overview.md). Each rule has an ID of the form `DM-nn` and cites the PRD requirement it serves.

## Entities

```
Project 1──* Setup
Project 1──* Check
Setup   1──* Session          (at most one session runs on the machine at a time)
Session 1──* Machine          (one per role; a role restarted gets a new machine start)
Session 1──* Run
Session 1──* Reload
Run     1──* CheckExecution   (one per time a check is selected, and per repeat)
CheckExecution 1──* Step       (nested)
CheckExecution 1──* ScriptError, Evidence
Run     1──1 ResultFile
```

| Entity | Attributes | Lives |
|---|---|---|
| **Project** | Root folder; the `shakedown.toml` file; the checks folder (default `checks/`); the `.shakedown/` folder. | As long as the repository. |
| **Setup** | Name; mode (`singleplayer` or `dedicated`); `clients` (game clients, 1 in singleplayer); `headless` (headless clients, 0 in singleplayer); `mods` (ordered list of mod folders); `world` (default `VR`). | In the project file. |
| **Session** | Id (`s-` plus a UTC timestamp `YYYYMMDD-HHMMSS`); setup; token; state; session folder; game port; start time; the machines; the mission start count; the reset baseline. | From `session start` to `session stop`. |
| **Machine** | Name (`server`, `pN`, `hcN`); kind (`server`, `client`, `headless`); start number (1 for the first start of that role in a session); process id and process start time; profile folder; state; last answer time; health flag and the clocks of its last heartbeat (`HLT-02`); loaded versions (game build, each mod's `CfgPatches` version). | One per start of a role. |
| **Check** | Id; name; tags; setups (all when not given); timeout (default 120 s); reset mode (`in-place` or `mission`); source text. | In the checks folder. Read fresh for every execution. |
| **Run** | Id (`r-` plus a UTC timestamp and a counter); session; selection (ids, tags); repeat count; state; start and end time; run folder. | From start to its result file. |
| **CheckExecution** | Check id; repeat index; state; outcome; reason; start and end time; steps; script errors; evidence; reload state of the code it ran on. | Inside a run. |
| **Step** | Path (the titles from the outermost step in), title, machine it ran on, start and end time on the wall clock and on each clock of `OBS-05`, state (`passed`, `failed`, `running` when the check ended inside it), error. | Inside a check execution. |
| **ScriptError** | Machine; text; file; line; stack trace; check id and step path stamped on that machine; clocks; whether an `expectError` block matched it. | Inside a check execution, or in the session log when no check ran. |
| **Reload** | Id; session; changed files; level per file with its reason; level applied; state; duration. | Inside a session. |
| **ResultFile** | One JSON document per run, defined by [contract/result-file.md](contract/result-file.md). | In the run folder. |

## Status machines

### Session

```
            start ok                run starts
starting ─────────────▶ ready ◀──────────────▶ running
   │                    │  ▲ ▲     run ends
   │ start fails        │  │ └──────────────────┐
   ▼                    │  │ reload/reset done  │
 failed        reload / │  │                    │
               reset    ▼  │                    │
                     reloading ──── fails ────▶ broken
                                                  │
 any state ── stop ──▶ stopping ──▶ stopped  ◀────┘ (stop)
```

| State | Meaning | Leaves by |
|---|---|---|
| `starting` | Processes are being started and the session mission loaded. | `ready` when every machine of the setup is `ready`; `failed` when a process exits, the setup's start ceiling (`SES-04`) passes, or a precondition fails. |
| `ready` | Every machine answers; no run, reload or reset is in progress. | `running`, `reloading`, `stopping`. |
| `running` | A run is executing checks. | `ready` when the run ends; `broken` when a machine is lost. |
| `reloading` | A reload or a reset (`RST-03`) is in progress. | `ready` when it completes; `broken` when it fails. |
| `broken` | A machine is lost, a reset failed or a reload failed. No check runs: any check started is `invalid` with the reason that broke the session. | `reloading` by a `game-restart` reload, which brings every machine back; `stopping`. |
| `failed` | The session never became ready. Every process it started has been stopped. `status` shows it, with its reason, until the next `session start` or `session stop` (`INS-02`). | Terminal. |
| `stopping` | Processes are being stopped and files cleaned up. | `stopped`. |
| `stopped` | Nothing of the session runs. Its folder stays. | Terminal. |

### Machine

```
launched ──▶ connected ──▶ ready ──▶ stopped
    │            │           │
    └────────────┴───────────┴──▶ lost
```

| State | Meaning |
|---|---|
| `launched` | The process runs; its extension has not connected. |
| `connected` | The extension connected and showed the session token. The session mission has not reported in. |
| `ready` | The in-game agent answered the runner's hello in the session mission, and on a dedicated session the server reports the client at client state 10 (`BRIEFING READ`) or later. |
| `lost` | The process exited without being asked, or the machine did not answer an order within its runner ceiling. Records the wall-clock time, the exit code and any crash dump found in its profile folder. |
| `stopped` | The process ended after the runner asked it to. |

A `game-restart` reload or a `mission` reset on a dedicated session replaces a machine with a new start of the same role (start number plus one). The old machine ends `stopped`.

### Run

`running` → `finished` when every selected check has an outcome; `running` → `stopped` when stopped by hand or by its runner ceiling, in which case the check executing gets `invalid` with reason `run-stopped` and the checks not started are listed as not run. Both end states write the result file.

### Check execution

```
pending ──▶ preparing ──▶ executing ──▶ done(outcome)
                │              │
                └──────────────┴──▶ done(invalid)
```

| State | What happens |
|---|---|
| `pending` | Selected, not started. |
| `preparing` | The reset of `RST-01` runs, or a mission restart for a check with reset mode `mission`. The check's source is read and compiled on the lead machine. |
| `executing` | The body runs on the lead machine. |
| `done` | The outcome and reason are fixed. The state the check left in the game stays until the next reset, so it can be inspected. |

### Reload

`requested` → `classifying` → `applying` → `verifying` → `done` or `failed`. A failed reload leaves the session `broken`. The result names the level applied and, per file, the level it needed and why (`RLD-02`).

## Outcome rules

- **DM-01** Every check execution ends with exactly one outcome: `pass`, `fail` or `invalid` (`CR-31`).
- **DM-02** The outcome is `pass` when the body ran to its end, no assertion failed, no wait ended at its ceiling, and no unexpected script error was caught on any machine of the session between the start of `executing` and the end of the body (`CR-31`, `CR-23`).
- **DM-03** The outcome is `fail` when the mod did not do what the check asserts. Its reason kind is one of (`CR-31`):
  - `assertion` - an expectation did not hold; the reason holds the expected and the last observed value per machine.
  - `wait-ceiling` - a wait reached its in-game ceiling; the reason holds the description and the last observed value.
  - `script-error` - an unexpected script error on a machine; the reason names the machine, the error and the step.
  - `expected-error-missing` - an `expectError` block saw no matching error by the end of the check.
  - `check-timeout` - the body ran past the check's timeout while every machine still answered.
  - `explicit` - the check called `skd_fnc_fail`.
- **DM-04** The outcome is `fail` with reason kind `check-error` when the check itself is wrong: its source does not compile, its header is malformed, it names a machine the session does not have, an action that does not exist on the target, a keybind that is not registered, or it calls the API with wrong arguments. A broken check is the author's to fix, so it is never `invalid`; the reason says what to fix (`CR-31`, `CR-50`).
- **DM-05** The outcome is `invalid` when the environment, not the mod, stopped the check from reaching a verdict. Its reason kind is one of (`CR-31`, G3):
  - `machine-lost` - a machine's process exited, or it did not answer within a runner ceiling; the reason names the machine, the exit code if any, and any crash dump.
  - `session-broken` - the session was `broken` before the check started.
  - `reset-failed` - the reset before the check did not complete.
  - `reload-failed` - the reload the check needed did not complete.
  - `run-stopped` - the run was stopped by hand or by its runner ceiling.
  - `unhealthy` - the clock watch (`HLT-02`) flagged a machine during the check: its mission time or its frame number stopped while it still answered.
  - `explicit` - the check called `skd_fnc_invalid`.
- **DM-06** When causes for both `fail` and `invalid` are present in one check execution, `invalid` wins only if its cause came first in wall-clock time; otherwise the first cause decides. A machine lost after an assertion already failed does not hide the failure, and a failure that follows a lost machine is not trusted (`CR-31`, G3).
- **DM-07** A wait that reaches its in-game ceiling is `fail`. A wait that reaches its runner ceiling with no answer from the machine is `invalid` with reason kind `machine-lost` (`CR-20`, `CR-31`; gap H5).
- **DM-08** A check executed after a `mission`-level reload of mod code that changed only in level A records `reload: partial` in its execution, because handlers the mod registered by value keep the old code (`RLD-05`, `CR-03`).
- **DM-09** With a repeat count above 1, a check's summary outcome is `pass` when every repeat passed, `fail` when every repeat that was not `invalid` failed, `flaky` when its repeats include both `pass` and `fail`, and `invalid` when every repeat was `invalid`. `flaky` is a summary label only; each repeat keeps its own outcome (`CR-32`, a `should`).

## Machine rules

- **DM-10** Machine names are fixed by role and index: `server`; `p1` to `pN` for the setup's `clients`; `hc1` to `hcN` for its `headless`. A check addresses machines only by these names. In singleplayer `p1` is the only machine and `server` names it too (`CR-10`; gap H3).
- **DM-11** Every game process is started with `-name=<machine name>` and its own profile folder, so a machine is found by its profile name, never by slot or Steam ID (`CR-10`; gaps H3, H8).
- **DM-12** The group names `all` (every machine of the session), `clients` (`p1` to `pN`) and `headless` (`hc1` to `hcN`) stand for those machines in the order listed. In singleplayer `all` and `clients` are `["p1"]` and `headless` is empty (`CR-10`, `CR-22`).

## Value rules

- **DM-13** Values cross between machines and the runner as JSON, made with the engine's `toJSON` and read with `fromJSON`. Before encoding, the in-game agent replaces every value JSON cannot carry with a tagged object (`CR-10`):

  | SQF value | JSON |
  |---|---|
  | Number, String, Boolean, Array | as is |
  | HashMap with only String keys | object |
  | HashMap with any other key | `{"$type": "hashmap", "pairs": [[key, value], ...]}` |
  | `nil` | `null` |
  | Object | `{"$type": "object", "netId": "<netId>", "class": "<typeOf>", "alive": <bool>}`; a null object has `"netId": ""` |
  | Group | `{"$type": "group", "netId": "<netId>"}` |
  | Side | `{"$type": "side", "name": "<str side>"}` |
  | Code | `{"$type": "code", "text": "<toString>"}` |
  | Any other type | `{"$type": "<typeName>", "text": "<str value>"}` |

- **DM-14** A tagged object or group that arrives at a machine as an argument is turned back into the object or group with `objectFromNetId` or `groupFromNetId` before the code runs. Code arguments are not turned back into code (`CR-10`).
