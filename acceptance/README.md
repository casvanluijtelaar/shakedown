# Acceptance suite

The acceptance suite belongs to the test developer. It drives the built framework through its public interface (the SQF check API, the command line, the agent tools and the result file) against the game, the way the documentation says a person drives it, and checks what each user story promises. It is written from the contract (`spec/`, `docs/`, `plan/milestones/`) and never from the source.

## Layout

- `docs/` - how the harness was built and what it learned (OKF bundle).
- `findings/` - one Finding concept per defect or document gap, in `findings/mNN/`. The format is in [findings/format.md](findings/format.md).
- `coverage/` - one Coverage concept per milestone: the result of the last full run.
- `run.py` - the entry point of the harness.
- `checks/` - the SQF checks the harness runs, written as a test author writes them.
- `harness/` - the Python that drives the command line and judges the results.

## Running it

What it needs: Windows with Arma 3 and Steam running and logged in, Python 3 (nothing beyond the standard library), and an installed framework (`python tools/install.py`, which fills `target\shakedown\`). No game, server or runner may be running when it starts: it refuses with exit code 4 if one is.

From the repository root:

```
python acceptance/run.py                 # every check, once
python acceptance/run.py --repeat 3      # the in-session checks three times in one game session (the flake check)
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
| `--repeat <n>` | Run the in-session checks `n` times in one game session. Default 1. |
| `--install <dir>` | The install folder holding `shakedown.exe`. Default: the `SHAKEDOWN_INSTALL` variable, else `shakedown` on `PATH`, else `target\shakedown\`. |
| `--game <dir>` | The Arma 3 folder. Default: the `SHAKEDOWN_GAME` variable, else `C:\Program Files (x86)\Steam\steamapps\common\Arma 3`. |
| `--work <dir>` | Where the generated project, `harness.log` and `report.json` go. Default: a new folder in the temp folder; its path is printed. |

What it does: it writes a fresh project (a `shakedown.toml` with a singleplayer setup `sp` and no mods, and a copy of `checks/`) into the work folder. Checks of the phase `offline` run first, with no game. If any `session` or `end` check is selected, it starts one session with `shakedown session start --setup sp`, runs the selected checks in it, stops it with `shakedown session stop`, and then runs the `end` checks. Every game process the session started is stopped by its id if it outlives the stop, and the suite waits up to 75 s for the runner it started to exit by itself before stopping it by id. A run that starts the game takes about two minutes; the `P-01` check alone waits about a minute for the runner to exit.

Output: one line `PASS` or `FAIL` per check and repeat, with the reason under each failure, and a last line with the counts and the path of `report.json`. Exit code 0 when every selected check passed, 1 when one failed, 2 for an unknown option, 3 when no check name matches or `shakedown.exe` is missing, 4 when a game or runner already runs.

## Who reads what

- The **test developer** owns everything in this folder.
- The **framework developer** reads this README and `findings/`, nothing else. It runs the checks a finding names through the filter, which is not reading them, and writes only `addressed` entries.
- The **technical writer** writes only findings for the framework developer and `addressed` entries on document gaps.
