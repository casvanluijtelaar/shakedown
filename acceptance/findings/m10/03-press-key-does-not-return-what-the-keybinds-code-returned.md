---
type: Finding
id: m10-3
milestone: M10
title: "`skd_fnc_pressKey` does not return what the keybind's code returned, so a check cannot assert that a key passes through to the game"
description: "skd_fnc_pressKey returns true once the keybind's code ran, so a check cannot assert through it that a key passes through to the game, as arma-grappling-hook's CTL-01 promises off a rope."
status: draft
raised_by: test
for: developer
checks:
  - "US-021 ACT-02 m10-3 the way docs/check-api.md gives to read what a keybind's code returned runs the fixture's signal down code once and reads false from arma-grappling-hook's let go off a rope"
stories: [US-021]
generated: { by: "claude-code/tester", at: 2026-11-13T07:10:46-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-13T08:14:00-08:00, note: "Deferred as DEF-26, outside the MVP: the port reaches the same keybind code through CBA_fnc_getKeybind with skd_fnc_on, and docs/check-api.md now shows that way to read whether a key was consumed; returning the code's value is a change to the check API contract and ACT-02 left for when a second mod needs it. No framework change." }
verified:
  - { by: "claude-code/tester", at: 2026-11-13T09:20:00-08:00 }
---

# `skd_fnc_pressKey` does not return what the keybind's code returned, so a check cannot assert that a key passes through to the game

Raised by arma-grappling-hook's test developer while porting that mod's M00 checks to Shakedown (`plan/milestones/m10.md`), against the install `shakedown 0.1.0` built 2026-11-13 06:18, and copied here word for word. The checks that hold it are the ported checks named in it, in arma-grappling-hook's `acceptance/shakedown/` on branch `shakedown/m10`; no acceptance check of this repository holds it yet.

## The finding as raised

3. **`skd_fnc_pressKey` does not return what the keybind's code returned, so a check cannot assert that a key passes through to the game.**
   `spec/contract/check-api.md`: `skd_fnc_pressKey` returns "`true` once the keybind's code ran". A CBA key handler that returns `false` passes the key on to the game, and the mod's CTL-01 promises that off a rope each key does (the old check `CTL-01-hold` asserted the handler's `false` off the rope and `true` on it). With `skd_fnc_pressKey` neither is readable; the port calls the handler from `CBA_fnc_getKeybind` on `p1` with `skd_fnc_on` for the off-rope half, which is the path `skd_fnc_pressKey` is meant to replace. The docs say nothing on reading whether a key was consumed. Returning the code's result (for `"down"`, and both for `"press"`) would cover it.
