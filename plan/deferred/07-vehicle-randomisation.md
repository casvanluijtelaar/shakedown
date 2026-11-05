---
type: Deferred
title: "skd_fnc_create does not switch a vehicle's randomisation off"
description: "SCN-01 promises vehicles created with vehicle randomisation off, but skd_fnc_create makes them with createVehicle as the engine does, so a class that randomises its look on init can differ between runs."
id: DEF-07
state: open
milestone: null
status: stable
---

# skd_fnc_create does not switch a vehicle's randomisation off

## Summary

`SCN-01` and the check API contract say `skd_fnc_create` runs "with vehicle randomisation off". Some vanilla and mod vehicles pick textures and animation sources at random in their `init` event handler (`BIS_fnc_initVehicle`), and a vehicle's look can matter to a check that measures its geometry or hit points.

## Current behaviour

`skd_fnc_create` makes a vehicle with `createVehicle` at `[0, 0, 0]`, sets its direction, then its position. The `init` handler runs inside `createVehicle`, before the framework can set the vehicle's `BIS_enableRandomization` variable to `false`. Three `C_Offroad_01_F` made this way had the same default textures (research `23-spikes-singleplayer-reset.md`), so no randomisation showed, but nothing stops it.

## The gap

A check that depends on a vehicle's look or its animation sources can see a different vehicle in a different run, which reads as a flaky mod.

## Options

1. After `createVehicle`, call `BIS_fnc_initVehicle` again with the class's default variant and no animation randomisation, which puts back what the `init` handler randomised.
2. Make the vehicle with `create3DENEntity`-style attributes or through `createVehicle` on a class whose randomisation is read from a mission variable, if the engine offers one; needs a look at `BIS_fnc_initVehicle` on the current game version.
3. Leave it and say in `docs/` that a check depending on a vehicle's look sets it itself.

## Recommendation

The first option, measured with a class that randomises visibly (`C_Offroad_01_F` over twenty creations), when a story needs a vehicle's look; until then the third.

## Acceptance criteria

1. Twenty `C_Offroad_01_F` made by `skd_fnc_create` in one session have the same textures and animation phases.
2. The check API contract and `SCN-01` say how randomisation is switched off.
