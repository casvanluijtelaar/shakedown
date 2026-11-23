---
type: Finding
id: m19-5
milestone: M19
title: An unhealthy flag of a machine's earlier start makes every check invalid after a reload restarted its game
description: After `shakedown reload` restarted the game of a session broken while p1 was flagged mission-time, every check ended invalid / unhealthy citing that old flag, though status showed the new p1 (start 2) with flag null at 150 frames a second.
raised_by: test
for: developer
checks: []
stories: [US-059]
generated: { by: "claude-code/tester", at: 2026-11-23T20:13:00Z }
addressed: []
verified: []
---

# An unhealthy flag of a machine's earlier start makes every check invalid after a reload restarted its game

This finding comes from checking the rewritten `docs/` after M19 closed, not from a milestone round.

## What the spec promises

`US-059` criterion 2: "While the clock stands still, `status` shows `p1` flagged unhealthy; once its clock moves again for 5 s the flag clears and the next check can pass." `docs/outcomes-and-broken-sessions.md`: "Every check that overlaps the flagged interval ends `invalid` / `unhealthy` ... The flag clears once both clocks have moved over a whole 5 s again", and `shakedown reload` on a broken session "always restarts the game of every machine".

## What the system does

In a singleplayer session of setup `sp-cba` (CBA and both fixtures), `p1`'s game stood in its pause menu (`isGamePaused` true, display 49 open; not caused by this session, which sends no input). `status --json` showed `p1` with `flag` `mission-time` since `2026-11-23T19:56:28.142Z`, and the reset before the next check failed, so the session broke. Then:

```
> shakedown reload
reload rl-20261123-195756-8 done: level game-restart in 26.57 s
  reached p1
> shakedown status
session: s-20261123-195222 of setup sp-cba, ready
  p1      client    ready      start 2 process 15816 profile ...\machines\p1-2
          health: 147.1 frames a second, answers in 0.002 s, mission time 32.1 s, frame 7460
> shakedown exec --machine p1 --code "[isGamePaused, diag_fps]"
p1  done  [false,150.943]
```

`status --json` gave `p1` `"flag": null, "since": null`. Every check run after it, over the next 30 s, ended the same way, `out/pass` (one `skd_fnc_expect` of `1 == 1`) among them:

```
> shakedown run out/pass out/pass out/failing
invalid out/pass  (2.0 s)
        unhealthy: p1's mission time stopped at 2026-11-23T19:56:26.105Z while it still answered: over 5.0 s of real time its mission time advanced 2.09 s. ...
invalid out/pass #2  (1.5 s)
        unhealthy: (the same)
invalid out/failing  (1.5 s)
        unhealthy: (the same)
```

The result file of run `r-20261123-195829-9` lists one machine, `p1` start 2, and the check's `reason.at` is `2026-11-23T19:56:26.105Z`, before that start's process was launched at `19:57:56.791Z`. Five runs in a row gave the same. Only `session stop` and a new `session start` made checks pass again: in the new session the same checks passed.

## What it costs

`shakedown reload` is the documented way out of a broken session. After it, every check is `invalid` with a reason about a process that no longer exists, while `status` says the machine is healthy, so a test author cannot get a verdict without restarting the session, and an agent following `status` has no clue why.

## The check that holds it

None yet: found while running the docs' examples; the transcript is in `acceptance/coverage/docs.md`. A check would flag a clock (`skdfx_fnc_holdClock` held past a reset failure, or a pause), break the session, `shakedown reload`, and run a passing check.
