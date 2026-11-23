---
type: Documentation
title: Outcomes and a broken session
description: What pass, fail and invalid mean and every reason kind each carries, which cause decides when several come in one check, how the runner sees a machine that stops answering, a process that ends or crashes, a game clock that stops and a machine whose frame rate collapses or whose answers lag, what each machine's health in status shows, what a lost machine leaves in the result file, how to look into a broken session without a restart, and how to make a failure on purpose.
status: draft
---

# Outcomes and a broken session

Every check execution ends with exactly one outcome, and every outcome but `pass` has a reason: a kind, a message and the wall-clock time of its cause.

- **`pass`**: the body ran to its end, no assertion or wait failed, and no machine raised an unexpected script error.
- **`fail`**: the mod did not do what the check asserts, or the check itself is wrong.
- **`invalid`**: the environment stopped the check from reaching a verdict: a machine, the session or the run went wrong, not the mod. Run it again.

A run exits 0 when every check passed, 1 when one failed, and 2 when none failed and one was invalid.

## The reason kinds

| Outcome | Kind | When |
|---|---|---|
| `fail` | `assertion` | An expectation did not hold; the evidence holds what was expected and what each machine last read. |
| `fail` | `wait-ceiling` | A wait reached its ceiling. |
| `fail` | `script-error` | A script error that no `skd_fnc_expectError` expected, on any machine, during the check. |
| `fail` | `expected-error-missing` | An `skd_fnc_expectError` block saw no matching error. |
| `fail` | `check-timeout` | The body ran past the check's `@timeout` while every machine still answered. |
| `fail` | `check-error` | The check is wrong: it does not compile, its header is malformed, it names a machine the session lacks, or it calls the API wrongly. Fix the check. |
| `fail` | `explicit` | The check called `skd_fnc_fail`. |
| `invalid` | `machine-lost` | A machine's process ended, or the machine stopped answering. The message names the machine, how it was lost, its exit code and its crash dumps. |
| `invalid` | `session-broken` | The session was broken before the check started. In a run with `--repeat`, such an execution is listed in `not_run` with reason `session-broken` instead ([When the session breaks during repeats](every-setup-and-repeats.md#when-the-session-breaks-during-repeats)). |
| `invalid` | `reset-failed` | The reset before the check did not complete within its 15 s; the message lists what did not match, or names the check whose cleanup did not end, or the ACE version that has no full heal for the reset. A `@reset mission` check whose mission restart did not bring a machine back ends this way too. |
| `invalid` | `reload-failed` | The reload the check needed did not complete. |
| `invalid` | `run-stopped` | The run was stopped by hand or by its runner ceiling ([Running a suite unattended](unattended-runs.md)). |
| `invalid` | `unhealthy` | During the check, while the machine still answered, its mission time or frame number stopped, its frame rate fell under 10 frames a second, or its answers took longer than 2 s. The message names the machine and the measure. |
| `invalid` | `explicit` | The check called `skd_fnc_invalid`. |

```
$ shakedown run pass failing explicit
pass    pass  (1.2 s)
fail    failing  (0.9 s)
        explicit: broken on purpose
invalid explicit  (0.9 s)
        explicit: no network
```

A check calls `skd_fnc_invalid` for a cause outside the mod that it detects itself, such as a dependency that is not loaded, so the run does not count it as the mod's failure.

## When two causes come in one check

The first cause in wall-clock time decides. Each reason carries its `at`, so the order is in the result file:

- An assertion that failed before a machine was lost, or before it was flagged unhealthy, stays `fail`: the failure is the mod's.
- A machine lost, or a clock stopped, before anything failed makes the check `invalid`, and a failure that follows is not trusted: a wait that reached its ceiling because the machine it waited on had stopped answering is not the mod's fault.
- A check whose body ran to its end and passed, but during which a machine was lost or a clock stood still, is `invalid` too.

## How the runner sees the environment fail

The runner, a process outside the game, holds the verdict on the environment:

- **A process that ends unasked.** The runner watches every game process of the session. One that ends without the runner asking is lost with reason `exited` and its exit code, within a quarter of a second. The runner looks for the `.mdmp` and `.bidmp` files the engine wrote into that machine's profile folder, waiting up to 5 s for one still being written, and copies each into the run folder as `dumps\<machine>\<file>`. A process ended from outside, such as with `taskkill /F`, leaves exit code 1 and no dump; a crash leaves the exit code of an access violation, `0xC0000005` (3221225477), or 1 in singleplayer, and a dump of each kind.
- **A machine that stops answering.** From the moment a machine is ready, its in-game agent sends a heartbeat every second of real time with its clocks. A machine whose heartbeat has not come for 10 s, or that has not answered an order within the order's ceiling plus a grace of 10 s, is lost with reason `no-answer`. A suspended process, a game that hangs and a machine whose scripts have stopped all end this way, in about 10 s. The 10 s count only time the runner itself runs: a runner that was suspended, paused in a debugger or starved for 2 s or more counts every machine's gap again from when it runs, so its own pause loses no machine.
- **A clock that stops.** While a machine still answers, the runner compares its clocks over the last 5 s of real time. When its mission time advanced less than half as much as real time, or its frame number did not advance, the machine is flagged `mission-time` or `frame`. Every check that overlaps the flagged interval ends `invalid` / `unhealthy`, naming the machine and the clock. The flag clears once both clocks have moved over a whole 5 s again. A singleplayer game paused with Escape stops its clock this way.
- **A machine that struggles.** The runner also judges each machine's frame rate, the frames of its heartbeats over the last 5 s of real time, and its answer time: with each heartbeat the runner sends the machine a ping, one at a time, which its agent answers through the same path as every order, and the answer time is how long the last ping took, or how long the one still unanswered has waited, if that is longer. A machine that ran fewer than 10 frames a second over 5 s is flagged `frame-rate`; one whose answer time stayed above 2 s at every heartbeat of 5 s is flagged `answer-time`. A machine starved of frames also loses mission time, as the engine caps the simulation step of each frame, so mission time that fell behind on steps under 10 frames a second is flagged `frame-rate` too, its message giving the frame rate over those steps, which can be under the 5 s frame rate `status` shows. A check that overlaps the flag ends `invalid` / `unhealthy`, its message naming the machine and the measure, and the flag clears once every measure held over a whole 5 s. A machine starved of CPU by other processes, or one whose scheduled scripts are swamped, ends this way. One floor serves every kind of machine: a rendering client runs about 58 frames a second while its window is presented to a display and about 19 while it is not, a dedicated server and a headless client about 47, and singleplayer more, so every healthy machine meets 10.

### Each machine's health in `status`

`status` shows each machine's health under it, once it has sent a heartbeat. `status --json` and the `status` tool give the same fields as `health`:

```
$ shakedown status
runner: running (process 10432)
session: s-20261121-200914 of setup dedicated, ready
  server  server    ready      start 1 process 11020 profile C:\...\s-20261121-200914\machines\server-1
          health: 47.1 frames a second, answers in 0.009 s, mission time 123.4 s, frame 5811
  p2      client    ready      start 1 process 11388 profile C:\...\s-20261121-200914\machines\p2-1
          health: 6.0 frames a second, answers in 0.017 s, mission time 118.9 s, frame 6290, flagged frame-rate since 2026-11-21T20:11:30.512Z
```

| `health` field | Holds |
|---|---|
| `flag` | `null`, or the measure flagged: `frame`, `mission-time`, `frame-rate` or `answer-time`. |
| `since` | When the flag was raised. |
| `frame_rate` | Frames a second over the last 5 s of heartbeats, to one decimal; `null` until the heartbeats span 1 s. |
| `answer_s` | The answer time in seconds; `null` before the first ping. |
| `heartbeat` | The clocks of the last heartbeat: `utc`, `tick`, `time` and `frame`. |

## A lost machine and the broken session

A machine that is lost ends the check that is running at once: `invalid` / `machine-lost`, its message naming the machine and how it was lost, the order it was given if any, and its crash dumps, with its last 20 RPT lines in the evidence as an attachment. The session is then **broken**: each following check of the run ends `invalid` / `session-broken`, or, in a run with `--repeat`, is not run with reason `session-broken`, and the run exits 2.

```
invalid waitp2  (11.4 s)
        machine-lost: p2 stopped answering: no heartbeat came for 10 s, its last answer at 2026-11-12T12:00:03.120Z; it was asked to poll `false`. The environment failed, not the mod; its process still runs for inspection, and the session is broken: restart its game with `shakedown reload`, or stop it with `shakedown session stop`.
invalid pass  (0.0 s)
        session-broken: The session was broken before this check started: ...
```

A session also breaks when a reset fails, when a reload fails, and when a client joins a server that is not the session's own ([A client sent to another server](starting-and-failed-starts.md#a-client-sent-to-another-server)). A run asked on a broken session is refused, exit 4, naming `shakedown reload`.

### What the result file keeps of a lost machine

`session.machines` lists the lost machine with `end_state` `lost` and a `lost` object: its `reason` (`exited`, `no-answer` or `foreign-server`), the wall-clock time it was lost (`at`), its `exit_code` (`null` when it stopped answering) and the paths of its crash dumps under `dumps/` ([Results and the result file](results.md)). The check's `evidence.logs` points to the machine's log lines for that check, up to its last line, even though the machine never wrote the check's end marker.

```json
{ "name": "p2", "kind": "client", "start": 1, "end_state": "lost",
  "lost": { "reason": "exited", "at": "2026-11-12T12:00:02.410Z", "exit_code": 3221225477,
            "dumps": ["dumps/p2/arma3_x64_2026-11-12_12-58-41.bidmp", "dumps/p2/arma3_x64_2026-11-12_12-58-41.mdmp"] },
  "game_version": "2.22.154089 stable", "mods": [] }
```

A crash dump is copied as the engine wrote it and may hold the Steam account; like everything under `.shakedown\`, git ignores it.

## Looking into a broken session

A broken session keeps every game process running, so it can be looked at as it is ([Asking a running game](asking-a-running-game.md)):

1. `shakedown status` shows the session `broken` with its `reason`: the message, the machine and that machine's last 20 RPT lines. Each machine shows its state (`lost` for the one that went), its process id and start time, its profile folder, its last answer and its health with the clocks of its last heartbeat.
2. `shakedown inspect --machine <name>` still answers for every machine that answers, while a check runs too. A lost machine is not asked: `inspect` returns it with state `lost` and every other field `null`, at once. A machine that stops answering while `inspect` waits on it is returned the same way within 20 s.
3. `shakedown logs --machine <name>` reads the machine's RPT, also of a lost machine; `shakedown errors` lists the script errors the session caught, inside checks and outside.
4. `shakedown exec` runs code on any machine that answers; a lost machine comes back `lost` at once.
5. `shakedown reset` is refused on a broken session with exit code 4, naming the ways out.

Two ways out: `shakedown reload`, which on a broken session always restarts the game of every machine ([Reloading changed mod files](reloading.md)), and `shakedown session stop`.

## Making a failure on purpose

To see what a failure does, a check or a person makes one at a known moment:

- **A stall**: suspend the machine's process by the process id `status` gives. From PowerShell:

  ```powershell
  Add-Type -Name N -Namespace W -MemberDefinition '[DllImport("ntdll.dll")] public static extern int NtSuspendProcess(IntPtr h);'
  [W.N]::NtSuspendProcess((Get-Process -Id 10432).Handle)
  ```

  `NtResumeProcess` the same way resumes it.
- **An end from outside**: `taskkill /F /PID 10432`. End a game process only by the id `status` gives, never by name.
- **A stopped clock**: pause a singleplayer game with Escape in its window.
- **A failing assertion**: change one expected value in a check and see it fail with its step and the value it read. A suite in which nothing can fail proves nothing.

With the fixture mod `@skd_fixture` of an install from source loaded ([spec/contract/fixture-mod.md](../spec/contract/fixture-mod.md)):

- **A crash**: `["p2", {[0.5] call skdfx_fnc_crash}] call skd_fnc_on`. The delay places it between two steps of the check.
- **An exit with a chosen code**: `["p2", {[0.5, 7] call skdfx_fnc_exit}] call skd_fnc_on`.
- **A stopped clock**: `[8] call skdfx_fnc_holdClock` in singleplayer.
