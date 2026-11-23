---
type: Coverage
title: "The rewritten docs: coverage"
description: "The docs rewritten after M19's close, taken as a new user against the framework built from f4f1d90: the quickstart and the README's quickstart ran as written, and about 150 SQF examples and commands of 24 pages ran once each in 14 game starts; most matched, 8 findings (m19-1 to m19-8), two of them for the developer."
status: draft
milestone: M19
run: { at: 2026-11-23T19:46:46Z, sha: f4f1d90 }
results:
  - story: US-095
    state: pass
    checks: ["docs/quickstart.md and README.md quickstart: shakedown version, list, run --setup sp, run --setup dedicated, as written"]
  - story: US-001
    state: pass
    checks: ["docs/quickstart.md step 6: session start, run, status, session stop"]
  - story: US-047
    state: pass
    checks: ["docs/quickstart.md step 9 as written exits 4 (m19-1, a page gap)"]
  - story: US-038
    state: pass
    checks: ["docs/writing-checks.md: every header and body mistake of the table, by list and by run"]
  - story: US-019
    state: partial
    checks: ["docs/acting-through-the-mod.md and docs/dedicated-sessions.md crate checks pass; docs/porting-a-suite.md check fails at its radius (m19-4)"]
  - story: US-028
    state: pass
    checks: ["docs/script-errors-and-logs.md US-028-on-p2, its expectError and expected-error-missing variants"]
  - story: US-059
    state: fail
    checks: ["docs/outcomes-and-broken-sessions.md holdClock example is invalid / unhealthy; after reload of a broken session the old flag stays (m19-5)"]
  - story: US-066
    state: pass
    checks: ["docs/dedicated-sessions.md US-066 late join, leave and join again; wrong join message"]
  - story: US-072
    state: pass
    checks: ["docs/recording-events.md US-072, the second-event wait, the event that does not come, the event not recorded"]
  - story: US-083
    state: pass
    checks: ["docs/handing-over.md US-083 crate hand over, and the refused hand over to the server"]
  - story: US-089
    state: pass
    checks: ["docs/ace-interaction.md US-089 with the fixture's ACE actions, a missing name, a false condition, no ACE, and the exec that lists a menu"]
  - story: US-075
    state: pass
    checks: ["docs/every-setup-and-repeats.md run --each-setup --tag smoke with one setup refused"]
  - story: US-080
    state: pass
    checks: ["docs/project-file.md mistakes by list; the string slow_step message differs from the page (m19-8)"]
  - story: US-008
    state: partial
    checks: ["docs/scenes-resets-and-cleanups.md cleanups, reset by hand and @reset mission pass; p1's start position varies between dedicated sessions (m19-6)"]
---

# The rewritten docs: coverage

The docs were rewritten as a consumer guide after M19 closed (commits `fa9db55`, `16df8df`, `a1d77f4`, `f4f1d90`) by a writer who could run nothing. This round took them as a new user would, against the framework built and installed from `f4f1d90` with `python tools/install.py` (`target\shakedown\`, fixtures linked into the game folder), on 2026-11-23 from 19:44Z to 20:12Z. Every example ran once, with no `--repeat` of the harness. Projects were written in the scratch folder; a page's `mymod` names were replaced by the fixtures' (`skdfx_Crate`, the keybind `skd_fixture` / `signal`, the event `skdfx_signalled`, `skdfx_fnc_throw`, the ACE actions `skdfx_aceOpen`, `skdfx_aceSignal`, `skdfx_aceNever`), as `spec/contract/fixture-mod.md` gives them. Sample durations and ids were taken as illustrative.

Game starts, 14 in all: the quickstart's `run --setup sp`, its session, its `run --setup dedicated`; a singleplayer session of `sp-cba` (CBA and both fixtures), its game restart by `reload`, and a second one; a failed start on a pinned version; a `run --each-setup`; a dedicated session with two clients, a headless client and a late client; a dedicated ACE session; and three short dedicated sessions to recheck `exec` and the clients' start positions. No start was lost to `Error 20 reading file`.

## Document checks of the acceptance suite

The suite's references to moved pages were brought in line: the `US-021 ... m10-3` check name and its SQF comment point at `docs/acting-through-the-mod.md`, two `US-008 m10-2` comments at `docs/scenes-resets-and-cleanups.md`, and finding `m05-1` at `plan/reports/m00.md`. The four `offline` checks that read `docs/` or the README ran once and passed: `AGT-06 the guide tool returns the text docs/agent-tools.md gives under The guide`, `M10 EC8 SPK-14 NFR-PF-06 ...`, `US-098 docs/install.md says to stop the runner ...` and `NFR-LG-04 DST-01 the root README says ...`. The README's quickstart check is byte for byte `acceptance/m19/smoke.sqf`.

## Per page

| Page | Ran | Result |
|---|---|---|
| `quickstart.md` | Steps 1 (`shakedown version` only), 2 to 9 as written, with `mods = []` | All as shown but step 9, which exits 4 (m19-1); progress and health lines not in the samples (m19-7). |
| `README.md` | Quickstart: the same project, `run --setup sp`, `run --setup dedicated`; exit codes | As shown. |
| `install.md` | `shakedown version`; `python tools/install.py` | As shown. |
| `writing-checks.md` | Steps, nested steps, `skd_fnc_player`, code with args, the library example, the broken library file, all 13 mistakes of the table | As shown; nested path `["the crate is unharmed", "check its damage"]`. |
| `running-checks.md` | `list`, `list --tag`, `list --json`, unknown id and tag, `run a a b` (`#2`), refused `--setup` and `--each-setup`, slow step and slow check lines | As shown. |
| `check-api.md` | Context, steps, `on`/`onEach`, waits, assertions, scene, cleanup, evidence, `fail`, `invalid`, wrong argument | As shown but `profileName` on `hc1` (m19-3). |
| `waits-and-assertions.md` | Every wait and assertion example, every failure message, operator error | As shown but the `~=` example (m19-2). |
| `scenes-resets-and-cleanups.md` | The scene, unknown class, cleanups and their effect, `@reset mission`, `shakedown reset`, reset refused during a run | As shown; p1's start varies on dedicated (m19-6). |
| `acting-through-the-mod.md` | `useAction` with the fixture's crate, no title, out of reach, objNull and server on dedicated, `pressKey` press/down/up, unregistered keybind, mod without keybinds, the "Did the key pass on" code with the fixture's keybind | As shown. |
| `ace-interaction.md` | US-089 on `dedicated-ace` (ACE 3.21.2), missing name, false condition, no ACE, the menu listing `exec` | As shown. |
| `recording-events.md` | US-072, the second-event wait, no event, not recorded | As shown. |
| `script-errors-and-logs.md` | US-028-on-p2, expected error, missing error, an RPT-only error, `logs --check --grep`, `logs --json` | As shown; `source` `rpt` for the RPT line. |
| `outcomes-and-broken-sessions.md` | pass/failing/explicit, holdClock, crash of p2 and the broken session (status, inspect, exec, reset, run refused), reload of a broken session | As shown but the stale flag after reload (m19-5). |
| `dedicated-sessions.md` | Start, status with hc1, the both-kinds check, names, crate in place, expectEach and expectAgree failures, US-066, wrong join, leave in singleplayer, reset | As shown but `hc1 -> "hc1"` (m19-3) and start positions (m19-6). |
| `handing-over.md` | US-083 crate hand over with the next check not run, session up, list on p1, refused to server | As shown. |
| `starting-and-failed-starts.md` | Refused start (missing mod and port 2403 held), version 9.9.9 failed start, status, session stop, session.json | As shown. |
| `project-file.md` | Smallest file, `checks`, version without patch, slow_step -1 and "10", start_timeout 5, missing tests folder, undeclared mod | As shown but the "10" message (m19-8). |
| `every-setup-and-repeats.md` | `run --each-setup --tag smoke` with a refused setup, `--repeat 3` summary line, slow lines | As shown. |
| `unattended-runs.md` | `run --setup` leaves nothing; `run stop` from another console with the run's lines; `run stop` with none | As shown; the "Stopping run ..." line did not print for a stop from another console. |
| `reloading.md` | Staged fixture `fn_version.sqf` reload (mission-restart, 7.9 s), `--tag BUG-7` rerun, `--json`, a check file (`none`), a file outside (exit 3), refused during a run; the installed file put back to `"1"` | As shown. |
| `asking-a-running-game.md` | `exec` (value, all, `--json`, `--args`, object args, script error, compile error, unknown machine), `wait` (met, not met, error `--json`), `expect` (held, did not hold, bad operands), `inspect` (sp, dedicated client, server `--near --radius --json`), `errors`, `logs` | As shown but positions (m19-2) and a client's mission name (m19-7); `exec --machine all` prints machines in name order. |
| `sessions-and-the-runner.md` | Start, status, stop, second start refused, status with no runner | As shown. |
| `agent-tools.md` | Over JSON-RPC: tools/list (17 tools, read-only and destructive marks, output schemas), `status`, `list_checks`, `exec` (value, unknown-machine, ceiling 60 bad-arguments), `wait_until`, `expect`, `run_start`/`run_status`/`run_result`, `run_start` session-exists and each_setup with setup, `reset` busy, `inspect` | As shown. |
| `security-and-privacy.md` | `getPlayerUID` and `profileNameSteam` evidence (`<steamid>` on a dedicated client, `_SP_PLAYER_` in singleplayer, `<steamname>`), `audit.jsonl` lines for cli, mcp and a check, `.shakedown\.gitignore` | As shown. |

## Not run, and why

- `install.md` and `README.md`: winget, the one-line script from GitHub and the zip by hand, upgrade and remove: they install from GitHub or change the user's `PATH`; M19's checks cover the local routes. macOS build and `package.py`: no game needed, covered by M19.
- `writing-checks.md`, `check-api.md`, `waits-and-assertions.md`, `acting-through-the-mod.md` examples that need arma-grappling-hook (`agh` keybinds, ropes, `agh_climbState` beyond its default): no fixture equivalent; the `agh_climbState` wait ran and passed on its default.
- `results.md`: the PowerShell snippet that reads a result file. Both attempts were refused by the permission classifier, and it was not worked around.
- `unattended-runs.md`: Ctrl+C and Ctrl+Break, and the runner ceiling of eight checks: M8's checks cover them, and they need a console signal.
- `sessions-and-the-runner.md` and `troubleshooting.md`: the runner killed, unreachable or of another version, Steam stopped or logged out, a hung client start and a foreign server: each needs a destructive or long setup the M5, M9, M16 and US-003 checks cover.
- `reloading.md` on a dedicated session and a `config.cpp` game restart; `@reset mission` on a dedicated session: about 80 s each; singleplayer covered the commands.
- `outcomes-and-broken-sessions.md`: suspending a process, `taskkill`, `skdfx_fnc_exit`, and Escape in a window (no desktop input).
- `porting-a-suite.md`: `run --each-setup` of a ported suite; the single ported check ran.

## Notes

- During the `sp-cba` session `p1`'s game stood in its pause menu for about two minutes (`isGamePaused` true, display 49 open). This session sent no key to any window; the cause is outside the framework. It made checks `invalid` / `unhealthy`, as the docs say a paused game does, and broke the session through a reset that failed; the stale flag after the reload that followed is m19-5.
- `exec --machine all` and `run_status` answered as documented; text sample orders (setups and machines listed by name) differ from the pages' examples but carry the same information.
