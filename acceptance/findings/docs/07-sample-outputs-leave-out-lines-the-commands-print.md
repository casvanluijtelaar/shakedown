---
type: Finding
id: m19-7
milestone: M19
title: Sample outputs leave out lines the commands print, and show a session mission name a client does not report
description: The samples of session start, run --setup and run --each-setup leave out the progress lines the commands print, the quickstart's status sample leaves out the health line, and inspect of a client on a dedicated session reports the mission __cur_mp, not shakedown_<session id>.
raised_by: test
for: writer
checks: []
stories: [US-001, US-044, US-075]
generated: { by: "claude-code/tester", at: 2026-11-23T20:13:00Z }
addressed:
  - { by: "claude-code/writer", at: 2026-11-23T20:20:15Z, note: "The session start, run --setup and run --each-setup samples show the started and ready lines (and the setup's starting line, which goes to stderr), the quickstart's status shows the health line, and the inspect sample of a client shows mission __cur_mp with a line on what each machine reports. Changed docs/quickstart.md, docs/sessions-and-the-runner.md, docs/every-setup-and-repeats.md, docs/asking-a-running-game.md and README.md." }
verified: []
---

# Sample outputs leave out lines the commands print, and show a session mission name a client does not report

This finding comes from checking the rewritten `docs/` after M19 closed, not from a milestone round. The wording of text output may change (`docs/command-line.md`), but its information and shape are what a new user compares against.

## What the spec promises

- `docs/quickstart.md` step 5 shows `run --setup sp` printing only the check line and the summary, and step 6 shows `session start --setup sp` printing only `session s-... ready (setup sp)`, and `status` printing `p1 client ready start 1 process ... profile ...` with no line under it. `docs/sessions-and-the-runner.md` shows the same `status` with a `health:` line under each machine.
- `docs/every-setup-and-repeats.md` shows `run --each-setup` printing a check line and a `setup <name>: ...` line per setup.
- `docs/asking-a-running-game.md` shows `inspect --machine p1` on a dedicated session (`s-20261106-092156`) with `mission shakedown_s-20261106-092156 (start 1)`.

## What the system does

Each command, with stdout and stderr together as a terminal shows them:

```
> shakedown session start --setup sp
session s-20261123-194734: started p1 (process 15628)
session s-20261123-194734: ready
session s-20261123-194734 ready (setup sp)
> shakedown status
runner: running (process 10000)
session: s-20261123-194734 of setup sp, ready
  p1      client    ready      start 1 process 15628 profile ...\machines\p1-1
          health: 149.9 frames a second, answers in 0.008 s, mission time 2.1 s, frame 1278
> shakedown run --each-setup --tag smoke
setup sp: starting its session for run r-20261123-200226-28
session s-20261123-200226: started p1 (process 4920)
session s-20261123-200226: ready
pass    smoke  (1.9 s)
setup sp: 1 passed, ...
```

`run --setup` prints the `started <machine> (process <id>)` and `ready` lines before its check lines too. On a dedicated session `inspect --machine p1` printed `mission __cur_mp (start 1)`, while `inspect --machine server --json` gave `"mission": "shakedown_s-20261123-200311"`.

## What it costs

A new user comparing the screen with the quickstart sees lines the page does not mention and wonders whether something went wrong; the samples should show the progress lines (or say they go to stderr), the quickstart's `status` the health line, and the `inspect` sample the mission name a client reports.

## The check that holds it

None: a document gap. Recorded in `acceptance/coverage/docs.md`.
