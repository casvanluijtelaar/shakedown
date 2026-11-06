---
type: Documentation
title: Script errors, logs and evidence
description: How a script error on any machine fails a check and is traced to its machine and step, how a check expects an error, what each machine's log in the run folder holds and how to read it with logs, the evidence a failed check keeps, and how the Steam account is kept out of all of it.
status: stable
---

# Script errors, logs and evidence

A verdict explains itself: a check that passes raised no unexpected script error on any machine; a check that fails names the step it failed in, the last value each machine showed, the script errors with their machine and step, and each machine's log for that check. Nothing a run leaves behind names the Steam account the game runs on.

## A script error fails the check

Every machine of the session, the lead and every client and headless client, catches each script error the engine reports with the `ScriptError` mission event and sends it to the runner at once (`ERR-01`). An error on any machine during a check fails it `fail` / `script-error` once the body ends (`ERR-03`), even when the body itself went on and passed:

```sqf
// @check US-028 An error the mod raises on p2 fails the check
["trigger on p2", {
    ["p2", { [] call skdfx_fnc_throw; true }] call skd_fnc_on;
}] call skd_fnc_step;
```

```
fail    US-028-on-p2  (3.4 s)
        script-error: A script error on p2 in step "trigger on p2": Undefined variable in expression: _skdfx_undefined (skdfx\addons\main\functions\fn_throw.sqf, line 4). Fix the error in the mod, or wrap the code that should raise it in skd_fnc_expectError.
```

The result file lists every error of the execution in the check's `errors`, each with:

| Field | Holds |
|---|---|
| `machine` | The machine it was raised on. |
| `text` | The engine's message without its `Error ` prefix, such as `Undefined variable in expression: _x` or `Generic error in expression`. |
| `file`, `line` | The file as the engine names it, without a leading backslash, and the line in it; for code that has no file, such as code a check sends to a machine, the file is empty and the line counts from the code's start. |
| `stack` | One `file:line` per frame, outermost first. The frames' variables are left out. |
| `check_id`, `step_path` | The check and the step the error was raised in. |
| `source` | `handler` when the machine's handler sent it, `rpt` when only the machine's RPT showed it (below). |
| `expected` | Whether an `skd_fnc_expectError` block matched it. |
| `at` | The clocks of the machine when it was raised: wall clock, `diag_tickTime`, mission time and frame. |

**Which step.** On the lead the error belongs to the step running there. On another machine it belongs to the step the code it ran in was sent from, through `skd_fnc_on`, a wait or an assertion. An error raised there in code nothing sent, such as a function the server reached with `remoteExec`, belongs to the step running on the lead at that moment, or, between two steps, to the last step that started.

**What counts.** Errors from the start of the check's reset to a second after its body ended are listed; those raised during the reset are listed but do not fail the check, and neither does a compile error in code the check sent, which ends the check `check-error` itself. When something else failed the check first, such as an assertion, that cause stands and the error is listed after it.

**What the engine reports.** The engine reports an undefined variable only in scheduled code: the check body, the code a check sends to a machine, and code started with `spawn` or `remoteExec`. The same mistake in unscheduled code, such as an event handler, `isNil` or `remoteExecCall`, is no error at all and leaves nothing to catch. A type error, such as `"a" + 1`, is reported in both.

**Code that ends in an assignment.** A step, a block of `skd_fnc_expectError` and code sent to a machine may end in an assignment, such as `skd_count = skd_count + 1`; its value is then nothing, which the check API takes as `nil`.

## Expecting an error

A check of a mod's error path says which error it expects with `skd_fnc_expectError [text, code, args]` (`ERR-04`). Every error on any machine whose text contains `text`, case-insensitively, from the start of `code` to the end of the check, is marked `expected` and fails nothing:

```sqf
["p2 rejects the bad call", {
    ["_skdfx_undefined", {
        ["p2", { [] call skdfx_fnc_throw; true }] call skd_fnc_on;
    }] call skd_fnc_expectError;
}] call skd_fnc_step;
```

Any other error still fails the check. When no error matched by the check's end, a check that would otherwise pass ends `fail` / `expected-error-missing`, quoting the text, with `evidence.failed_step` the step the block was called in.

## Errors only the RPT shows

The runner also reads each machine's lines for every check from its RPT and adds every script error they show that no handler error of that machine with the same text and file accounts for (`ERR-02`), with `source` `rpt`. Its `at` gives the wall clock to the second, from the RPT's local time, and `tick`, `time` and `frame` 0, which the RPT does not give. Such an error fails the check like any other. A line written to the RPT by other means than the engine's error report, such as `diag_log text "Error in expression <...>"`, is one.

## Each machine's log, with the run

Every machine writes its RPT to its profile folder in the session folder (`status` shows each machine's `profile`). At the start and end of a run, and of each check execution, every machine writes a marker line into its RPT:

```
 7:15:18 [shakedown] r-20261106-071512-1#3 start 2026-11-06T07:15:18.412Z
```

The key is the run id, or `<run id>#<n>` for the run's `n`th check execution. The run folder keeps the lines between each pair, redacted (`LOG-04`):

```
.shakedown/runs/<run id>/
  result.json
  logs/<machine>.log          each machine's lines for the whole run
  checks/<n>/<machine>.log    each machine's lines for check execution n, from its reset to its end
```

The result file names them: `logs` for the run, and each check's `evidence.logs`, by machine. The engine writes a line to the RPT about half a second after a script writes it, so the runner waits for each end marker to reach the file before it cuts the lines.

## Reading logs: `shakedown logs`

`shakedown logs --machine <m>` prints a machine's log lines, redacted (`LOG-03`). It works while a run is active and while the session is broken.

| Option | Takes the lines from |
|---|---|
| none | The machine's RPT in the running session, up to now. |
| `--run <id>` | That run's `logs/<machine>.log`. |
| `--check <id>` | The check's lines: while it executes, its lines so far; otherwise its last execution in the newest run that has one, or in the run `--run` names. |
| `--grep <regex>` | Only the lines the regular expression matches. |
| `--last <n>` | The last `n` lines of those, default 200. |

```
shakedown logs --machine p2 --check US-028-on-p2 --grep "Error"
shakedown logs --machine server --last 50 --json
```

With `--json` it prints `{"lines": [...], "truncated": ..., "path": ...}`: `truncated` is `true` when more lines matched than it printed, and when the lines are longer than 20,000 characters it prints the last ones that fit and `path` names the file in the session folder that holds them all.

## Evidence of a failed check

A check that does not pass keeps in its result (`RES-03`, `OBS-04`):

- `reason`: the kind, a message, the machine it concerns and the wall-clock time of the cause. A wait at its ceiling names what was awaited and the ceiling, such as `skd_fnc_waitUntil: "flag reached p1" was not met on p1 within its ceiling of 3 s.`; a wait given no ceiling ends at 10 s.
- `evidence.failed_step`: the path of the step it failed in; for a script error, the error's step.
- `evidence.observed`: the last value the failed wait or assertion read on each machine it polled.
- `evidence.attached`: what the check attached with `skd_fnc_evidence`.
- `evidence.logs`: each machine's lines for the check, kept for every outcome.
- `errors`: every script error, expected or not.

## The Steam account stays out

Every game process runs under its machine name (`-name=p1`), so `name player` on `p1` is `p1`. At every session start the runner reads the names of the Steam account last logged in on the machine, its account name and its persona name, from Steam's `config\loginusers.vdf`, and keeps them in memory only (`INT-STM-04`). Everything the framework returns or stores outside the profile folders, and every RPT of a stopped session, has each Steam ID (a 17-digit number starting `7656119`) replaced by `<steamid>` and each of the two names, as a whole word and case-sensitively, replaced by `<steamname>` (`LOG-02`). That holds also for what a check itself writes out:

```sqf
private _id = ["p1", { [getPlayerUID player, profileNameSteam] }] call skd_fnc_on;
["uid", _id select 0] call skd_fnc_evidence;
```

gives `{"label": "uid", "value": "<steamid>"}` in the result file. A running session's RPTs are written by the game and hold what it writes until the session stops; `logs` redacts them as it reads them.
