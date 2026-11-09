---
type: Research
title: "Spike SPK-06: the runner outlives its client and dies with its game"
description: "How Claude Code holds and ends a stdio MCP server on Windows, which way of starting the runner leaves the client's job and process tree, that a dedicated session started from one Claude Code session is found ready by the next, and how fast the game ends when the runner is killed, on the test machine in M5."
status: stable
generated: { by: "claude-code/developer", at: 2026-11-09T11:10:00+01:00 }
sources:
  - { id: probe, resource: "a stdio MCP server in Python, written for this spike, run under `claude -p` (Claude Code 2.1.319) with `--mcp-config`, recording its job object and starting three children by the three ways of INT-WIN-02, on the Windows test machine, 2026-11-09", read: 2026-11-09 }
  - { id: tests, resource: "crates/shakedown/tests/m05.rs, ses_08_us_007_the_runner_outlives_the_host_ending_shakedown_mcp, spk_06_us_007_a_session_started_through_claude_code_outlives_it and us_035_nfr_rl_03_the_runner_killed_during_a_run_leaves_no_game_5_of_5, runs of 2026-11-09 on the Windows test machine, Arma 3 2.22 stable", read: 2026-11-09 }
---

# Spike SPK-06: the runner outlives its client and dies with its game

M5 ran `SPK-06` (`spec/06-open-questions.md`) on the Windows test machine with Claude Code 2.1.319 (`claude -p`, headless), the project's `.mcp.json` of `e2e/grappling-hook` passed with `--mcp-config`, and the install folder on `PATH`. The spike passed: the runner is started through the relay of `INT-WIN-02`, with WMI as its fallback, and no scheduled task is needed.

## How Claude Code holds and ends a stdio server

A probe server, a stdio MCP server in Python, recorded the job object it ran in and started three long-lived children: one with `DETACHED_PROCESS` alone, one also with `CREATE_BREAKAWAY_FROM_JOB`, and one through WMI's `Win32_Process.Create`[^probe].

- The server runs in a job object with the limits kill-on-close, breakaway allowed, silent breakaway allowed and die-on-unhandled-exception (`LimitFlags` 0x3C00): the job libuv gives every child process on Windows. Silent breakaway means the server's own children are not in that job at all.
- When the Claude Code session ends, it closes the server's standard input; the probe saw the end of its input and exited by itself. All three children were still running after Claude Code had exited, each ended afterwards by its id.

So under Claude Code a plain detached start already survives; `SES-08` still asks for the two harder ways a host can end a client, which the framework's test reproduces.

## Which start leaves the client's job and tree

`ses_08_us_007_the_runner_outlives_the_host_ending_shakedown_mcp` starts `shakedown mcp`, has it start the runner, then ends it the way a host would, and checks the runner still runs and answers `status` with the same process id[^tests]:

| How the host ends `shakedown mcp` | How the runner was started | Runner after the host ended the client |
|---|---|---|
| Closes a kill-on-close job that holds it, breakaway forbidden | WMI | running |
| Closes a kill-on-close job that holds it, breakaway allowed | breakaway into the relay, then WMI from the relay | running |
| `taskkill /T /F` of its process tree, no job of its own | WMI | running |

A breakaway leaves only the innermost job. A job around the client's own that forbids breakaway still holds the relay; the agent shell this spike ran in is such a job, which is why the relay went on by WMI in the second row and the client went straight to WMI in the third. The relay therefore tries breakaway and falls back to WMI itself. The runner's parent is then the relay, which has exited, or the WMI service: no walk of the client's process tree reaches it. A WMI start takes about 0.4 s, most of it PowerShell starting, inside the 10 s the client waits for `runner.json`.

**Forces:** `INT-WIN-02` describes the relay, `SHAKEDOWN_RUNNER_RELAY=1` in its environment so the command line stays `shakedown runner`, and WMI as the fallback of both the client and the relay. Each start writes the way it went to `runner.log`. The scheduled-task fallback of `SPK-06` is not needed.

## The session outlives the Claude Code session that started it

`spk_06_us_007_a_session_started_through_claude_code_outlives_it`[^tests]: a first `claude -p` session called `session_start` with the setup `dedicated` of `e2e/grappling-hook` (a server and two game clients with CBA_A3 and arma-grappling-hook) and ended. No `shakedown mcp` process was left; the runner, started by breakaway into the relay and on by WMI, ran on and the session became `ready` 55 s after the start. A second `claude -p` session called `status` and replied with the same session id and `ready`. The M2 check `US-009-dedicated-freed-rope` then passed against that session from the command line, and its result file names the session.

## The runner killed: the game ends with it

In the same test the runner was killed with `taskkill /F` by its process id: every game process of the session had ended 0.1 s later, through the job object that holds them with kill-on-close (`STP-01`). The mission copy in `MPMissions` was still there until the next command started a runner, whose recovery (`P-12`) deleted it and wrote `runner-died` into the dead session's `session.json`.

`us_035_nfr_rl_03_the_runner_killed_during_a_run_leaves_no_game_5_of_5` kills the runner five times during a run, on a dedicated session with a server, a client and a headless client, each runner started by WMI from the agent shell[^tests]. Every game process had ended 0.07 to 0.08 s after each kill, 5 of 5 (`NFR-RL-03`); each time `status` then showed the runner `not-running` and no session, and the next `session start` cleared the dead session's mission copy and marked it `runner-died`.

In the first run of that test the fifth runner, started by WMI, ended by itself 40 s into its session start, before its client `p1` connected, and left no line in `runner.log` and no entry in the Windows event log; its game processes ended with it as they should. The second run passed 5 of 5. A process WMI creates is in no job object, checked with `IsProcessInJob`, so the WMI service does not end it. The cause is open; the runner writes a panic to `runner.log`, so a recurrence shows whether it was one.

[^probe]: A Python stdio MCP server run under `claude -p` with `--mcp-config`, 2026-11-09, scratch files of the M5 developer session.
[^tests]: `crates/shakedown/tests/m05.rs`, run with `cargo test -p shakedown --test m05 -- --ignored --nocapture --test-threads=1` after `python tools/install.py`.
