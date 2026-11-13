---
type: Deferred
title: "skd_fnc_pressKey does not return what the keybind's code returned"
description: "skd_fnc_pressKey returns true once the keybind's code ran, so a check cannot read through it whether the key was consumed or passed on to the game, and calls the keybind's code itself with skd_fnc_on to assert that."
id: DEF-26
state: open
milestone: null
status: stable
---

# skd_fnc_pressKey does not return what the keybind's code returned

## Summary

A CBA key handler that returns `false` passes the key on to the game, and one that returns `true` consumes it. A mod can promise either, as arma-grappling-hook's `CTL-01` does: off a rope each of its keys passes through. `skd_fnc_pressKey` (`ACT-02`) returns `true` once the keybind's code ran, so that promise is not readable through it. Raised by the M10 port as finding `m10-3`.

## Current behaviour

`skd_fnc_pressKey [machine, mod, action, mode]` calls the key-down code, the key-up code or both, and returns `true`. `docs/check-api.md` shows the way round it: the check calls the code `CBA_fnc_getKeybind` returns, with `skd_fnc_on` on the player's machine, and reads its value, which is the same path `skd_fnc_pressKey` takes.

## The gap

A check that asserts whether a key is consumed repeats the key lookup of `skd_fnc_pressKey` in its own SQF, about ten lines, and depends on the layout of CBA's keybind array.

## Options

1. Return the code's value: for `"down"` and `"up"` the value of that code, for `"press"` the pair `[down, up]`. A handler that returns nothing gives `nil` in its place. A check that tests the return of `"press"` against `true` breaks.
2. Keep `true`, and add a mode such as `"down-value"` that returns the down code's value.
3. Leave the documented way round it.

## Recommendation

The first option, when a second mod needs it: it is a change to the check API contract and to `ACT-02`, and it costs a few lines in the function on the pressing machine. Until then the documented way works and reaches the same code.

## Acceptance criteria

1. With the fixture's `skd_fixture` / `signal` keybind changed to return `false` from its key-down code, `["p1", "skd_fixture", "signal", "down"] call skd_fnc_pressKey` returns `false`, in singleplayer and on a client of a dedicated session.
2. `"press"` returns the down and the up code's values, as the contract gives it.
3. `docs/check-api.md` drops the way round it.
