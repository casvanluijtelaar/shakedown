---
type: Finding
id: m06-1
milestone: M6
title: A reset on a dedicated session failed once on p2's direction and broke the session
description: In the first full suite run of M6, shakedown reset after the US-008 check that moves p2 to 200 degrees took 15.4 s and failed, finding p2 facing 337.96 degrees against a baseline of 0, and the dedicated session was broken for the 34 checks after it; the same reset passed in the 9 tries after it.
raised_by: test
for: developer
checks: ["US-008 RST-03 NFR-PF-04 M2 exit 3: shakedown reset after a check that changed the scene is done in under 2 s on a dedicated session"]
stories: [US-008]
generated: { by: "claude-code/tester", at: 2026-11-09T13:35:09-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-09T14:23:53-08:00, note: "p2 was killed the moment the reset put it back, by a shot from the enemy the check makes on hc1, and respawned on the respawn marker, which is its baseline position, facing a direction of the respawn's choosing; P-06 step 4 set the player again only when its position was off. It now also sets it again while its direction is more than 1 degree off (P-06 edited; DEF-22 holds the damage and late-death cases). Proved by m02 us_008_rst_01_rst_05_a_player_killed_as_the_reset_puts_it_back_faces_its_baseline, which failed with this finding's message before the fix, and the named check passed 3 of 3 with --repeat 3." }
verified: []
---

# A reset on a dedicated session failed once on p2's direction and broke the session

## What the spec promises

`RST-01`: "restores every player's unit (alive, position, direction, velocity, damage, loadout, animation state, out of any vehicle) ... The next check sees the baseline state on every machine." `US-008`, criterion 3: "`shakedown reset` on a dedicated session with two clients and one headless client, after such a check, reports state `done` with a duration under 2 s." `NFR-PF-04`: "A reset in place takes under 2 s on a dedicated session with two clients and one headless client."

## What the system does

In the full suite run of 2026-11-09 from 20:41Z, on the `ded` setup (a server, two clients, a headless client, CBA, both fixtures and arma-grappling-hook), against the framework installed from `679c79e`: after the 21 `US-008` start-state checks, the suite ran three times `shakedown run us008/US-008-start-state-02 --json` followed by `shakedown reset --json`. The check moves `p1` into an MRAP and places `p2` at `[140, 180, 0]` facing 200 degrees, with damage 0.3 and no items. Each run passed. The first two resets were `done` in 0.8 s. The third:

```
shakedown reset --json        exit 4 in 15.4 s, nothing on stderr
runner.log: s-20261109-205157 reset failed: server: it sees p2 facing 337.957, the baseline 0; p2: the player faces 337.958, the baseline 0
```

The session was then `broken` ("the reset failed: server: it sees p2 facing 337.957, the baseline 0; p2: the player faces 337.958, the baseline 0"), and every later run of the `ded` session was refused, exit 4. 337.96 is neither the direction the check gave p2 (200) nor the baseline (0).

The same reset passed in 9 of 9 tries since: `--check "US-008 RST-03" --repeat 3` (3 repeats of three resets, one of them after a check that killed `p2`), and the next full run's three resets, all `done` in 0.8 to 0.9 s. The check and the template are unchanged since M2, whose rounds and every full run up to M5 saw no such failure.

## What it costs

A test author who walks away from a full suite (`US-033`) comes back to one invalid session: every check after the failed reset is refused or `invalid` / `session-broken`, here 34 checks of six milestones, and the suite says nothing about them. The reset is the step every in-place check depends on (`US-008`), so a reset that fails now and then on a direction it should have set costs the whole rest of a run, not one check.

## The check that holds it

`US-008 RST-03 NFR-PF-04 M2 exit 3: shakedown reset after a check that changed the scene is done in under 2 s on a dedicated session`. It failed once in 10 tries; run it with `--repeat 3` several times, or the full suite, to see it.
