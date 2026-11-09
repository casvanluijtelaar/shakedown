---
type: Deferred
title: "An enemy a check made can still hit a player while the reset puts it back"
description: "The reset deletes the entities a check made and restores the players in the same phase, so a shot an enemy fired before its machine deleted it can wound or kill a player that is already back at its baseline; the direction is set again, but the damage is not verified and a death after the 1.5 s settle window fails the reset, and a death during the reset was measured to take it to 4.2 and 4.8 s."
id: DEF-22
state: open
milestone: null
status: stable
---

# An enemy a check made can still hit a player while the reset puts it back

## Summary

Finding `m06-1` was a reset that failed on p2's direction. Its cause: an armed enemy that a check made on the headless client shot p2 while the reset ran, the shot killed p2 right after its machine had put it back, and the respawn put the new unit on the respawn marker, which is p2's baseline position, facing a direction of the respawn's choosing. The verification of `P-06` step 4 sets the player's direction again while it is more than 1 degree off, for up to 1.5 s, so that case resets in under 2 s. Two neighbouring cases remain.

## Current behaviour

`P-06` deletes what each machine's recorder saw in step 3, on every machine at once, in the same phase in which each machine restores its own player. An enemy unit local to the headless client fires until the headless client deletes it, and a bullet already fired flies on. A probe of 40 checks like `US-008`'s, with an enemy 30 m from p2, saw p2 killed during the check and once at the moment the reset put it back.

## The gap

- A shot that wounds but does not kill a player after step 3 leaves it with damage above 0; step 4 does not verify damage, so the next check starts with a hurt player, against `RST-01`.
- A death after the 1.5 s of step 4 leaves the respawned unit on its baseline position but facing elsewhere, or a death with a respawn point away from the baseline leaves it away, and the reset fails at its ceiling.
- A death during the reset makes it slower than `NFR-PF-04`'s 2 s even when it verifies. On the build installed in the main checkout at `22199e4`, before the direction rule of step 4, the probe ran 48 checks like `US-008`'s, each followed by `shakedown reset`, in one dedicated session (`s-20261109-220129`, two clients, a headless client, CBA and `@skd_fixture`). 46 resets were done in 0.82 to 1.14 s, median 0.89 s; two were done in 4.238 s and 4.782 s. In both, p2's own log shows it in the `unconscious` animation as its machine put it back and then a new unit on the respawn marker, so p2 was killed by the enemy's fire during the reset. Where the other 3 s go is not measured: the probe logged no phase times, so whether the respawn, the `free` phase's wait for a living player or the settle window takes them is not known. The probe has not been run on the build with the direction rule, `749d8d6`.

## Options

1. **Delete before restoring.** Split step 3: every machine deletes what it recorded, the runner waits for every machine, then every machine restores its player. Shots fired before the deletion still fly for a moment.
2. **Verify the damage and a death in step 4.** Set damage 0 again while it is above 0 and handle a respawn during step 4 the way step 2 does, extending the settle window from the respawn.
3. **Both.**

## Recommendation

Option 3 when a check of a mod's own shows a hurt player after a reset; until then the direction rule of step 4 covers what was seen.

## Acceptance criteria

- A check that leaves an armed enemy firing at p2, run 20 times with a reset after each on a dedicated session with two clients and a headless client, sees p2 alive, unhurt and at its baseline position and direction at the start of every next check, and every reset is done under 2 s.
- The time of each reset phase is known for a reset during which a player dies, and such a reset is done under 2 s, or `NFR-PF-04` names the case it does not cover.
