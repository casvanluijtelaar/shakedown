---
type: Finding
id: m10-2
milestone: M10
title: "A cleanup runs on the lead machine only, so on a dedicated session it cannot undo what a check did on a client; the docs tell a check to use one for exactly that"
description: "A cleanup runs on the lead machine only, which is the server on a dedicated session, so it cannot undo an action, allowDamage, an event handler or a variable a check set on a client, though the docs tell a check to use one for exactly that."
status: draft
raised_by: test
for: developer
checks: []
stories: [US-008]
generated: { by: "claude-code/tester", at: 2026-11-13T07:10:46-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-13T08:14:00-08:00, note: "Fixed: skd_fnc_cleanup takes a third argument, the machine it runs on (default the lead), and the reset runs each machine's cleanups on that machine, so a check undoes a client's allowDamage, action or variable even when it fails first (spec/contract/check-api.md, P-06 step 1, docs/check-api.md, docs/dedicated-sessions.md, docs/porting-a-suite.md). The game tests cfg_01_rst_01_dm_14_m10_findings_in_singleplayer and _on_a_dedicated_server prove it: a check that sets player allowDamage false on p1, names a cleanup for p1 and fails is followed by one that finds p1 damageable." }
verified: []
---

# A cleanup runs on the lead machine only, so on a dedicated session it cannot undo what a check did on a client; the docs tell a check to use one for exactly that

Raised by arma-grappling-hook's test developer while porting that mod's M00 checks to Shakedown (`plan/milestones/m10.md`), against the install `shakedown 0.1.0` built 2026-11-13 06:18, and copied here word for word. The checks that hold it are the ported checks named in it, in arma-grappling-hook's `acceptance/shakedown/` on branch `shakedown/m10`; no acceptance check of this repository holds it yet.

## The finding as raised

2. **A cleanup runs on the lead machine only, so on a dedicated session it cannot undo what a check did on a client; the docs tell a check to use one for exactly that.**
   `docs/check-api.md`, "Cleanups and ending a check", shows `private _id = player addAction ["Wave", {}]; [{ player removeAction (_this select 0) }, [_id]] call skd_fnc_cleanup;` and says a cleanup is for "state the reset cannot name: an action added to the player, a variable, a mod's setting". `docs/porting-a-suite.md`, "Points to watch": "State the reset does not touch ... variables on units or in `missionNamespace`, actions added to the player ... A check that changes them registers a `skd_fnc_cleanup`." `spec/contract/check-api.md`: a cleanup is code "to run on the lead machine".
   On a dedicated session the lead machine is the server, where `player` is `objNull`; a client's `addAction`, `allowDamage`, a CBA event handler or a variable in the client's `missionNamespace` are local to that client, and a cleanup has no `skd_fnc_on` to reach it (no `skd_fnc_*` call is allowed outside the body). The port's dedicated checks set `player allowDamage false` on `p1` and could not register a cleanup for it: the body puts it back at its end, so a check that fails earlier leaves `p1` unhurtable for the next check. The docs should either say that a cleanup reaches only the lead machine and what to do for a client's state, or let a cleanup name a machine.
