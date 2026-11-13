---
type: Finding
id: m10-5
milestone: M10
title: "The docs do not say how an object in `args` is looked up on a machine that has not got it in place yet; inside `skd_fnc_on` a just-created object never came right within 10 s"
description: "skd_fnc_on sent at once after skd_fnc_create on a dedicated session gave code on p1 an object that never came right within 10 s, and the docs do not say which functions look an object in args up once and which at each evaluation."
status: draft
raised_by: test
for: developer
checks:
  - "US-013 DM-14 m10-5 on a dedicated session each of 20 crates waited for on every client right after skd_fnc_create is found there in place within the 10 s ceiling, and skd_fnc_on then reads it there, not objNull"
stories: [US-013]
generated: { by: "claude-code/tester", at: 2026-11-13T07:10:46-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-13T08:14:00-08:00, note: "Fixed and documented: it was objNull. skd_fnc_on looks an object in args up once, when its code starts, so on a client that does not know the tower yet the code held objNull for the whole 10 s, and getPosASL of objNull is [0,0,0], the map origin. A wait and a polled assertion now look each object up again before every evaluation, so the documented skd_fnc_waitUntil finds the object as soon as it arrives, where it looked it up once too and passed by timing (DM-14, spec/contract/check-api.md, docs/dedicated-sessions.md with a table of which function looks an object up when, docs/check-api.md). The game test cfg_01_rst_01_dm_14_m10_findings_on_a_dedicated_server proves it: 20 crates, each waited for on every machine at once after skd_fnc_create, then read with skd_fnc_on." }
verified:
  - { by: "claude-code/tester", at: 2026-11-13T09:20:00-08:00 }
---

# The docs do not say how an object in `args` is looked up on a machine that has not got it in place yet; inside `skd_fnc_on` a just-created object never came right within 10 s

Raised by arma-grappling-hook's test developer while porting that mod's M00 checks to Shakedown (`plan/milestones/m10.md`), against the install `shakedown 0.1.0` built 2026-11-13 06:18, and copied here word for word. The checks that hold it are the ported checks named in it, in arma-grappling-hook's `acceptance/shakedown/` on branch `shakedown/m10`; no acceptance check of this repository holds it yet.

## The finding as raised

5. **The docs do not say how an object in `args` is looked up on a machine that has not got it in place yet; inside `skd_fnc_on` a just-created object never came right within 10 s.**
   `spec/contract/check-api.md`: "Objects and groups in `args` and in results travel by net id and arrive as the object or group (`DM-13`, `DM-14`)." `docs/check-api.md`, "Building a scene": `skd_fnc_create` "returns once the server has made the object. On a dedicated session another machine sees it a moment later: a check that reads it there at once can find no object, or the object at the spot the engine made it, before it is in place. Such a check waits for it first, as dedicated sessions shows." Only `skd_fnc_useAction` says its target "is looked up every frame".
   The port's first version, on `dedicated-cba`, created the tower with `skd_fnc_create` at `[1300, 500]` and at once sent `["p1", { params ["_tower"]; [{ !isNull _tower && { ((getPosASL _tower) distance2D [1300, 500]) < 0.1 } }, 10] call acc_fnc_poll; ... getPosASL _tower ... }, [_tower], 15] call skd_fnc_on` (`acc_fnc_poll` polls each frame with a ceiling). The poll ran its 10 s out and the code then found the tower's top near the map origin: evidence `lip` `[[2.07538, 0.588126, 23.32], 18.3]` in run `r-20261113-144546-8`, check `MP-01-drawn`, the first check of the session, which then failed `noEdge`. So for 10 s the `_tower` that code held was either `objNull` or the object at the spot the engine made it. Waiting first with `skd_fnc_waitUntil` and the object in its `args`, as `docs/dedicated-sessions.md` shows, and only then calling `skd_fnc_on`, worked in every dedicated run since (3 of 3). Which of the two it was, and whether `skd_fnc_on` resolves an object once on arrival while `skd_fnc_waitUntil` resolves it at each evaluation, the docs do not say. Saying which functions look an object up once and which at each evaluation, and what a machine that does not know the object yet receives, would have saved the failed run. A 10 s wait for an object to reach a client also seems long; if the object was there at the origin all that time, that is worth a look too.
