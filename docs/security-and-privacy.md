---
type: Documentation
title: Security and privacy
description: How Shakedown keeps control of a session to the local user with loopback connections, tokens and a process check, records every piece of code sent to the game in audit.jsonl, keeps the Steam account out of everything it writes, keeps the project clean, and what it does not yet cover.
status: draft
---

# Security and privacy

Shakedown runs arbitrary SQF in your game, so it keeps control to the local user, records every piece of code it sends, and keeps the Steam account out of everything it writes.

## Only the local user controls the session

- **Loopback only.** The runner and the extension in each game process talk only over `127.0.0.1`; `netstat -ano` shows every socket of the runner, and every connection of a game process to it, on `127.0.0.1`. Nothing in `shakedown.exe` listens on another address, so Windows Firewall never asks about it. The free-port check of a dedicated start reads the system's UDP table and binds nothing. The MCP server talks over stdio.
- **Tokens.** Every client presents the runner token from `runner.json` on each connection, and every game process's extension presents its session token, which the runner passes to the process in its environment only, never on a command line. Neither token appears on any process's command line, in the result file, in a log the framework writes or in any command or tool output.
- **Token form.** Both have a fixed form, `skd_r_` or `skd_s_` followed by 64 lowercase hexadecimal digits, so a search for `skd_[rs]_[0-9a-f]{64}` finds a leak. `runner.json` is the one file that holds the runner token. Its access list is protected, so it takes nothing from its folder, and grants the current user alone: no other account or group, the Administrators group included, can read it.
- **Process check.** An extension's hello is also checked against the process at the other end of its connection, read from the system's TCP table: only the process the runner started for that machine of that session is let in, so a program that copied the session token still cannot pose as a game process.

Anything else is closed without a reply, runs no code, and adds a line to `%LOCALAPPDATA%\Shakedown\runner.log`:

```
refused client connection from pid 13940: bad-token
refused extension connection from pid 13940: bad-token
refused extension connection from pid 13940: unknown-process
refused client connection from pid 13940: not-hello
```

## The game's own servers

Every server the framework starts has `loopback = 1` and BattlEye off, is named `shakedown <session id>`, and every client is told to connect to it alone. Code a check sends that makes a client join another server ends that client's process and breaks the session ([A client sent to another server](starting-and-failed-starts.md#a-client-sent-to-another-server)).

## Every piece of code sent to the game is recorded

Each session folder holds `audit.jsonl`, one JSON object per line for every `exec` order sent to a machine: its `time`, its `source`, its `target` machine and its `code`. The source is `cli` for `shakedown exec`, `mcp` for the `exec` tool, and the check id for a check: the check's body, sent to the lead, and each piece of code the body sends to another machine with `skd_fnc_on`, a wait or a polled assertion. Code the body runs on the lead itself is part of the body's line. An `exec` to a group has one line per machine.

```
{"code":"6 * 7","source":"cli","target":"server","time":"2026-11-09T10:12:04.511Z"}
{"code":"6 * 8","source":"mcp","target":"hc1","time":"2026-11-09T10:12:05.093Z"}
{"code":"private _v = [\"p1\", { 40 + 2 }] call skd_fnc_on; ...","source":"US-052-on-p1","target":"server","time":"2026-11-09T10:11:58.020Z"}
{"code":"40 + 2","source":"US-052-on-p1","target":"p1","time":"2026-11-09T10:11:58.090Z"}
```

The code is redacted like everything else the runner stores. `runner.log` holds process ids, ports, session ids and paths, and is redacted the same way.

## The Steam account stays out

Every game process runs under its machine name (`-name=p1`), so `name player` on `p1` is `p1`, not your Steam name. At every session start the runner reads the names of the Steam account last logged in on the machine, its account name and its persona name, from Steam's `config\loginusers.vdf`, and keeps them in memory only.

Everything the framework returns or stores outside the profile folders, and every RPT of a stopped session, has:

- each Steam ID (a 17-digit number starting `7656119`) replaced by `<steamid>`;
- each of the two names, as a whole word and case-sensitively, replaced by `<steamname>`.

That holds also for what a check itself writes out:

```sqf
private _id = ["p1", { [getPlayerUID player, profileNameSteam] }] call skd_fnc_on;
["uid", _id select 0] call skd_fnc_evidence;
```

gives `{"label": "uid", "value": "<steamid>"}` in the result file. A running session's RPTs are written by the game and hold what it writes until the session stops; `logs` redacts them as it reads them. The reason of a failed start, in `status` and in `session.json`, is redacted too.

So a run folder can be shared, with one exception: crash dumps under `dumps\` are copied as the engine wrote them and may hold the Steam ID and names.

## Your project stays clean

Shakedown writes in your project only `.shakedown\`, whose `.gitignore` of `*` keeps every session, run, log, dump and long tool result out of git. Outside the project it writes the runner's files in `%LOCALAPPDATA%\Shakedown\`, the mission copy of a dedicated session in the game's `MPMissions` folder (deleted when the session stops), the links `<game>\@shakedown` and, for reloads, `<game>\skd_release`.

## Not covered yet

- Crash dumps in a run folder are not redacted.
- A Steam name that is also a common word, or a machine name, is replaced wherever that word appears.
- The code `wait`, `expect` and `inspect` send, and the condition of `wait_until`, are not recorded in `audit.jsonl`.
