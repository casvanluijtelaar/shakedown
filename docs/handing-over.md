---
type: Documentation
title: Handing over to a person
description: How a check sets up a situation and hands it over to a person with skd_fnc_handOver, how to write the things to look for, what the person finds on screen and in the command's output, how the session stays up for them and is taken back, and what a hand over to a machine with no screen does.
status: draft
---

# Handing over to a person

Some things no check can judge: how an effect looks, how a vehicle handles, whether a sound plays at the right moment. A check can still do the tedious part, building the situation, and then hand it over to a person who looks at it. The person never pastes into the debug console: they are put in a game client's window at the start of the situation, with a list of what to look for.

## Writing a check that hands over

Build the scene as in any check, then call `[machine, items] call skd_fnc_handOver` with the game client and one to ten things to look for:

```sqf
// @check US-083 A tester is handed p1 in front of the mod's crate, with three things to look for
// @setups dedicated
private _crate = ["build the scene", {
    ["p1", AGLToASL [130, 130, 0], 0] call skd_fnc_place;
    ["p1", { player removeItems "FirstAidKit"; { _x == "FirstAidKit" } count items player }] call skd_fnc_on;
    ["mymod_Crate", AGLToASL [130, 132, 0], 180] call skd_fnc_create
}] call skd_fnc_step;

["p1 can open the crate", {
    ["p1", {
        params ["_crate"];
        !isNull _crate
            && { (actionIDs _crate) findIf { ((_crate actionParams _x) select 0) isEqualTo "Open crate" } >= 0 }
            && { (player distance _crate) < 3 }
    }, "the crate reached p1 with its action Open crate in reach", 10, [_this select 0]] call skd_fnc_waitUntil;
}, [_crate]] call skd_fnc_step;

["p1", [
    "An ammo crate stands 2 m in front of you.",
    "Looking at it, the action menu (mouse wheel) offers Open crate.",
    "You carry no first aid kit; opening the crate puts one in your inventory."
]] call skd_fnc_handOver;
```

Write each item for a person who knows the game and not SQF: what they will see, and where to look. Make each item true before the hand over, and check what can be checked: the person judges how it looks and feels, not whether the scene was built. Three things in the scene above serve that:

- An action shows in the action menu only while the player is within its radius, measured from the player to the object's centre, so `p1` stands 2 m from a crate whose action has a radius of 3 m, not at the edge of it. The step before the hand over waits on `p1` until the action is there and in reach.
- Every game client starts near the same spot, so a player placed there stands inside another. Place the player away from it.
- What the person is to notice must change on screen: `p1`'s loadout carries a first aid kit, so the scene takes it out and the one from the crate is the only one. The reset puts the loadout back later.

`skd_fnc_handOver` does not return: the body ends there, and every step still running is closed as `passed`. The check's outcome is then decided as for any check whose body reached its end, so a script error raised earlier still fails it.

## What the person finds

When the check passes, it is **handed over**:

- In the client's window, at the top centre, a box titled `Shakedown: look for` lists the items, numbered in the order given. The list is also written to that client's RPT.
- The player stands where the check put it, and everything the check created is there: no reset follows the check.
- `shakedown run` prints the check's line with `handed over to p1`, the summary, and then:

```
pass    handover/US-083-crate  (14.2 s)
        handed over to p1
1 passed, 0 failed, 0 invalid, 0 not run. Result: C:\...\.shakedown\runs\r-20261118-152230-1\result.json
Handed over to p1 by handover/US-083-crate. Look through p1's window for:
  1. An ammo crate stands 2 m in front of you.
  2. Looking at it, the action menu (mouse wheel) offers Open crate.
  3. You carry no first aid kit; opening the crate puts one in your inventory.
Session s-20261118-152104 stays up for you. End it with `shakedown session stop`.
```

- `run --json` and the MCP tool `run_result` carry the same as `handed_over`: the check, the machine, the items, the session's id and the command that ends it. The result file records `handed_over`, with the `machine` and the `items`, on the check's execution ([Results and the result file](results.md)).

The run ends with the check that handed over: the checks after it are listed as not run, with reason `handed-over`. The exit code is that of the checks that ran, so a hand over alone exits 0.

### Where the list stands

The list stands in the column Arma 3's default HUD layout leaves free, between the vehicle info at the top left and the weapon info at the top right, so it covers neither the action menu and the communication menu on the left nor the weapon info, stance, stamina bar and hints on the right, at any aspect ratio or interface size. It starts half a GUI grid unit below the top of the safe zone, spans the space between 11.5 units from its left edge and 15.3 units from its right edge, at most 24 units wide and centred in it, and grows downwards with its items. A HUD layout changed in the game's options is not followed.

## The session stays up

The session stays up and `ready` for the person, also when the run started it itself with `--setup`, which otherwise stops its session at its end. `shakedown status` shows it. So one command sets the whole situation up:

```
shakedown run --setup dedicated handover/US-083-crate
```

A game client's window renders only while the machine has a display, such as a connected remote desktop; the session stands either way. In `run --each-setup`, a hand over keeps that setup's session up and starts no further setup.

## Taking the session back

The next `run`, `reset`, `reload` or `session stop` takes the session back as usual. The reset before the next check, or `shakedown reset`, removes the list from the screen with the rest of the check's state. `shakedown session stop` ends the session.

## A hand over that is refused

Only a game client has a window a person can look through. A hand over to `server` or a headless client on a dedicated session ends the check `fail` / `check-error`, naming the game clients it can hand over to, and the run goes on with its reset as usual:

```
fail    refuse-server  (0.6 s)
        check-error: skd_fnc_handOver: server has no screen a person can look through. Hand over to a game client: p1, p2.
```

In singleplayer `"server"` names `p1`, as everywhere, so the hand over goes to `p1`. Items that are not an array of 1 to 10 strings that are not empty end the check `fail` / `check-error` naming the argument. A list that is not on the client's screen within 10 s ends it `fail` / `wait-ceiling`. A check that fails or is `invalid` for any reason is not handed over, and the run goes on.
