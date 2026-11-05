# Acceptance suite

The acceptance suite belongs to the test developer. It drives the built framework through its public interface (the SQF check API, the command line, the agent tools and the result file) against the game, the way the documentation says a person drives it, and checks what each user story promises. It is written from the contract (`spec/`, `docs/`, `plan/milestones/`) and never from the source.

## Layout

- `docs/` - how the harness was built and what it learned (OKF bundle).
- `findings/` - one Finding concept per defect or document gap, in `findings/mNN/`. The format is in [findings/format.md](findings/format.md).
- `coverage/` - one Coverage concept per milestone: the result of the last full run.
- `run.py` - the entry point of the harness.
- `checks/` - the SQF checks the harness runs, written as a test author writes them.
- `templates/` - SQF checks the harness writes into the project in numbered copies, such as the 20 checks of `NFR-RL-02`.
- `harness/` - the Python that drives the command line and judges the results.

## Running it

What it needs: Windows with Arma 3 and Steam running and logged in, Python 3 (nothing beyond the standard library), an installed framework with its fixture mods (`python tools/install.py`, which fills `target\shakedown\` and links the fixtures into the game folder), CBA_A3, and a built arma-grappling-hook for the M1 demo. No game, server or runner may be running when it starts: it refuses with exit code 4 if one is.

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

Options:

| Option | Meaning |
|---|---|
| `--check <text>` | Run only checks whose name contains this text. Repeat it for several. |
| `--repeat <n>` | Run the in-session checks `n` times in each game session. Default 1. |
| `--install <dir>` | The install folder holding `shakedown.exe`. Default: the `SHAKEDOWN_INSTALL` variable, else `shakedown` on `PATH`, else `target\shakedown\`. |
| `--game <dir>` | The Arma 3 folder. Default: the `SHAKEDOWN_GAME` variable, else `C:\Program Files (x86)\Steam\steamapps\common\Arma 3`. |
| `--cba <dir>` | The CBA_A3 mod folder. Default: the `SHAKEDOWN_CBA` variable, else the Steam Workshop folder of item 450814997. |
| `--agh <dir>` | arma-grappling-hook's built mod folder. Default: the `SHAKEDOWN_AGH` variable, else `C:\Users\Shadow\Documents\arma-grappling-hook\.hemttout\build`. |
| `--work <dir>` | Where the generated project, `harness.log` and `report.json` go. Default: a new folder in the temp folder; its path is printed. |

What it does: it writes a fresh project into the work folder: a copy of `checks/`, 20 numbered copies of `templates/NFR-RL-02-start-state.sqf`, and a `shakedown.toml` with two singleplayer setups. `sp` loads only the fixture `@skd_fixture`, so it is a setup without CBA; `cba` loads CBA, both fixtures and arma-grappling-hook. Checks of the phase `offline` run first, with no game. Then, for each setup a selected `session` check needs (`cba` first, then `sp`), it starts one session with `shakedown session start --setup <name>`, runs that setup's checks in it and stops it with `shakedown session stop`. The `end` checks run after the last session, which is always `sp`. Every game process a session started is stopped by its id if it outlives the stop, and the suite waits up to 75 s for the runner it started to exit by itself before stopping it by id. A full run takes about four minutes; the `P-01` check alone waits about a minute for the runner to exit.

Output: one line `PASS` or `FAIL` per check and repeat, with the reason under each failure, and a last line with the counts and the path of `report.json`. Exit code 0 when every selected check passed, 1 when one failed, 2 for an unknown option, 3 when no check name matches or `shakedown.exe` is missing, 4 when a game or runner already runs.

## Who reads what

- The **test developer** owns everything in this folder.
- The **framework developer** reads this README and `findings/`, nothing else. It runs the checks a finding names through the filter, which is not reading them, and writes only `addressed` entries.
- The **technical writer** writes only findings for the framework developer and `addressed` entries on document gaps.
