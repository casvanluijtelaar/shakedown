---
type: Documentation
title: Handing over to a manual tester
description: How a check sets up a situation and hands it over to a person with skd_fnc_handOver, what the person finds on screen and in the command's output, how the session stays up for them, how the session is taken back, and what a hand over to a machine with no screen does.
status: draft
---

# Handing over to a manual tester

Some things no check can judge: how a rope looks, how a climb feels, whether a sound plays at the right moment. A check can still do the tedious part, building the situation, and then hand it over to a person who looks at it (`MAN-01`). The person never pastes into the debug console: they are put in a game client's window at the start of the situation, with a list of what to look for.

## Writing a check that hands over

Build the scene as in any check, then call `skd_fnc_handOver` with the game client and one to ten things to look for:

```sqf
// @check US-083 The owner is handed p1 in front of the fixture's crate, with three things to look for
// @setups dedicated
private _crate = ["build the scene", {
    ["p1", AGLToASL [100, 100, 0], 0] call skd_fnc_place;
    ["skdfx_Crate", AGLToASL [100, 103, 0], 180] call skd_fnc_create
}] call skd_fnc_step;

["p1", [
    "An ammo crate stands 3 m in front of you.",
    "Looking at it, the action menu (mouse wheel) offers Open crate.",
    "Opening it puts a first aid kit in your inventory."
]] call skd_fnc_handOver;
```

Write each item for a person who knows the game and not SQF: what they will see, and where to look. `skd_fnc_handOver` does not return: the body ends there, and every step still running is closed as passed. The check's outcome is then decided as for any check whose body reached its end, so a script error raised earlier still fails it.

## What the person finds

When the check passes, it is **handed over**:

- In the client's window, at the top left, a box titled `Shakedown: look for` lists the items, numbered. The list is also written to that client's RPT.
- The player stands where the check put it, and everything the check created is there: no reset follows the check.
- `shakedown run` prints the check's line with `handed over to p1`, the summary, and then:

```
Handed over to p1 by handover/US-083-crate. Look through p1's window for:
  1. An ammo crate stands 3 m in front of you.
  2. Looking at it, the action menu (mouse wheel) offers Open crate.
  3. Opening it puts a first aid kit in your inventory.
Session s-20261118-152104 stays up for you. End it with `shakedown session stop`.
```

- `run_result`, and `run --json`, carry the same as `handed_over`: the check, the machine, the items, the session's id and the command that ends it. The result file records `handed_over` on the check's execution ([contract/result-file.md](../spec/contract/result-file.md)).

The run ends with the check that handed over: the checks after it are listed as not run, with reason `handed-over`. The exit code is that of the checks that ran, so a hand over alone exits 0.

## The session stays up

The session stays up and `ready` for the person, also when the run started it itself with `--setup`, which otherwise stops its session at its end. `shakedown status` shows it. One command sets the whole situation up:

```
cd e2e
shakedown run --setup dedicated handover/US-083-crate
```

On the test machine a game client's window renders only while someone is connected through Shadow PC; the session stands either way.

In `run --each-setup`, a hand over keeps that setup's session up and starts no further setup.

## Taking the session back

The next `run`, `reset`, `reload` or `session stop` takes the session back as usual. The reset before the next check, or `shakedown reset`, removes the list from the screen with the rest of the check's state. `shakedown session stop` ends the session.

## A hand over that is refused

Only a game client has a window a person can look through. A hand over to `server` or a headless client on a dedicated session ends the check `fail` / `check-error`, naming the game clients it can hand over to, and the run goes on with its reset as usual (`US-084`):

```
fail    refuse-server  (0.6 s)
        check-error: skd_fnc_handOver: server has no screen a person can look through. Hand over to a game client: p1, p2.
```

In singleplayer `"server"` names `p1`, as everywhere, so the hand over goes to `p1`. Items that are not an array of 1 to 10 strings that are not empty end the check `fail` / `check-error` naming the argument. A check that fails or is `invalid` for any reason is not handed over, and the run goes on.
