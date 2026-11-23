---
type: Documentation
title: Scenes, resets and cleanups
description: How a check builds its scene with objects, units, the player's place, the date and the weather, what the reset between checks restores and what it leaves, ACE's full heal and dead players, cleanups for state the reset cannot name, a check that starts in a fresh mission with @reset mission, resetting by hand, and what a failed reset does.
status: draft
---

# Scenes, resets and cleanups

Every check starts from the state the session's mission started in, builds the situation it needs, and leaves the clearing up to the reset that runs before the next check. This page covers both halves. The reference is [The check API](check-api.md#building-a-scene).

## The session mission

A session runs a mission Shakedown generates: an empty mission on the setup's `world` (`VR` by default), with `p1` at the setup's `spawn` (`[100, 100]` by default) and each further client `pN` 10 m further east per N. A check builds everything else itself. Positions in the check API are ASL: convert a position on the ground with `AGLToASL`, as `AGLToASL [100, 105, 0]`.

## Building a scene

```sqf
private _scene = ["build the scene", {
    private _wall = ["Land_Wall_IndCnc_4_F", AGLToASL [100, 110, 0], 180] call skd_fnc_create;
    private _enemy = ["O_Soldier_F", AGLToASL [100, 140, 0], 180] call skd_fnc_create;
    ["p1", AGLToASL [100, 104, 0], 0] call skd_fnc_place;
    [[2035, 6, 24, 22, 0]] call skd_fnc_setDate;
    [0.9, 0.2, 0.6] call skd_fnc_setWeather;
    [_wall, _enemy]
}] call skd_fnc_step;
_scene params ["_wall", "_enemy"];
```

### Objects, vehicles and units: `skd_fnc_create`

`[class, position, direction] call skd_fnc_create` makes a `CfgVehicles` class on the server, sets its direction and then its position, and returns it once the server has made it. A class that is a kind of `CAManBase` becomes a unit in a new group of the side its config gives, so `O_Soldier_F` is an OPFOR soldier. The vehicle is made as the engine's `createVehicle` makes it, so a class that randomises its look when it is created can look different between runs.

An unknown class, or one that cannot be created, ends the check `fail` / `check-error`:

```
skd_fnc_create: the class "B_Soldier_Typo" is not in CfgVehicles (or is not creatable). Check its spelling and that its mod is in the setup.
```

On a dedicated session another machine sees the new object a moment later. A check that reads it there at once can find no object, or the object at the spot the engine made it before it moved it into place. Wait for it on that machine first ([Code, waits and agreement across machines](dedicated-sessions.md#code-waits-and-agreement-across-machines)):

```sqf
private _crate = ["Land_WoodenBox_F", AGLToASL [100, 102, 0], 0] call skd_fnc_create;
["p2", {
    private _c = _this select 0;
    !isNull _c && { (getPosASL _c) distance2D [100, 102, 0] < 0.1 }
}, "p2 sees the crate in place", 5, [_crate]] call skd_fnc_waitUntil;
```

### The player's place: `skd_fnc_place`

`[unit, position, direction, ceiling] call skd_fnc_place` takes an object or a machine name (that machine's player), sets the direction and then the position on the machine where the unit is local, and returns once the server sees the unit within 0.1 m and 3 degrees of it. At its ceiling it ends the check `fail` / `wait-ceiling` with the position the server last saw.

Every game client starts near the same spot, so a player placed there may stand inside another. Place each player you use away from the others.

### Date and weather: `skd_fnc_setDate`, `skd_fnc_setWeather`

`[[year, month, day, hour, minute], ceiling] call skd_fnc_setDate` and `[overcast, fog, rain, ceiling] call skd_fnc_setWeather` (each 0 to 1) set the date or the weather on the server, force the change, and set the same on every other machine. They return once every machine reports the date to the minute, or the overcast within 0.01; at the ceiling they end the check `fail` / `wait-ceiling` with each machine's values. Note the extra brackets around the date: it is one argument.

## The reset between checks

Before every check the session is reset in place, in under a second. On the machines it runs on all at once:

1. **The previous check's cleanups run**, each on the machine it names, last registered first ([Cleanups](#cleanups)).
2. **Every player is freed**: out of any vehicle, and alive again after a death.
3. **Everything created since the mission started is deleted**: every unit, vehicle and object, whatever created it, through the check API or the check's own SQF, on whichever machine. Each player's loadout, damage, position, direction and animation are put back as they were at the mission's start, and the date and weather are set back on every machine.
4. **The result is checked**: each machine checks its own state, and the server also every player's position as it sees it.

On a dedicated session a reset after a check that changed everything takes about 0.75 s, and one with nothing to undo about 0.45 s.

- **ACE medical.** When ACE medical is loaded, each player is also given ACE's full heal (`ace_medical_treatment_fnc_fullHealLocal`, on the machine the player is local to), which clears the wounds, blood loss, pain, fractures, tourniquets and medication that `setDamage` leaves. Shakedown is checked against ACE 3.21.2. ACE marks that function as not public, so an ACE version without it fails the reset, `invalid` / `reset-failed`, with a message that names the ACE version the machine loaded.
- **A player who died.** In singleplayer a player who dies during a check is handed a new unit where it fell at once, so the mission goes on; on a dedicated session it respawns at once. The reset brings that unit back to the start and deletes the body, and the client keeps its start number.
- **Clients that joined or left.** On a dedicated session the reset first brings the session back to the machines it started with: a late client that joined leaves, and a client that left joins again ([Joining late, leaving and joining again](dedicated-sessions.md#joining-late-leaving-and-joining-again)).
- **What the check left stays until then.** After a run against a session you keep up, the last check's scene is still there, so you can look at it in the game window or with `shakedown inspect`.

### What the reset does not touch

- variables set on units, on objects that existed at the start, or in `missionNamespace`;
- actions added to a player with `addAction`;
- event handlers a check added;
- a mod's own state and its settings.

A check that changes those registers a cleanup. A check that depends on them being fresh starts in a restarted mission instead.

## Cleanups

```sqf
private _id = ["p1", { player addAction ["Wave", {}] }] call skd_fnc_on;
[{ player removeAction (_this select 0) }, [_id], "p1"] call skd_fnc_cleanup;

["p1", { player allowDamage false }] call skd_fnc_on;
[{ player allowDamage true }, [], "p1"] call skd_fnc_cleanup;

["server", { mymod_setting_ropeLength = 30 }] call skd_fnc_on;
[{ mymod_setting_ropeLength = 15 }] call skd_fnc_cleanup;    // on the lead: the server, or p1 in singleplayer
```

`[code, args, machine] call skd_fnc_cleanup` registers code that runs on `machine` at the start of the next reset, each machine's last registered first. It runs whatever the check's outcome, so a check that fails half way leaves no such state to the next.

- `machine` defaults to the lead machine: `p1` in singleplayer, the server on a dedicated session. State that is local to one client, such as an action on its player, `allowDamage` on its unit, a CBA event handler or a variable in its `missionNamespace`, is undone on that client, so its cleanup names it. On a dedicated server `player` is `objNull`.
- Like code given to `skd_fnc_on`, a cleanup sees only its `args` as `_this`, calls no `skd_fnc_*` function, and looks up objects in `args` once, when it starts.
- Keep cleanups short. A cleanup still running at the reset's ceiling of 15 s is terminated and fails the reset, naming the check that registered it. Its time also counts against the run's ceiling ([The runner ceiling](unattended-runs.md#the-runner-ceiling)).
- A check needs no cleanup for what it created: the reset deletes it.

## A check that needs a fresh mission

A check that depends on state the reset cannot name, such as a public variable an earlier check set or a mod's setting, asks for a mission restart instead with `// @reset mission` in its header:

```sqf
// @check US-011 A fresh mission clears what an earlier check set
// @reset mission
["every machine starts with no such variable", {
    ["all", { isNil "my_flag" }, "==", true, "my_flag is nil"] call skd_fnc_expectEach;
}] call skd_fnc_step;
```

- **Singleplayer**: `p1` ends the mission and the game loads the session mission again, in about 2 s. `p1` keeps its process and its start number.
- **Dedicated**: the game clients and the headless clients are stopped, the server kicks any user it still lists and restarts the mission, and fresh client and headless client processes join it, each with its start number one higher. The server keeps its process. With two clients and a headless client this takes about 48 s, most of it the clients' own start: about as long as starting the session.

Every machine captures a new baseline at the new mission start, and the library folder is compiled again. The result file lists every machine start that took part in the run, so a run with a `@reset mission` check shows the old and the new start of each client. A mission restart does not reload a changed mod file: the processes it keeps still hold the files they read; `shakedown reload` does ([Reloading changed mod files](reloading.md)).

Start without `@reset mission`, and add it only to a check that passes alone and fails after another. When the mission restart does not bring a machine back, that check ends `invalid` / `reset-failed`, naming the machine, and the session is broken: the checks after it end `invalid` / `session-broken`, and the run exits 2. `shakedown reload` brings it back.

## Resetting by hand

```
> shakedown reset
reset done in 0.579 s
```

`shakedown reset` (the MCP tool `reset`) runs the same reset outside a run. It is refused, exit code 4, while a run, a reload or another reset is in progress, naming it, and on a broken session, naming the ways out:

```
Session s-20261111-194841 is busy with reload rl-20261111-195102-7 (level game-restart), so it cannot be reset now. Wait for it to be ready; `shakedown status` shows it.
```

## When the reset fails

A reset that cannot bring the session back within 15 s fails, naming what did not match, each item with its machine, or the check whose cleanup did not end, or the ACE version without the full heal. Before a check, that check ends `invalid` / `reset-failed`; by hand, `shakedown reset` prints `reset failed in <s> s` with each mismatch and exits 4. Either way the session is broken: every check after it is `invalid` / `session-broken` until `shakedown reload` restarts the game or the session is stopped.

Common causes, and what to do:

- **A cleanup that runs long or never ends.** Keep cleanups short.
- **A unit the check made still acting during the reset**, such as an enemy whose shot, already in flight, hits a player who is back at the start. Delete or disarm such units at the end of the check.
- **An ACE version without the full heal.** The message names the version; use ACE 3.21.2 or pin the version you checked against.
