---
type: Documentation
title: How the harness is built
description: The parts of the acceptance harness, how a check is named, phased and judged, and what the first round learned about driving the framework.
status: draft
---

# How the harness is built

The harness follows the last section of `spec/07-tech-stack.md`: the checks that need the game are SQF files written against the check API, and Python drives the `shakedown` command line, reads the result files and judges them. It shares no code with the framework and opens none of its source.

## Parts

| Part | Holds |
|---|---|
| `run.py` | Options, the generated project with its setups, one session per setup around its in-session checks, cleanup by process id, the report. |
| `harness/core.py` | The check registry (`@check(name, phase, needs, setup)`), the batch registry (`@batch(setup)`), the setups in the order their sessions run, and the shared state of a run: the session, every run and its result file. |
| `harness/driver.py` | Calls `shakedown` with a ceiling, reads `--json` output, lists and stops processes by id with `tasklist` and `taskkill`, reads `runner.json`. |
| `harness/schema.py` | A validator for the JSON Schema 2020-12 keywords `result.schema.json` uses. It refuses a schema with a keyword it does not know, so a new keyword in the contract shows up as a harness error. Written here instead of taking `jsonschema` as a dependency. |
| `harness/m00.py` | The checks of M0 and their batch. |
| `harness/m01.py` | The checks of M1 and their batches. |
| `harness/m02.py` | The checks of M2, their batches, and the `US-004` starts that fail. |
| `harness/m03.py` | The checks of M3 and their batch, the reading of the Steam identity at run time, and the RPT reading of `INT-AR-07` that the `end` checks of `US-030`, `US-064`, `DEF-09` and `NFR-RL-05` use. |
| `harness/mcp.py` | An MCP client over stdio, as Claude Code is one: it starts the server the project's `.mcp.json` declares, with the install folder on `PATH`, speaks newline-delimited JSON-RPC to it, keeps every progress notification with its time, cancels a call, and judges a tool result (`structured`, `tool_error`). Written here instead of taking an MCP SDK as a dependency. |
| `run_mac.py` | The Mac half of M4: `US-046`'s third criterion with no game, through `harness/mcp.py`, which stops the server by its id with `taskkill` on Windows and by its process on macOS. |
| `harness/m04.py` | The checks of M4 and their batches: the tools offline, the refused and slow starts, and the CLI and tool answers of the `ded` and `sp` sessions. |
| `checks/` | The SQF checks: one folder per story, `m01/` for M1's requirements, `sel/` for the selection, `agh/` for the M1, M2 and M3 demos against arma-grappling-hook. |
| `templates/` | SQF checks written into the project in numbered copies. |

## Checks

A check is a Python function with a name that starts with the story or requirement ID it proves. Its phase says when it runs:

- `offline`: once, before any game starts. Commands that need no runner, such as `list`.
- `start`: once, before the first session. It starts what it needs itself, such as a session start that must fail, and stops every game process it left by its id.
- `session`: once per repeat, inside the session of its setup (`sp` or `cba`). The setups are `ded`, `cba` and `sp`, in that order. Each check names the SQF checks it reads (`needs`). Per repeat each module's batch for that setup makes its runs: the union of what the selected checks need, in a fixed order that puts every broken check before the check that must pass after them, plus the runs a check needs on its own (the 20 checks of `NFR-RL-02`, the four selections of `RUN-01`, the four selections with a repeated id and one text run of `m01-3`, the demo run). Then every selected check is judged against those result files.
- `end`: once, after the last session stopped.
- `nosteam`: needs no `steam` process running, so a run leaves it out unless a `--check` value selects it; then it runs before the `start` checks. It is the second criterion of `US-003`.

A failed assertion fails only its own check; the others are still judged. A harness error is reported as a failure of the check it happened in.

## What the first round learned

- A singleplayer session starts in about 40 s on the test machine and a run of eleven checks takes a tenth of a second, so the flake check costs one start.
- `run --json` gives `result_file` as an absolute path; the run folder is its parent.
- A header mistake is reported with no `machine` in its reason and no steps, which is how the checks see that the game was not touched.
- Stopping the runner by id leaves its `runner.json` behind; the next command starts a new runner over it. The harness therefore waits for the runner to exit by itself first.

## What the M1 round learned

- Two sessions, `cba` and `sp`, cost about 40 s each to start; everything inside them takes about 40 s more, most of it the two `US-039` timeouts and the 20 checks of `NFR-RL-02`.
- `run` given an id more than once executes it once per time (`m01-3`), so the demo's ten passes are one run of the demo and the second check, given ten times each, in turn.
- The reset leaves variables on units and in `missionNamespace`, as `docs/check-api.md` says. A check that counts something on the player's unit, such as the fixture's `skdfx_signalDown`, registers a cleanup that puts it back; the `NFR-RL-02` checks keep the start state they compare against in `acc_m01_base` for the same reason.
- arma-grappling-hook's placed hook needs a lip: the demo check finds a container's south lip by casting rays down from above, places `p1` 0.6 m inside it facing out, and calls `agh_fnc_placeHook` from there.

## What the M2 round learned

- A dedicated session of a server, two clients and a headless client is ready in 47 to 50 s; the 21 checks of `US-008` take about 45 s together, and `shakedown reset` after one of them answers in under a second.
- The headless client runs as `arma3server_x64.exe`, like the server. The harness counts both images as game processes.
- `skd_fnc_create` returns once the server has the object. A client receives it a moment later, and first at the place the server created it before moving it into position. A check that reads the object's position on a client, as the demos do to find the container's lip, waits until that client sees it in place, looking it up by net id each frame.
- arma-grappling-hook's server checks that the unit carries the placed hook, so the demos wait until the server sees the hook in `p1`'s backpack before placing it.
- arma-grappling-hook gives a respawned unit none of its actions again, in a dedicated session also without a reset. No check of the `ded` session kills `p1`, so its demos do not fail on the mod. The `US-009` check of the `ded` session kills `p2`.
- `skdfx_flag` survives the reset, as `fixture-mod.md` says, so the `US-025` and `US-026` checks set it at their start.
- `status` answers at once while a start runs in another command, and before the runner has the session it gives `session` `null`. The `US-004` checks poll it twice a second. Once a start fails, `status` gives `session` `null` again, before `session start` has returned.
- `session start` writes its output as it ends, and the output of a failed start passes the 4 KB a Windows pipe holds, as `docs/dedicated-sessions.md` says. A command the harness starts in the background has its stdout and stderr read by two threads while it runs; with pipes nobody reads, a start that failed at 31 s returned only when the harness stopped polling, at about 104 s.

## What the M3 round learned

- The checks that need the Steam ID or the Steam names read them at run time, as `INT-STM-04` says the runner does, from `loginusers.vdf` under the registry's `SteamPath`. A message of the harness names a hit only as "the Steam ID", "a Steam ID" or "the Steam name", never the value, so `harness.log` and `report.json` hold neither.
- A script error that only the RPT shows is made with `diag_log text`, one line per RPT line of the engine's error: the `Error in expression <...>` line, the indented `Error position:` and `Error <message>` lines, and the `File <file>..., line n` line, whose file the engine writes without a leading backslash.
- The checks whose story says the run exits 1 run alone, so the exit code is theirs; the others share one run.
- `NFR-RL-05` is judged after the sessions stop: each stopped session's RPTs are cut at the check markers, and every error inside a check execution is looked up in that execution's `errors`. Errors outside any execution are counted in `harness.log`; M3 keeps none of them (`ERR-05` is M4's).
- An M3-only run of the ten selections takes about three minutes: the `ded` start, about a minute of runs, and the `sp` start the `end` checks need.


## What the M4 round learned

- The tools are judged against their command line twins by masking only what is read at another moment: durations, clocks, `last_answer_at` and a machine's heartbeat. Everything else of the two answers must be equal.
- A tool result over 20,000 characters is `{"result_path": ...}`: the full `list_checks` of the generated project is one, so a check that compares it reads the file.
- A check that masks or trims an answer works on a copy: the `AGT-02` check validates every stored answer against its tool's `outputSchema` after the other checks are judged.
- A check execution goes on about 1.5 s after its outcome is decided, through the wait for script errors in flight, so "ends at once" is judged from `started_at` to `reason.at`, not by `duration_s`.
- `skdfx_flag` survives the reset, so the `US-023` wait waits for a value of its own execution, not for the flag to exist.
- `compile` of text that is not SQF gives code whose text is that text, with a script error on the lead machine; sent to another machine it does not compile there. That is how a check sends code that does not compile (`US-014`). The lead's error is the check's own and decides `script-error` when it lands first, which it did in three of four executions, so the check wraps the call in `skd_fnc_expectError`.
- A `slow` dedicated start on Altis with one client takes about 100 s, so `session_start` reliably answers `starting` at 50 s. The machine kept about 11 GB free before and after it.
- 100 `exec` calls through the tool take 17 ms per call at the 95th percentile to a singleplayer `p1` or a client, 43 ms to the dedicated server and the headless client.
- A run started through `run_start` has no command answer; the M4 batches add it to the suite's runs from its `run_result`, so the checks over every run (`SES-05`, `NFR-RL-05`) read its result file too.
- In a full run the singleplayer session has served the M0 and M1 checks, `RST-05`'s killed player among them, before the M4 batch. There the game raised `Undefined variable in expression: tsw_camtargetunit` on `p1` three times during the 55 s check of `US-050`, which the framework listed in that execution's errors and failed it `script-error`. The `US-050` checks judge when `run_status` answers, not that check's outcome.
- With no `steam` process, `session start` still brings up a runner, which refuses in 0.1 s and exits by itself about 60 s later. A check that refuses a start hands that runner to the suite as soon as the start answers, so the suite waits for it even when the check fails; a next run that starts while it lives refuses with exit 4.

## What the Mac round learned

- On a Mac the `shakedown` binary alone serves `run_mac.py`: `cargo build --release -p shakedown`, as `docs/install.md` gives it, with Rust 1.99 or later and no HEMTT. `tools/install.py` stops at `hemtt build` there, before it assembles an install folder (`m04-2`). The same cargo build on the Windows test machine gives `target/release/shakedown.exe`, which answers `list`, `guide`, `version`, `status` and the MCP server's `status`, `list_checks` and `guide` with no runner; `run_mac.py` itself refuses to run off a Mac, since it asserts macOS and no Steam.
- The guide is 4,837 characters on the Mac and 4,885 on Windows, the same text with other line endings; the checks compare it with the documented text with whitespace folded.
- With no runner, `status`, `list_checks` and `guide` each answer in under 0.01 s, and the server exits 0 as soon as its stdin closes.
