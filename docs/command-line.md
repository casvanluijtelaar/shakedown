---
type: Documentation
title: The command line
description: Every shakedown command and option with its defaults and limits, its MCP twin and what it prints, the common options --project and --json, how Ctrl+C stops a run, and the exit codes of every command.
status: draft
---

# The command line

One binary, `shakedown`, does everything. Every command except `list`, `guide`, `version` and `mcp` talks to the runner; `status` asks a runner that runs and starts none, and every other such command starts one when needed ([Sessions and the runner](sessions-and-the-runner.md)). The contract is [spec/contract/cli.md](../spec/contract/cli.md).

## Common options

| Option | Meaning |
|---|---|
| `--project <dir>` | Use this project folder, instead of the nearest folder from the current one upward that holds `shakedown.toml`. |
| `--json` | Print one JSON document on stdout and nothing else: the structured content of the MCP tool named for the command below. Messages and progress lines go to stderr. |
| `--help` | Show the commands, or a command's options. |

Without `--json`, output is short lines of text for a person. Its wording may change; its information does not. Write scripts against `--json`.

## Commands

| Command | Does | MCP tool |
|---|---|---|
| `shakedown status` | The runner, the session and its machines with each machine's health, the active run and, with `--json`, the last reload. Shows a failed start until the next `session start` or `session stop`. Works with no runner, and with one that does not answer. | `status` |
| `shakedown session start --setup <name>` | Checks the environment, starts a session and returns when it is `ready` or `failed` ([Starting a session, and when it fails](starting-and-failed-starts.md)). | `session_start` |
| `shakedown session stop` | Stops the session and returns when nothing of it runs. During a run, stops the run first. Clears a failed start that `status` still shows, and exits 0. | `session_stop` |
| `shakedown list [<id>...] [--tag <tag>]...` | Lists checks from the files, with no runner, each selected check once. A check with a header mistake is listed with it. | `list_checks` |
| `shakedown run [<id>...] [--tag <tag>]... [--setup <name> \| --each-setup] [--repeat <n>]` | Runs the selection and returns when the run ends, printing one line per check execution ([Running checks](running-checks.md)). With `--setup` and no session, starts one and stops it after, unless a check handed it over. `--each-setup`, with no session, runs in each setup a selected check names. `--repeat` runs each check `n` times. | `run_start`, `run_status`, `run_result` |
| `shakedown run stop` | Stops the active run and returns once its result file is written ([Running a suite unattended](unattended-runs.md)). | `run_stop` |
| `shakedown reset` | Resets the session in place ([Resetting by hand](scenes-resets-and-cleanups.md#resetting-by-hand)). | `reset` |
| `shakedown reload [<file>...]` | Reloads changed files at the level they need; on a broken session, restarts the game ([Reloading changed mod files](reloading.md)). | `reload` |
| `shakedown exec --machine <m> (--code <sqf> \| --file <path>) [--args <json>] [--ceiling <s>]` | Runs SQF on a machine or a group and prints the value and any script errors it raised. | `exec` |
| `shakedown wait --machine <m> --condition <sqf> [--description <text>] [--args <json>] [--ceiling <s>]` | Waits for a condition on one machine. | `wait_until` |
| `shakedown expect --machine <m> --value <sqf> --op <op> --expected <json> [--args <json>] [--ceiling <s>]` | Tries an assertion live, as `skd_fnc_expectOn` would. | `expect` |
| `shakedown inspect --machine <m> [--near <netId>] [--radius <m>]` | A compact state summary of a machine. | `inspect` |
| `shakedown logs --machine <m> [--run <id>] [--check <id>] [--grep <regex>] [--last <n>]` | A machine's log lines, redacted. | `logs` |
| `shakedown errors [--machine <m>] [--run <id>] [--check <id>]` | Script errors caught in the session. | `errors` |
| `shakedown guide` | How to use Shakedown and write a check, in short. | `guide` |
| `shakedown version` | The framework version, as `shakedown 0.2.0`. | - |
| `shakedown mcp` | Runs the MCP server on stdio ([AI agents and the MCP tools](agent-tools.md)). | - |

`shakedown runner` runs the runner itself. The other commands start it; it is not for direct use.

[Asking a running game](asking-a-running-game.md) shows `exec`, `wait`, `expect`, `inspect`, `logs` and `errors` at work.

## Option defaults and limits

| Option | Default | Limits |
|---|---|---|
| `run --repeat` | 1 | 1 to 100 |
| `run --setup`, `run --each-setup` | - | not both |
| `exec --ceiling` | 10 | above 0, at most 600 |
| `wait --ceiling`, `expect --ceiling` | 10 | above 0 |
| `inspect --radius` | 50 | metres |
| `logs --last` | 200 | lines |
| `--args`, `--expected` | - | JSON: `--args` an array, `--expected` any value |
| `--machine` | - | `server`, `p1`..`pN`, `hc1`..`hcN`; `exec` also `all`, `clients`, `headless` |

`--op` is one of `==`, `!=`, `<`, `<=`, `>`, `>=`, `~=`, `in`, `contains`.

The command line has no 60 s limit per call; `run`, `session start`, `reload` and `exec` wait up to their own ceilings. The MCP tools have one ([AI agents and the MCP tools](agent-tools.md#what-every-tool-keeps-to)).

## Stopping a run from the console

A run started by `shakedown run` is stopped when the command receives a Ctrl+C or a Ctrl+Break console event, as `run stop` does, from the moment the command starts, also while the session it starts is starting. The command then prints the run's lines and summary, stops a session it started, and exits by the outcomes. A further Ctrl+C or Ctrl+Break is taken as the same stop. A program that drives the command line sends Ctrl+Break to the command's process group.

`run` with the one word `stop` and no other option is always `run stop`, never a check named `stop`; such a check runs with `--tag` or beside another id.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Done. For `run`: every executed check passed. `run --each-setup` exits with the highest code of its setups' runs. |
| `1` | `run`: at least one check `fail` (a `flaky` repeat summary included). |
| `2` | `run`: no check `fail`, and at least one `invalid`, the run was stopped, or a check was not run because its session failed to start or broke. |
| `3` | The request is wrong: an unknown command, option, check, machine or setup, a malformed project file, SQF that does not compile. The message says what to fix. |
| `4` | The request is right but cannot be done now: no session, a session already running, a precondition of the start failed, a session start that ended `failed`, a reload that ended `failed`, a reset that failed, the session is broken or busy with a run, a reload or a reset, `run stop` with no run active, the runner refused, the runner does not answer or runs another version. The message says what to do. |
| `5` | The framework itself failed. The message gives the path of `runner.log`. |

`exec`, `wait` and `expect` exit 0 when they got an answer, whatever the answer: `wait` that ended at its ceiling and `expect` that did not hold report it in their output, with exit code 0. `status` exits 0 also when the runner does not answer.
