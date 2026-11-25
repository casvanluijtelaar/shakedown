---
type: Documentation
title: Script errors, logs and evidence
description: How a script error on any machine fails a check and is traced to its machine and step, which errors count, how a check expects an error, errors only the RPT shows, each machine's log in the run folder and how to read it with logs, and the evidence a check keeps.
status: draft
---

# Script errors, logs and evidence

A verdict explains itself. A check that passes raised no unexpected script error on any machine; a check that fails names the step it failed in, the last value each machine showed, the script errors with their machine and step, and each machine's log for that check.

## A script error fails the check

Every machine of the session, the lead and every client and headless client, catches each script error the engine reports, with the `ScriptError` mission event, and sends it to the runner at once. An error on any machine during a check fails it `fail` / `script-error` once the body ends, even when the body itself went on and passed:

```sqf
// @check US-028 An error the mod raises on p2 fails the check
["trigger on p2", {
    ["p2", { [] call mymod_fnc_throw; true }] call skd_fnc_on;
}] call skd_fnc_step;
```

```
fail    US-028-on-p2  (3.4 s)
        script-error: A script error on p2 in step "trigger on p2": Undefined variable in expression: _mymod_undefined (mymod\addons\main\functions\fn_throw.sqf, line 4). Fix the error in the mod, or wrap the code that should raise it in skd_fnc_expectError.
```

The result file lists every error of the execution in the check's `errors`, in the order of their wall-clock time, each with:

| Field | Holds |
|---|---|
| `machine` | The machine it was raised on. |
| `text` | The engine's message without its `Error ` prefix, such as `Undefined variable in expression: _x` or `Generic error in expression`. |
| `file`, `line` | The file as the engine names it and the line in it. For code that has no file, such as code a check sends to a machine, the file is empty and the line counts from the code's start. An error the handler caught has its file without a leading backslash; one only the RPT showed keeps the path as the RPT writes it, which can start with one. |
| `stack` | One `file:line` and scope per frame, outermost first. The frames' variables are left out. |
| `check_id`, `step_path` | The check and the step the error was raised in. |
| `source` | `handler` when the machine's handler sent it, `rpt` when only the machine's RPT showed it (below). |
| `expected` | Whether an `skd_fnc_expectError` block matched it. |
| `at` | The machine's clocks when it was raised: wall clock, `diag_tickTime`, mission time and frame. |

**Which step.** On the lead the error belongs to the step running there. On another machine it belongs to the step the code it ran in was sent from, through `skd_fnc_on`, a wait or an assertion. An error raised there in code nothing sent, such as a function the server reached with `remoteExec`, belongs to the step running on the lead at that moment, or, between two steps, to the last step that started. `evidence.failed_step` of a `script-error` is the error's step.

**What counts.** Errors from the start of the check's reset to a second after its body ended are listed. Those raised during the reset are listed but do not fail the check, and neither does a compile error in code the check sent, which ends the check `check-error` itself. When something else decided the check first, such as an assertion, that cause stands and the error is listed after it.

**What the engine reports.** The engine reports an undefined variable only in scheduled code: the check body, the code a check sends to a machine, and code started with `spawn` or `remoteExec`. The same mistake in unscheduled code, such as an event handler, `isNil` or `remoteExecCall`, is no error at all and leaves nothing to catch. A type error, such as `"a" + 1`, is reported in both.

## Expecting an error

A check of a mod's error path says which error it expects with `[text, code, args] call skd_fnc_expectError`. It runs `code` with `call` in the body and returns its value. Every error on any machine whose text contains `text`, case-insensitively, from the start of `code` to the end of the check, is marked `expected` and fails nothing:

```sqf
["p2 rejects the bad call", {
    ["_mymod_undefined", {
        ["p2", { [] call mymod_fnc_throw; true }] call skd_fnc_on;
    }] call skd_fnc_expectError;
}] call skd_fnc_step;
```

- Any other error still fails the check.
- When no error matched by the check's end, a check that would otherwise pass ends `fail` / `expected-error-missing`, quoting the text, with `evidence.failed_step` the step the block was called in.
- `text` must be a String that is not empty.
- Match on a part of the message that is stable, such as the variable's name, rather than the whole line.

## Errors only the RPT shows

The runner also reads each machine's lines for every check from its RPT and adds every script error they show that no handler error of that machine with the same text and file accounts for, with `source` `rpt`. Its `at` gives the wall clock to the second, from the RPT's local time, and `tick`, `time` and `frame` 0, which the RPT does not give. Such an error fails the check like any other. A line written to the RPT by other means than the engine's error report, such as `diag_log text "Error in expression <...>"`, is one.

## Errors outside a check

Errors raised between checks, or by `shakedown exec`, are kept with the session. `shakedown errors` lists them all, inside checks and outside, the oldest first; an error raised outside any check has `check_id` `null` ([Asking a running game](asking-a-running-game.md#script-errors-errors)).

## Each machine's log, with the run

Every machine writes its RPT to its profile folder in the session folder (`status` shows each machine's `profile`); the folder and its RPT are kept after the session stops. At the start and end of a run, and of each check execution, every machine writes a marker line into its RPT:

```
 7:15:18 [shakedown] r-20261106-071512-1#3 start 2026-11-06T07:15:18.412Z
```

The key is the run id, or `<run id>#<n>` for the run's `n`th check execution. The run folder keeps the lines between each pair, redacted:

```
.shakedown\runs\<run id>\
  result.json
  logs\<machine>.log          each machine's lines for the whole run
  checks\<n>\<machine>.log    each machine's lines for check execution n, from its reset to its end
```

The result file names them: `logs` for the run, and each check's `evidence.logs`, by machine. The engine writes a line to the RPT about half a second after a script writes it, so the runner waits for each end marker to reach the file before it cuts the lines. A machine that was lost during a check still has its lines for that check, up to its last line.

## Reading logs: `shakedown logs`

`shakedown logs --machine <m>` prints a machine's log lines, redacted. It works while a run is active and while the session is broken.

| Option | Takes the lines from |
|---|---|
| none | The machine's RPT in the running session, up to now. |
| `--run <id>` | That run's `logs\<machine>.log`. |
| `--check <id>` | The check's lines: while it executes, its lines so far; otherwise its last execution in the newest run that has one, or in the run `--run` names. |
| `--grep <regex>` | Only the lines the regular expression matches. |
| `--last <n>` | The last `n` lines of those, default 200. |

```
shakedown logs --machine p2 --check US-028-on-p2 --grep "Error"
shakedown logs --machine server --last 50 --json
```

With `--json` it prints `{"lines": [...], "truncated": ..., "path": ...}`: `truncated` is `true` when more lines matched than it printed, and when the lines are longer than 20,000 characters it prints the last ones that fit and `path` names the file in the session folder that holds them all; without `--json` it then says `Every line is in <path>.` on stderr.

## The evidence a check keeps

A check that does not pass keeps in its result:

- `reason`: the kind, a message, the machine it concerns and the wall-clock time of the cause. A wait at its ceiling names what was awaited and the ceiling, such as `skd_fnc_waitUntil: "flag reached p1" was not met on p1 within its ceiling of 3 s.`
- `evidence.failed_step`: the path of the step it failed in; for a script error, the error's step.
- `evidence.observed`: the last value the failed wait or assertion read on each machine it polled.

Every check, whatever its outcome, keeps:

- `evidence.attached`: what the check attached with `[label, value] call skd_fnc_evidence`, as `{"label": ..., "value": ...}`.
- `evidence.logs`: each machine's lines for the check.
- `errors`: every script error, expected or not.
- `events`: every event the check recorded, on every machine, with its arguments and clocks ([Recording the mod's events](recording-events.md)).

```sqf
["crate contents", ["p1", { itemCargo (_this select 0) }, [_crate]] call skd_fnc_on] call skd_fnc_evidence;
```

Everything the framework writes is redacted: a Steam ID or the Steam account's names in a log, an error or a value read `<steamid>` and `<steamname>` ([Security and privacy](security-and-privacy.md)).
