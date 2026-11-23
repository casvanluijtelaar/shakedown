---
type: Finding
id: m19-4
milestone: M19
title: The ported check in docs/porting-a-suite.md stands p1 4 m from a crate whose action reaches 3 m
description: The rewritten check places the crate at [100, 108] and p1 at [100, 104], so with an Open crate action of radius 3 m, as the fixture and docs/handing-over.md have it, skd_fnc_useAction fails at its ceiling.
raised_by: test
for: writer
checks: []
stories: [US-019]
generated: { by: "claude-code/tester", at: 2026-11-23T20:13:00Z }
addressed: []
verified: []
---

# The ported check in docs/porting-a-suite.md stands p1 4 m from a crate whose action reaches 3 m

This finding comes from checking the rewritten `docs/` after M19 closed, not from a milestone round.

## What the spec promises

`docs/acting-through-the-mod.md`: "An action shows only while the player is within its radius, measured from the player to the object's centre: place the player well inside it, not at its edge", and a player out of reach ends `fail` / `assertion` naming the distance and the radius. `docs/handing-over.md` uses a crate "whose action has a radius of 3 m" and stands `p1` 2 m from it for that reason. `docs/porting-a-suite.md`, Rewriting one check, gives its check as the result of a port that reaches the old suite's verdict:

```sqf
private _crate = ["mymod_Crate", AGLToASL [100, 108, 0], 0] call skd_fnc_create;
["p1", AGLToASL [100, 104, 0], 0] call skd_fnc_place;
...
["p1", _this select 0, "Open crate"] call skd_fnc_useAction;
```

## What the system does

The page's check, with the fixture's `skdfx_Crate` (an `Open crate` action of radius 3 m, `spec/contract/fixture-mod.md`) for `mymod_Crate`, in a singleplayer session of a setup named `sp-cba`:

```
fail    port/US-012-unhurt  (11.8 s)
        slow step: use the crate through the mod's action (10.0 s)
        assertion: skd_fnc_useAction: p1 could not use "Open crate" within 10 s: the player was 4.01 m from skdfx_Crate 0:19, outside the radius 3 m of "Open crate".
```

The build does what `ACT-01` and the acting page say. The old `init.sqf` on the page also stands the player 4 m away, so the example keeps the old suite's distance.

## What it costs

A person porting a suite copies the example and gets a failure, on the page that tells them a ported check must reach the old suite's verdict. Standing `p1` 2 m from the crate, as the other pages do, makes it pass.

## The check that holds it

None: a document gap. Recorded in `acceptance/coverage/docs.md`.
