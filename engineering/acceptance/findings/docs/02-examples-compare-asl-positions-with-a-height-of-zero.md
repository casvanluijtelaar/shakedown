---
type: Finding
id: m19-2
milestone: M19
title: Examples compare ASL positions with a height of 0, where the session's ground is about 5 m ASL
description: docs/waits-and-assertions.md's `~=` example compares getPosASL with [100, 104, 0] and fails, and the exec and inspect samples show heights of 0, though on the session mission's default world VR a player stands at about 5.0 m ASL.
raised_by: test
for: writer
checks: []
stories: [US-019, US-044]
generated: { by: "claude-code/tester", at: 2026-11-23T20:13:00Z }
addressed:
  - { by: "claude-code/writer", at: 2026-11-23T20:20:15Z, note: "The `~=` example takes `AGLToASL [100, 104, 0]` as its target, a paragraph says `~=` is three-dimensional and the ground of VR is about 5 m ASL, and the exec, expect and inspect samples show heights of about 5. Changed docs/waits-and-assertions.md and docs/asking-a-running-game.md." }
verified: []
---

# Examples compare ASL positions with a height of 0, where the session's ground is about 5 m ASL

This finding comes from checking the rewritten `docs/` after M19 closed, not from a milestone round.

## What the spec promises

The check API takes and returns ASL positions; `docs/scenes-resets-and-cleanups.md`: "Positions in the check API are ASL: convert a position on the ground with `AGLToASL`". `docs/waits-and-assertions.md`, under The operators, gives as an example of `~=`, with `p1` placed at `AGLToASL [100, 104, 0]` as the other examples place it:

```sqf
["p1", { getPosASL player }, "~=", [[100, 104, 0], 0.5], "p1 stands at the wall"] call skd_fnc_expectOn;
```

`docs/asking-a-running-game.md` shows `exec --machine p1 --code "getPosASL player"` answering `[100,100,0.00143862]`, and `inspect` showing the player at `"position":[100,104,0]`.

## What the system does

In a singleplayer session on the default world `VR`, with `p1` placed by `["p1", AGLToASL [100, 104, 0], 0] call skd_fnc_place`:

```
fail    wa/operators  (11.5 s)
        assertion: p1 stands at the wall: expected ~= [[100,104,0],0.5] on p1 within 10 s, last read [100,104,5.00144].
```

The same line with `[AGLToASL [100, 104, 0], 0.5]` passes, and so do the page's `in` and `contains` examples. `shakedown exec --machine p1 --code "getPosASL player"` answers `p1  done  [100,100,5.00144]`, on a dedicated session too, and `inspect` shows `"position":[100,100,5.00144]`. The build agrees with the contract: the ground of `VR` is at about 5 m ASL.

## What it costs

A test author who copies the `~=` example gets a failing assertion on a correct mod, and the samples teach that a position on the ground has height 0 ASL. The example needs `AGLToASL` on its target, and the samples a height of about 5.

## The check that holds it

None: a document gap. Recorded in `acceptance/coverage/docs.md`.
