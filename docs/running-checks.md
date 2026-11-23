---
type: Documentation
title: Running checks
description: How to list checks without the game, run them against a session or with one that starts and stops with the run, select them by id and tag and in which order they run, what the lines printed for each check mean, the exit codes, and what a run leaves behind.
status: draft
---

# Running checks

## Listing checks

```
> shakedown list
rope/US-012-hook-holds  US-012 A hook thrown at a wall holds a player's weight  [US-012 smoke rope]
rope/US-013-hook-misses  US-013 A hook thrown into the air falls back  [US-013 rope]
broken  (header error: line 1: @check must start with the story or requirement ID the check proves, such as US-012 or NFR-PF-02, then a short sentence; it starts with `A`.)
> shakedown list --tag smoke
rope/US-012-hook-holds  US-012 A hook thrown at a wall holds a player's weight  [US-012 smoke rope]
```

`shakedown list [<id>...] [--tag <tag>]...` reads the check files and prints each selected check once: its id, its name and its tags. It needs no game and no runner, and it works on macOS and Linux too. A check whose header is wrong is listed with what is wrong. With `--json` it prints each check's `id`, `name`, `tags`, `setups` (`null` for every setup), `timeout_s`, `reset` and `error`.

## Two ways to run

**With a session of its own.** With no session running, `run --setup <name>` starts a session of that setup, runs, and stops the session at the end, whatever the outcomes:

```
shakedown run --setup sp
```

**Against a running session.** Start a session once, run as often as you like, and stop it when you are done. The session stays up between runs and every run skips the game's start:

```
shakedown session start --setup sp
shakedown run
shakedown run rope/US-012-hook-holds
shakedown session stop
```

`run` without `--setup` needs a running session; with none it exits 4:

```
No session runs. Start one with `shakedown session start --setup <name>`; the project declares the setups: sp, dedicated.
```

`run --setup` naming another setup than the running session's is refused, exit 4:

```
Session s-20261105-161208 of setup sp runs, not one of setup dedicated. Stop it with `shakedown session stop`, or run without --setup against it.
```

A run against a session you started leaves it `ready`, and what the last check left stays in the game until the next reset, so you can look at it. To run in every setup with one command, see [Every setup, repeats and slow steps](every-setup-and-repeats.md).

## Choosing checks

| Command | Runs |
|---|---|
| `shakedown run` | Every check. |
| `shakedown run a b` | The checks `a` and `b`, in that order. |
| `shakedown run --tag smoke` | Every check tagged `smoke`. |
| `shakedown run --tag smoke --tag rope` | Every check tagged `smoke` or `rope`. |
| `shakedown run a --tag smoke` | `a`, then every check tagged `smoke` that is not `a`. |
| `shakedown run a a b` | `a`, `a`, then `b`: an id given twice runs twice. |
| `shakedown run a --repeat 10` | `a` ten times in a row ([Repeating a check](every-setup-and-repeats.md#repeating-a-check)). |

Ids keep the order given, and an id given more than once runs that check once per time: `run a a b` runs `a`, `a`, `b`, and `run a b a` runs `a`, `b`, `a`. Checks a tag selects run after them in id order, each once. An id may be written with `.sqf` or with `\`.

A check whose `@setups` leaves out the session's setup is not run, and is listed in the result as not run with reason `setup`.

An unknown id or a tag no check carries is refused, exit 3, with what exists:

```
No check has the id `rope/US-99`. The checks are: broken, rope/US-012-hook-holds, rope/US-013-hook-misses.
No check carries the tag nosuch. The tags that exist are: US-012, US-013, rope, smoke. Select checks by one of those; `shakedown list` shows each check's tags.
```

`run` with the one word `stop` and nothing else is always `run stop`, which stops the active run; a check named `stop` runs with `--tag` or beside another id ([Stopping a run](unattended-runs.md#stopping-a-run-by-hand)).

## What `run` prints

One line per check execution as it ends, then a summary with the path of the result file:

```
pass    rope/US-012-hook-holds  (13.5 s)
fail    rope/US-013-hook-misses  (10.1 s)
        assertion: the hook fell back: expected == true on p1 within 10 s, last read false.
pass    rope/US-012-hook-holds #2  (13.2 s)
pass    rope/US-078-climb  (14.2 s)
        slow step: climb to the top (12.1 s)
invalid rope/US-079-long  (61.3 s, slow)
        machine-lost: p2 stopped answering: ...
3 passed, 1 failed, 1 invalid, 0 not run. Result: C:\mods\mymod\.shakedown\runs\r-20261105-161317-1\result.json
```

- The outcome, `pass`, `fail` or `invalid`, then the check id and the duration.
- `#<n>` after the id marks the second and later executions of a check given more than once or repeated.
- `slow` after the duration marks an execution that took more than half its `@timeout`; a `slow step:` line names each step longer than the project's `slow_step` ([Slow steps](every-setup-and-repeats.md#slow-steps-and-slow-checks)).
- An indented line gives the reason of an outcome that is not `pass`: its kind and its message ([Outcomes and a broken session](outcomes-and-broken-sessions.md)).
- `handed over to <machine>` marks a check that handed the session to a person ([Handing over to a person](handing-over.md)).

The wording of these lines is for a person and may change; scripts read `--json` instead, which prints the whole result of the run as one JSON document (the MCP tool `run_result`), and sends the lines to stderr.

## Exit codes

| Code | `run` means |
|---|---|
| `0` | Every executed check passed. |
| `1` | At least one check failed. |
| `2` | No check failed, and at least one was `invalid`, the run was stopped, or a check was not run because its session failed to start or broke. |
| `3` | The request is wrong: an unknown command, option, check, machine or setup, a malformed project file. |
| `4` | The request cannot be done now: no session, a session of another setup running, a session start that was refused or failed, a broken or busy session. |
| `5` | The framework itself failed; the message gives the path of `runner.log`. |

A check that is `fail` / `check-error` because the check itself is wrong counts as a failure: exit 1. See [The command line](command-line.md#exit-codes) for every command.

## What a run leaves behind

Each run writes a folder in the project:

```
.shakedown\runs\<run id>\
  result.json               the result file
  logs\<machine>.log        each machine's log lines for the run, redacted
  checks\<n>\<machine>.log  each machine's log lines for the run's nth check execution, redacted
  dumps\<machine>\<file>    the crash dumps a machine lost during the run left
```

The run id has the form `r-<date>-<time>-<n>`, as `r-20261105-161317-1`. [Results and the result file](results.md) explains every field. A session's own folder, `.shakedown\sessions\<session id>\`, holds the generated mission, a dedicated session's `server.cfg`, each machine's profile folder `machines\<machine>-<start>` with its RPT, `audit.jsonl` and, once the session ends, `session.json`. Git ignores everything in `.shakedown\`.

## While a run goes on

- `shakedown status` shows the run's id, how many checks are done and the check it is on.
- `shakedown inspect`, `logs`, `errors` and `exec` work during a run ([Asking a running game](asking-a-running-game.md)).
- `shakedown reload` and `shakedown reset` are refused until the run ends ([Reloading changed mod files](reloading.md#running-a-reload)).
- Ctrl+C in the console of `run`, or `shakedown run stop` from another, stops it ([Stopping a run by hand](unattended-runs.md#stopping-a-run-by-hand)).
