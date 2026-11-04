---
type: User Story
title: "Build a scene: objects, units, a player's position and facing, time and weather"
description: "A test author places objects, units and vehicles, moves a player to an exact spot facing an exact way and sets the date and weather, and every machine sees the same scene, from one check that runs in singleplayer and on a dedicated server."
status: draft
id: US-017
role: test-author
kind: feature
requirements: [SCN-01, SCN-02, SCN-03, API-03]
---

# US-017 Build a scene: objects, units, a player's position and facing, time and weather

As a test author, I want to set up a situation, such as a squad on a slope at dusk, from the check itself, so that I never paste into the debug console to get there.

## Acceptance criteria

1. After `["p1", _pos, 135] call skd_fnc_place`, `["all", {[getPosASL (_this select 0), getDir (_this select 0)]}, ...]` read on every machine puts the player within 0.1 m of `_pos` facing 135 within 3 degrees.
2. `["B_Soldier_F", _pos, 90] call skd_fnc_create` returns a unit in a group of side WEST, and `[[2035, 6, 1, 19, 30]] call skd_fnc_setDate` and `[0.8, 0.2, 0] call skd_fnc_setWeather` return `true`, after which every machine reports that date to the minute and that overcast within 0.01.
3. The same check file, with no change, passes in a singleplayer session and in a dedicated session with two clients.

## Next to

- [US-018](US-018-unknown-class-or-argument.md) (mistake): an unknown class or a wrong argument.
