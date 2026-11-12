# Acceptance suite

The acceptance suite belongs to the test developer. It drives the built framework through its public interface (the SQF check API, the command line, the agent tools and the result file) against the game, the way the documentation says a person drives it, and checks what each user story promises. It is written from the contract (`spec/`, `docs/`, `plan/milestones/`) and never from the source.

## Layout

- `docs/` - how the harness was built and what it learned (OKF bundle).
- `findings/` - one Finding concept per defect or document gap, in `findings/mNN/`. The format is in [findings/format.md](findings/format.md).
- `coverage/` - one Coverage concept per milestone: the result of the last full run.
- `run.py` - the entry point of the harness.
- `checks/` - the SQF checks the harness runs, written as a test author writes them.
- `templates/` - SQF checks the harness writes into the project in numbered copies, such as the 20 checks of `NFR-RL-02`.
- `walk/` - the three SQF checks of the M8 walk-away suite, which the harness copies into a project of their own.
- `m09/` - the SQF checks of M9, which the harness copies into the project `m09` of their own.
- `harness/` - the Python that drives the command line and judges the results.

## Running it

What it needs: Windows with Arma 3, including the Altis terrain, and Steam running and logged in, Python 3 (nothing beyond the standard library), an installed framework with its fixture mods (`python tools/install.py`, which fills `target\shakedown\` and links the fixtures into the game folder), CBA_A3, and a built arma-grappling-hook for the M1 demo. No game, server or runner may be running when it starts: it refuses with exit code 4 if one is.

From the repository root:

```
python acceptance/run.py                 # every check, once
python acceptance/run.py --repeat 3      # the in-session checks three times in each game session (the flake check)
python acceptance/run.py --list          # print every check's name and phase, start nothing
```

To run only the checks a finding names, give each name from the finding's `checks` list with `--check`. A check runs when its name contains any `--check` value, so a story ID such as `--check US-038` selects every check of that story:

```
python acceptance/run.py --check "US-038 a body with a syntax error ends fail check-error with the compile message and its line"
```

One check needs Steam stopped: `US-003 SES-03 INT-STM-01 with no steam process ...`, of the phase `nosteam`. A run without `--check` leaves it out; it runs, before the `start` checks, only when a `--check` value selects it, and fails at once while a `steam` process runs. Stop Steam by its process id before it and start and log in to Steam after it:

```
python acceptance/run.py --check "US-003 SES-03 INT-STM-01"
```

One check starts headless Claude Code sessions, the first criterion of `US-007` under Claude Code itself: `US-007 SES-06 SES-08 INT-CC-02 a headless Claude Code session ...`, of the phase `claude`. A run without `--check` leaves it out; it runs, before the `start` checks, only when a `--check` value selects it. It needs `claude` on `PATH`, logged in, and Steam. In `e2e/grappling-hook` it starts `claude -p` with `--mcp-config` naming that project's `.mcp.json` and `--strict-mcp-config`, with the install folder on `PATH`; it registers nothing in any user configuration. The first session starts the `dedicated` setup through `session_start` and ends; a second one calls `status`. Each is bounded, and the check stops the session, the runner and any `shakedown mcp` left by their process ids:

```
python acceptance/run.py --check "a headless Claude Code session"
```

The M5 `start` checks start their own sessions and end the runner they started by its process id: four `sp` sessions through `shakedown mcp`, ended by a host in four ways, and six `ded` sessions, five of them ended by `taskkill /F` of the runner during a run. They wait up to 75 s for a runner that an earlier check left to exit by itself first.

One criterion runs on the Mac, with no game and no Steam: the third of `US-046`. `run_mac.py` writes a project of its own with the `.mcp.json` of `docs/agent-tools.md`, starts `shakedown mcp` from it with the install folder on `PATH`, speaks JSON-RPC to it on stdio and judges `status`, `list_checks` and `guide`, and that the server starts no other process. It needs Python 3 and the `shakedown` binary, built as `docs/install.md` gives it for macOS and Linux: `cargo build --release -p shakedown` puts it in `target/release/`. Run it from the repository root, once per server start:

```
python3 acceptance/run_mac.py --install <folder holding shakedown>   # default: target/release/
```

`--repeat <n>` starts the server `n` times; `--work <dir>` names the folder for the project, `harness.log` and `report.json`. Exit code 0 when every repeat passed, 1 when one failed, 3 when the binary is missing.

Options of `run.py`:

| Option | Meaning |
|---|---|
| `--check <text>` | Run only checks whose name contains this text. Repeat it for several. |
| `--repeat <n>` | Run the in-session checks `n` times in each game session. Default 1. |
| `--install <dir>` | The install folder holding `shakedown.exe`. Default: the `SHAKEDOWN_INSTALL` variable, else `shakedown` on `PATH`, else `target\shakedown\`. |
| `--game <dir>` | The Arma 3 folder. Default: the `SHAKEDOWN_GAME` variable, else `C:\Program Files (x86)\Steam\steamapps\common\Arma 3`. |
| `--cba <dir>` | The CBA_A3 mod folder. Default: the `SHAKEDOWN_CBA` variable, else the Steam Workshop folder of item 450814997. |
| `--agh <dir>` | arma-grappling-hook's built mod folder. Default: the `SHAKEDOWN_AGH` variable, else `C:\Users\Shadow\Documents\arma-grappling-hook\.hemttout\build`. |
| `--work <dir>` | Where the generated project, `harness.log` and `report.json` go. Default: a new folder in the temp folder; its path is printed. |

What it does: it writes a fresh project into the work folder: a copy of `checks/`, 20 numbered copies of `templates/NFR-RL-02-start-state.sqf`, 21 of `templates/US-008-start-state.sqf`, a `.mcp.json` as `docs/agent-tools.md` gives it, a staged copy of `@skd_fixture_cba`'s installed sources in `stage\`, and a `shakedown.toml` with six setups. `sp` loads only the fixture `@skd_fixture`, so it is a setup without CBA; `cba` loads CBA, both fixtures and arma-grappling-hook; `ded` is a dedicated server with two clients and a headless client loading the same mods as `cba`; `ded30` is `ded` with a start ceiling of 30 s, which it cannot meet; `slow` is a dedicated server with one client on Altis, whose start takes longer than the 50 s a `session_start` call waits; `rld` is a dedicated server with two clients and a headless client loading CBA and both fixtures, each watched by reload: `@skd_fixture` in its installed copy, `@skd_fixture_cba` in the staged copy with a `rebuild` that copies the staged `fnc_version.sqf` into the installed copy, as `spec/contract/fixture-mod.md` gives it; `rld7` is a dedicated server with two clients and `rld7sp` singleplayer, each loading `@skd_fixture` alone, watched in a staged copy of its installed sources in `stage\` with a `rebuild` that copies the staged `fn_version.sqf` into the installed copy. Checks of the phase `offline` run first, with no game and no runner; they need no runner left from an earlier run. Then the `start` checks run with no session: those of `US-004` each start a `ded` or `ded30` session that fails, those of M4 refuse starts, start an `sp` session and a `slow` session through the `session_start` tool and stop them, and each stops what it left by process id. Then, for each setup a selected `session` check needs (`ded`, then `rld`, `rld7`, `rld7sp`, then `cba`, then `sp`), it starts one session with `shakedown session start --setup <name>`, runs that setup's checks in it and stops it with `shakedown session stop`. The M6 checks of `rld` change the installed copy of the fixtures only through the `rebuild` during a reload and by creating `fn_new.sqf`, and put both back before the next session starts. The M7 checks of `rld7` and `rld7sp` change the installed `fn_version.sqf` only through the `rebuild` during a reload, and put it back before the next session starts. The `end` checks run after the last session, which is always `sp`. The M8 `start` checks run after the other `start` checks, once per `--repeat`, each with no session running and in a project of its own in the work folder: `walk\` with the checks of `walk/` and the setup `dedicated-2` (a dedicated server with two clients loading `@skd_fixture` alone), and `ceil\` with eight copies of `templates/US-063-ceiling.sqf` and the setup `sp`. Per repeat they make `run --setup dedicated-2` run to its end, the same run stopped by Ctrl+C in its second check, `run --setup sp` of the ceiling checks, `session start` and `session stop`, and `session_start` and `session_stop` through the MCP server; each part that a game start loses to `Error 20 reading file` of a base-game file starts again, at most three times in all. The M8 batch of the `sp` session stops three checks in the second by Ctrl+C, by Ctrl+Break, by `shakedown run stop` and by `run_stop`, and runs the eight ceiling checks from the command line and through `run_start`. Ctrl+C and Ctrl+Break go through `harness/console_signal.py`, which starts `shakedown run` in a hidden console of its own and sends the event there, Ctrl+C to that console and Ctrl+Break to the command's process group; nothing goes to the desktop. The M9 `start` checks run after those of M8, once per `--repeat`, each part with no session running, in the project `m09` of their own in the work folder, which holds the checks of `m09/` and the setups `ded` (a dedicated server with two clients) and `sp`, each loading `@skd_fixture` alone. Per repeat five parts each start and stop a session: on `ded`, `p2` suspended by its id with `NtSuspendProcess` while a check waits on it, then the broken session asked and `p1` suspended too; `p2` crashed by `skdfx_fnc_crash` during a wait; `p2` crashed just after an assertion failed; and `p2` ended with `taskkill /F` by its id during a wait; on `sp`, the three outcomes, `p1`'s clock held by `skdfx_fnc_holdClock`, a cleanup that never ends, `shakedown reload` of the broken session and a `shakedown reset` that fails. A suspended process is resumed by its id before its session stops, and a part lost to `Error 20 reading file` starts again, at most three times in all. Every game process a session started is stopped by its id if it outlives the stop, and the suite waits up to 75 s for the runner it started to exit by itself before stopping it by id. The agent tools are driven over their protocol: the harness starts `shakedown mcp` from the project's `.mcp.json`, with the install folder on `PATH`, and speaks JSON-RPC to it on stdio; only the `claude` check starts Claude Code. A full run takes about fifty minutes; the `P-01` check alone waits about a minute for the runner to exit.

Output: one line `PASS` or `FAIL` per check and repeat, with the reason under each failure, and a last line with the counts and the path of `report.json`. Exit code 0 when every selected check passed, 1 when one failed, 2 for an unknown option, 3 when no check name matches or `shakedown.exe` is missing, 4 when a game or runner already runs.

## Who reads what

- The **test developer** owns everything in this folder.
- The **framework developer** reads this README and `findings/`, nothing else. It runs the checks a finding names through the filter, which is not reading them, and writes only `addressed` entries.
- The **technical writer** writes only findings for the framework developer and `addressed` entries on document gaps.
