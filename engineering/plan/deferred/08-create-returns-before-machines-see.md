---
type: Deferred
title: "skd_fnc_create returns before every machine sees the object in place"
description: "On a dedicated session skd_fnc_create returns on the server before the other machines see the object at its place, so a check that reads it on a client at once can see no object or the object at the spot it was made."
id: DEF-08
state: open
milestone: null
status: stable
---

# skd_fnc_create returns before every machine sees the object in place

## Summary

`skd_fnc_create` makes the object on the server and returns it there. On a dedicated session the other machines learn of the object, and then of its direction and position, over the network a moment later. The owner decided that `skd_fnc_create` waits until every machine sees the object in place before it returns, as `skd_fnc_place`, `skd_fnc_setDate` and `skd_fnc_setWeather` wait for what they set. No milestone is chosen for it yet.

## Current behaviour

`skd_fnc_create` returns as soon as the server has made the object, set its direction and then its position. A check that reads the object on a client or a headless client at once can find no object there, or the object at the spot the engine first made it. `docs/check-api.md` and `docs/dedicated-sessions.md` say that a check reading a just-created object on another machine waits for it first, and show the wait.

## The gap

Every check that creates an object and reads it on another machine needs a wait of its own, and a check that leaves it out passes or fails by timing, which reads as a flaky mod.

## Options

1. `skd_fnc_create` polls every machine of the session after it creates the object, as `skd_fnc_place` polls the server, and returns once each one sees the object within 0.1 m and 3 degrees of where it was put; at a ceiling the check ends `fail` / `wait-ceiling` naming the machine that did not see it.
2. A separate function a check calls when it needs the object everywhere, leaving `skd_fnc_create` as it is.
3. Leave it as the documented wait in the check.

## Recommendation

The first option, the owner's decision. In singleplayer it costs nothing, since the only machine is the server. On a dedicated session the wait it adds to each creation is measured when it is built, with two clients and a headless client, and written into the check API contract.

## Acceptance criteria

1. On a dedicated session with two clients and a headless client, an object made with `skd_fnc_create` is read at its place by every machine right after the call returns, 20 times out of 20.
2. A machine that does not see the object in place within the ceiling ends the check `fail` / `wait-ceiling` naming that machine.
3. The added wait per creation is measured and written into `SCN-01` and the check API contract, and `docs/` drop the check's own wait from their examples.
