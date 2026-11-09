---
type: Specification
title: Command line
description: The contract for the shakedown command line - the project file, every command with its options, its human and JSON output, and the exit codes.
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# Command line

One binary, `shakedown`, does everything. Every command except `list`, `guide`, `version` and `mcp` talks to the runner. `status` asks a runner that runs and starts none; every other command starts it when needed (`P-01`). The words used here are defined in [../00-overview.md](../00-overview.md).

## The project file: `shakedown.toml`

The project is the nearest folder, from the current folder upward, that holds `shakedown.toml`, or the folder given with `--project`. Relative paths in the file are relative to the project folder. An unknown key, a missing required key or a value of the wrong type is refused with exit code 3, naming the key and its line (`CFG-01`).

```toml
game = 'C:\Program Files (x86)\Steam\steamapps\common\Arma 3'   # required
checks = "checks"      # default "checks"
base_port = 2402       # default 2402; the server uses it and the next four

[mods.cba]
path = 'C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\450814997'
patch = "cba_main"     # optional: the CfgPatches class whose version the result records

[mods.agh]
path = 'C:\Program Files (x86)\Steam\steamapps\common\Arma 3\z\agh'   # the linked dev build
patch = "agh_main"
source = "addons"      # optional: makes the mod watched by reload; the folder holding its addon folders
prefix = 'z\agh\addons'  # required with source: the game path of that folder

[setups.sp]
mode = "singleplayer"
mods = ["cba", "agh"]  # mod names, loaded in this order, before the framework's addon

[setups.dedicated-2]
mode = "dedicated"
clients = 2            # game clients, default 1; always 1 in singleplayer
headless = 0           # headless clients, default 0; always 0 in singleplayer
mods = ["cba", "agh"]
world = "VR"           # default "VR"
spawn = [100, 100]     # default [100, 100]: where p1 starts; pN starts 10 m further east per N
start_timeout = 180    # default 180 seconds (SES-04)
```

| Key | Type | Required | Meaning |
|---|---|---|---|
| `game` | path | yes | The Arma 3 installation folder. |
| `checks` | path | no | The checks folder. |
| `base_port` | integer 1024-65530 | no | The server's game port. |
| `mods.<name>.path` | path | yes | The mod's folder, as `-mod` takes it. |
| `mods.<name>.patch` | string | no | The `CfgPatches` class whose `versionStr` each machine reports. |
| `mods.<name>.source` | path | no | The folder of the mod's addon sources that `reload` watches. |
| `mods.<name>.prefix` | string | with `source` | The game path of `source`, for matching files to functions. |
| `setups.<name>.mode` | `"singleplayer"` or `"dedicated"` | yes | |
| `setups.<name>.clients` | integer 1-8 | no | |
| `setups.<name>.headless` | integer 0-8 | no | |
| `setups.<name>.mods` | array of mod names | yes | |
| `setups.<name>.world` | string | no | The world's class name in `CfgWorlds`. |
| `setups.<name>.spawn` | `[x, y]` | no | |
| `setups.<name>.start_timeout` | integer 30-1800 | no | |

A setup name and a mod name are letters, digits and `-`, starting with a letter.

## Common options

| Option | Meaning |
|---|---|
| `--project <dir>` | Use this project folder. |
| `--json` | Print one JSON document on stdout and nothing else: the structured content of the MCP tool named for the command below. Messages go to stderr. |

Without `--json`, output is short lines of text for a person; its wording is not part of the contract, its information is.

## Commands

| Command | Does | MCP twin |
|---|---|---|
| `shakedown status` | The runner, the session and its machines, the active run, the last reload (`INS-02`, `INS-04`). A start that failed is shown, state `failed` with its reason, until the next `session start` or `session stop`. Works with no runner, and with a runner that does not answer. | `status` |
| `shakedown session start --setup <name>` | Starts a session and returns when it is `ready` or `failed` (`SES-01`). | `session_start` |
| `shakedown session stop` | Stops the session and returns when nothing of it runs (`SES-07`). On a failed start that `status` still shows, clears it and exits 0. | `session_stop` |
| `shakedown list [<id>...] [--tag <tag>]...` | Lists checks from the files, with no runner, each selected check once (`RUN-02`). | `list_checks` |
| `shakedown run [<id>...] [--tag <tag>]... [--setup <name>]` | Runs the selection against the session and returns when the run ends, printing one line per check execution as it ends. An id given more than once runs that check once per time it is given, in the order given (`RUN-03`); the line of its second and later executions carries `#<repeat>` right after the check id, before the duration. With `--setup` and no session, starts one and stops it after (`RUN-04`). With `--setup` naming another setup than the running session's, refuses. | `run_start`, then `run_status` and `run_result` |
| `shakedown run stop` | Stops the active run (`RUN-05`). | `run_stop` |
| `shakedown reset` | Resets the session (`RST-03`). | `reset` |
| `shakedown reload [<file>...]` | Reloads changed mod code (`RLD-01`). Files are paths under a watched `source`. | `reload` |
| `shakedown exec --machine <m> (--code <sqf> \| --file <path>) [--args <json>] [--ceiling <s>]` | Runs SQF on a machine or a group and prints the value and any script errors it raised (`CTL-01`, `CTL-05`). `--ceiling` default 10, above 0, at most 600. | `exec` |
| `shakedown wait --machine <m> --condition <sqf> [--description <text>] [--args <json>] [--ceiling <s>]` | Waits for a condition on a machine (`OBS-01`). | `wait_until` |
| `shakedown expect --machine <m> --value <sqf> --op <op> --expected <json> [--args <json>] [--ceiling <s>]` | Tries an assertion live, as `skd_fnc_expectOn` would, without a check (`OBS-06`). | `expect` |
| `shakedown inspect --machine <m> [--near <netId>] [--radius <m>]` | A compact state summary of a machine (`INS-01`). `--radius` default 50. | `inspect` |
| `shakedown logs --machine <m> [--run <id>] [--check <id>] [--grep <regex>] [--last <n>]` | A machine's log lines, redacted (`LOG-03`). `--last` default 200. | `logs` |
| `shakedown errors [--machine <m>] [--run <id>] [--check <id>]` | Script errors caught in the session (`ERR-05`). | `errors` |
| `shakedown guide` | How to use Shakedown and write a check, in short (`AGT-06`). | `guide` |
| `shakedown version` | The framework version. | - |
| `shakedown mcp` | Runs the MCP server on stdio (`AGT-01`). | - |
| `shakedown runner` | Runs the runner. Started by the other commands; not for direct use. | - |

The command line has no ceiling of 60 s per call; `run`, `session start`, `reload` and `exec` wait up to their own ceilings. A run started by `shakedown run` is stopped when the command receives a Ctrl+C or a Ctrl+Break console event, as `run stop` does (`RUN-05`); a program that drives the command line sends Ctrl+Break to the command's process group.

## The runner's file and connection

The command line and the MCP server reach the runner through these; a test of `SEC-01` and `SEC-03` reaches it the same way.

- `%LOCALAPPDATA%\Shakedown\runner.json` is `{"port": integer, "token": string, "pid": integer, "started_at": Time}`, readable by the current user only (`INT-WIN-03`). `started_at` is the start time of the process `pid`; a client takes the file to name a live runner only while a process with that id runs that started in the same second, and otherwise deletes it and starts a runner (`P-01`). `pid` is also in `status` as `runner_pid`.
- Tokens have the form of `SEC-06`: the runner token `skd_r_` and a session token `skd_s_`, each followed by 64 lowercase hexadecimal digits.
- The runner listens on `127.0.0.1:<port>`. Each message is one UTF-8 JSON object on one line. The first message on a connection is a hello: `{"hello": "client", "version": string, "token": runner token}` from a client, or `{"hello": "extension", "session": session id, "machine": machine name, "token": session token}` from a game process's extension.
- The runner answers a client's hello with `{"hello": "runner", "version": string}`. A client that gets no answer within 5 s, or an answer with another version than its own, starts no other runner and refuses the request with exit code 4, naming the runner's process id and, if it answered, its version, and saying that ending that process ends its session with it; `status` reports the runner `unreachable` instead and exits 0 (`P-01`).
- A connection whose first message is not a hello, whose token is wrong, or, for an extension, whose other end is not the process the runner started for that machine of that session (`SEC-03`), is closed without a reply, runs no code, and adds one line to `%LOCALAPPDATA%\Shakedown\runner.log`: `refused <client or extension> connection from pid <pid>: <not-hello, bad-token or unknown-process>`.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Done. For `run`: every executed check passed. |
| `1` | `run`: at least one check `fail`. |
| `2` | `run`: no check `fail`, at least one `invalid`. |
| `3` | The request is wrong: an unknown command, option, check, machine or setup, a malformed project file, SQF that does not compile. The message says what to fix. |
| `4` | The request is right but cannot be done now: no session, a session already running, a precondition of `SES-03` failed, a session start that ended `failed`, a reload that ended `failed`, the session is `broken` or busy with a run, a reload or a reset, the runner refused, the runner does not answer or runs another version. The message says what to do. |
| `5` | The framework itself failed. The message gives the path of `runner.log`. |

`exec`, `wait` and `expect` exit 0 when they got an answer, whatever the answer: `wait` that ended at its ceiling and `expect` that did not hold report it in their output, with exit code 0.
