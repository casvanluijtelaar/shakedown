---
type: Documentation
title: Outcomes and a broken session
description: What pass, fail and invalid mean and every reason kind each carries, which cause decides when several come in one check, how the runner sees a machine that stops answering, a process that ends or crashes and a game clock that stops, what a lost machine leaves in the result file, and how to find out what went wrong in a broken session without a restart.
status: draft
---

# Outcomes and a broken session

Every check execution ends with exactly one outcome, and every outcome but `pass` has a reason: a kind, a message and the wall-clock time of its cause (`OUT-01`, `RES-05`). `fail` means the mod did not do what the check asserts, or the check itself is wrong. `invalid` means the environment stopped the check from reaching a verdict: a machine, the session or the run went wrong, not the mod. A run exits 0 when every check passed, 1 when one failed, and 2 when none failed and one was invalid ([spec/contract/cli.md](../spec/contract/cli.md)).

## The reason kinds

| Outcome | Kind | When |
|---|---|---|
| `fail` | `assertion` | An expectation did not hold; the evidence holds what was expected and what each machine last read. |
| `fail` | `wait-ceiling` | A wait reached its in-game ceiling. |
| `fail` | `script-error` | A script error that no `skd_fnc_expectError` expected, on any machine, during the check. |
| `fail` | `expected-error-missing` | An `skd_fnc_expectError` block saw no matching error. |
| `fail` | `check-timeout` | The body ran past the check's `@timeout` while every machine still answered. |
| `fail` | `check-error` | The check is wrong: it does not compile, its header is malformed, it names a machine the session lacks, or it calls the API wrongly. Fix the check. |
| `fail` | `explicit` | The check called `skd_fnc_fail`. |
| `invalid` | `machine-lost` | A machine's process ended, or the machine stopped answering. The message names the machine, how it was lost, its exit code and its crash dumps. |
| `invalid` | `session-broken` | The session was `broken` before the check started. |
| `invalid` | `reset-failed` | The reset before the check did not complete within its 15 s; the message lists what did not match, or names the check whose cleanup did not end. |
| `invalid` | `reload-failed` | The reload the check needed did not complete. |
| `invalid` | `run-stopped` | The run was stopped by hand or by its runner ceiling ([Running a suite unattended](unattended-runs.md)). |
| `invalid` | `unhealthy` | A machine's mission time or frame number stopped during the check while the machine still answered. |
| `invalid` | `explicit` | The check called `skd_fnc_invalid`. |

```
$ shakedown run pass failing explicit
pass    pass  (1.2 s)
fail    failing  (0.9 s)
        explicit: broken on purpose
invalid explicit  (0.9 s)
        explicit: no network
```

## When two causes come in one check

The first cause in wall-clock time decides (`DM-06`). Each reason carries its `at`, so the order is in the result file:

- An assertion that failed before a machine was lost, or before a clock stopped, stays `fail`: the failure is the mod's.
- A machine lost, or a clock stopped, before anything failed makes the check `invalid`, and a failure that follows is not trusted: a wait that reached its ceiling because the machine it waited on had stopped answering is not the mod's fault.
- A check whose body ran to its end and passed, but during which a machine was lost or a clock stood still, is `invalid` too.

## How the runner sees the environment fail

The runner, a process outside the game, holds the verdict on the environment (`OBS-03`):

- **A process that ends unasked** (`P-09`). The runner watches every game process of the session. One that ends without the runner asking is `lost` with reason `exited` and its exit code, within a quarter of a second. The runner looks for the `.mdmp` and `.bidmp` files the engine wrote into that machine's profile folder, waiting up to 5 s for one still being written, and copies each into the run folder as `dumps/<machine>/<file>`. A process ended from outside, such as with `taskkill /F`, leaves exit code 1 and no dump; a crash leaves the exit code of an access violation, `0xC0000005`, or 1 in singleplayer, and a dump of each kind.
- **A machine that stops answering** (`P-07`). From the moment a machine is `ready`, its in-game agent sends a heartbeat every second of real time with its clocks (`HLT-02`). A machine whose heartbeat has not come for 10 s, or that has not answered an order within the order's in-game ceiling plus a grace of 10 s, is `lost` with reason `no-answer`. A suspended process, a game that hangs and a machine whose scripts have stopped all end this way, in about 10 s.
- **A clock that stops** (`HLT-02`). While a machine still answers, the runner compares its clocks over the last 5 s of real time. When its mission time advanced less than half as much as real time, or its frame number did not advance, the machine is flagged: `status` shows `health.flag` `mission-time` or `frame` and the time it was raised. Every check that overlaps the flagged interval ends `invalid` / `unhealthy`, naming the machine and the clock. The flag clears once both clocks have moved over a whole 5 s again. The likeliest cause on the test machine is a rendering client whose remote desktop stream was cut; a singleplayer game paused with Escape stops its clock too.

A machine that is lost ends the check that is running at once: `invalid` / `machine-lost`, its message naming the machine and how it was lost, the order it was given if any, and its crash dumps, with its last 20 RPT lines in the evidence as an attachment. The session is then `broken`: each following check of the run ends `invalid` / `session-broken`, and the run exits 2.

```
invalid waitp2  (11.4 s)
        machine-lost: p2 stopped answering: no heartbeat came for 10 s, its last answer at 2026-11-12T12:00:03.120Z; it was asked to poll `false`. The environment failed, not the mod; its process still runs for inspection, and the session is broken: restart its game with `shakedown reload`, or stop it with `shakedown session stop`.
invalid pass  (0.0 s)
        session-broken: The session was broken before this check started: ...
```

## What the result file keeps of a lost machine

`session.machines` lists the lost machine with `end_state` `lost` and a `lost` object: its `reason` (`exited` or `no-answer`), the wall-clock time it was lost (`at`), its `exit_code` (`null` when it stopped answering) and the paths of its crash dumps under `dumps/` ([spec/contract/result-file.md](../spec/contract/result-file.md)). The check's `evidence.logs` points to the machine's log lines for that check, up to its last line, even though the machine never wrote the check's end marker.

```json
{ "name": "p2", "kind": "client", "start": 1, "end_state": "lost",
  "lost": { "reason": "exited", "at": "2026-11-12T12:00:02.410Z", "exit_code": 3221225477,
            "dumps": ["dumps/p2/arma3_x64_2026-11-12_12-58-41.bidmp", "dumps/p2/arma3_x64_2026-11-12_12-58-41.mdmp"] },
  "game_version": "2.22.154089 stable", "mods": [] }
```

A crash dump is copied as the engine wrote it and may hold the Steam account; like everything under `.shakedown/`, git ignores it.

## Diagnosing a broken session

A broken session keeps every game process running, so it can be looked at as it is ([Asking a running game](asking-a-running-game.md)):

1. `shakedown status` shows the session `broken` with its `reason`: the message, the machine and that machine's last 20 RPT lines. Each machine shows its `state` (`lost` for the one that went), its `pid` and `process_started_at`, its `profile` folder, its `last_answer_at` and its `health` with the clocks of its last heartbeat.
2. `shakedown inspect --machine <name>` still answers for every machine that answers, while a check runs too. A lost machine is not asked: `inspect` returns it with `state` `lost` and every other field `null`, at once. A machine that stops answering while `inspect` waits on it is returned the same way within 20 s.
3. `shakedown logs --machine <name>` reads the machine's RPT, also of a lost machine; `shakedown errors` lists the script errors the session caught, inside checks and outside.
4. `shakedown exec` runs code on any machine that answers; a lost machine comes back `lost` at once.
5. `shakedown reset` is refused on a broken session with exit code 4, naming the ways out.

Two ways out: `shakedown reload`, which on a broken session always restarts the game of every machine ([Reloading and the mission restart](reloading.md)), and `shakedown session stop`.

## Making a failure on purpose

To see what a failure does, or to test the framework, a check or a person makes one at a known moment:

- **A crash**: `["p2", {[0.5] call skdfx_fnc_crash}] call skd_fnc_on` with the fixture mod `@skd_fixture` loaded ([spec/contract/fixture-mod.md](../spec/contract/fixture-mod.md)). The delay places it between two steps of the check.
- **An exit with a chosen code**: `["p2", {[0.5, 7] call skdfx_fnc_exit}] call skd_fnc_on`.
- **A stopped clock**: `[8] call skdfx_fnc_holdClock` in singleplayer.
- **A stall**: suspend the machine's process by the `pid` `status` gives. From PowerShell:

  ```powershell
  Add-Type -Name N -Namespace W -MemberDefinition '[DllImport("ntdll.dll")] public static extern int NtSuspendProcess(IntPtr h);'
  [W.N]::NtSuspendProcess((Get-Process -Id 10432).Handle)
  ```

  `NtResumeProcess` the same way resumes it.
- **An end from outside**: `taskkill /F /PID 10432`.
