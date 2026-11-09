---
type: Deferred
title: "A singleplayer reset after the player died may leave the death camera running"
description: "After a reset in which RST-05 replaced a dead singleplayer p1, a later check failed script-error on Undefined variable tsw_camtargetunit errors on p1, so the game's death-camera script may still run after the reset."
id: DEF-14
state: open
milestone: null
status: stable
---

# A singleplayer reset after the player died may leave the death camera running

## Summary

`RST-05` keeps a singleplayer session alive when its player dies: the agent's `Killed` handler hands the player a new unit and the reset deletes the body. In M4's first full acceptance run, a later check in the same singleplayer session failed `script-error` on errors that came from neither the check nor the mod. The likeliest source is the death camera the game starts when the player dies, still running after the player was replaced.

## Current behaviour

`fn_killed.sqf` creates a unit of the dead player's class in the dead unit's group, puts it where the old one fell, and calls `selectPlayer` on it; the dead unit goes into `skd_created`, so the reset deletes it. Nothing ends a script the game started on the player's death. In the acceptance round's first full run, in the `sp` session in which a check had killed the player, the game raised `Undefined variable in expression: tsw_camtargetunit` on `p1` three times during the 55 s check of `US-050`. The framework listed the errors in that execution and failed it `script-error`, as `ERR-03` says (`acceptance/coverage/m04.md`, "Runs that were not green on the first try"). No variable of that name exists in the framework or the fixtures. The later full runs, 171 of 171, did not show it.

## The gap

A check that passes on its own can fail `script-error` because an earlier check in the session killed the player. That breaks the promise that the reset puts every check on the same start (`RST-01`), and the result blames the check.

## Options

1. Find the script that uses `tsw_camtargetunit`, and have the reset, or `fn_killed.sqf`, end it and destroy its camera once the new unit is the player.
2. Make the replacement before the engine starts the death camera, if an earlier event than `Killed` allows it.
3. List errors raised by the game's own scripts, outside the session's mods, without failing the check.

## Recommendation

The first. It keeps `ERR-03` as it is and fixes the reset, where the leftover comes from. Do it in M9, which builds the reset's failures, or earlier if it shows again.

## Acceptance criteria

1. In a singleplayer session, a check that kills `p1` and then 20 checks that each wait 30 s all pass, and no `tsw_camtargetunit` error is in any result file or in `p1`'s RPT.
2. After that reset, `inspect` of `p1` lists no script the death left running.
