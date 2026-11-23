---
type: Finding
id: m19-6
milestone: M19
title: On a dedicated session p1 starts at spawn plus 10 m in some sessions, in p2's place
description: In two of four fresh dedicated sessions with two clients, p1 started at [110, 100] and p2 at [100, 100], and the reset put each back there, though the session mission puts pN at spawn x plus 10 (N - 1).
raised_by: test
for: developer
checks: []
stories: [US-008]
generated: { by: "claude-code/tester", at: 2026-11-23T20:13:00Z }
addressed: []
verified: []
---

# On a dedicated session p1 starts at spawn plus 10 m in some sessions, in p2's place

This finding comes from checking the rewritten `docs/` after M19 closed, not from a milestone round.

## What the spec promises

`INT-AR-04`, the session mission: "for each `pN`, the late clients of `MCH-01` included, a playable rifleman (`B_Soldier_F`, side WEST) in a group of its own, at `[spawn x + 10 × (N - 1), spawn y]` on the ground". `docs/scenes-resets-and-cleanups.md`: "with `p1` at the setup's `spawn` (`[100, 100]` by default) and each further client `pN` 10 m further east per N", and `docs/project-file.md`, `spawn`: "Where `p1` starts on the world; `pN` starts 10 m further east per N". `US-008`: every check starts from the session's start state.

## What the system does

A project with `[setups.dedicated]` `mode = "dedicated"`, `clients = 2`, `mods = []`, default `spawn`. Three fresh sessions, each read right after `session start` and again after `shakedown reset`:

```
> shakedown exec --machine p1 --code "[isGamePaused, getPosASL player]"      (session s-20261123-200941)
p1  done  [false,[110,100,5.00144]]

> shakedown exec --machine clients --code "getPosASL player"                 (next session, before and after reset)
p1  done  [110,100,5.00144]
p2  done  [100,100,5.00144]

> shakedown exec --machine clients --code "getPosASL player"                 (next session, before and after reset)
p1  done  [99.9962,100.007,5.00144]
p2  done  [110,100,5.00144]
```

In a fourth dedicated session (setup with two clients, a headless client and a late client) `inspect --machine p1` showed `p1` at `[100,100,5.00144]` after a reset. So which client stands at the spawn point changes from one session start to the next, and the reset keeps whatever start each client got.

## What it costs

A check that leans on the documented start positions, such as one that leaves `p2` where it starts 10 m east of `p1`, or reads `p1` at the spawn point, passes in one session and fails in the next on a correct mod, which is the flakiness the framework exists to remove. The docs' own advice to place each player is the only guard.

## The check that holds it

None yet: found while running the docs' examples; the transcript is in `acceptance/coverage/docs.md`. A check would read `getPosASL player` on `p1` and `p2` after each of several dedicated starts.
