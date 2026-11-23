---
type: Research
title: "Spikes SPK-04, SPK-05, SPK-10 and SPK-12 in singleplayer"
description: "What the reset in place, the player's death and endMission under -autotest, an action fired with action User, and net ids measured in a singleplayer session on the test machine in M1, with what CBA's keybind registry and a created vehicle showed on the way."
status: stable
generated: { by: "claude-code/developer", at: 2026-11-05T19:45:00+01:00 }
sources:
  - { id: spikes, resource: "crates/shakedown/tests/spikes_m01.rs, spk_12_spk_10_spk_05_singleplayer_p1, run of 2026-11-05 on the Windows test machine (session s-20261105-185201), Arma 3 2.22.154089 stable", read: 2026-11-05 }
  - { id: probe, resource: "crates/shakedown/tests/spikes_m01.rs, int_cba_01_keybind_registry_probe, run of 2026-11-05 with CBA_A3 3.19.0.260808 and arma-grappling-hook 0.1.0 (session s-20261105-185602)", read: 2026-11-05 }
  - { id: twenty, resource: "crates/shakedown/tests/m01.rs, nfr_rl_02_spk_04_twenty_checks_each_see_the_start_state, and the reset lines of %LOCALAPPDATA%\\Shakedown\\runner.log, runs of 2026-11-05", read: 2026-11-05 }
  - { id: demo, resource: "e2e/grappling-hook runs of 2026-11-05 against one session, with the reset failures of sessions s-20261105-191502 and s-20261105-191625 in runner.log", read: 2026-11-05 }
---

# Spikes SPK-04, SPK-05, SPK-10 and SPK-12 in singleplayer

M1 ran `SPK-12`, `SPK-10` and `SPK-05` whole and the singleplayer half of `SPK-04` (`spec/06-open-questions.md`) on the Windows test machine, Arma 3 2.22.154089 stable, in a session the runner started with `-autotest`. The game ran in a window with no person connected through Shadow PC. Each runs again with `cargo test -p shakedown --test spikes_m01 -- --ignored --nocapture` and, for `SPK-04`, `cargo test -p shakedown --test m01 nfr_rl_02 -- --ignored --nocapture`, after `python tools/install.py`.

## SPK-12: passed

In singleplayer `netId` gives every object an id of the form `0:<n>` and `objectFromNetId (netId _x)` returns the object, for two created boxes (`0:6`, `0:7`), a created unit (`0:10`) and `player` (`0:5`); `groupFromNetId (netId _g)` returns a created group (`0:290`); `netId objNull` is `""` and `objectFromNetId ""` is null[^spikes].

**Forces:** `DM-13` and `DM-14` stand as written in singleplayer. Code a check runs on the lead machine itself takes its arguments and gives its result through the same encoding and decoding, so a check behaves alike on the lead and on another machine (`P-03` step 5).

## SPK-10: failed, as the spec allowed for

`player action ["User", target, id]` ran the action's script every time, whatever its condition and radius: once with the target 10 m away and a radius of 3 m, once 2 m away with its condition false, once with it true; and on the player's own unit once with its condition false and once true[^spikes].

**Forces:** `ACT-01`'s own wait for the action's title, condition and radius is the only check; the engine adds none. No promise changes. The condition is evaluated with `_target` and `_this` as the engine documents them, in an unscheduled call, every frame of the wait.

## SPK-05: the first fallback holds; endMission passed

| What | Result |
|---|---|
| `p1` killed with `setDamage 1`, `respawn = 3`, no handler | The engine ended the mission and `-autotest` loaded the next copy of it: the mission start count went from 1 to 2, the order sent before the death was never answered, and the player was alive again only as the new mission's unit, everything the check built gone. The pass condition fails at its first death[^spikes]. |
| A `Killed` handler that creates a unit of the dead one's class where it fell and calls `selectPlayer` on it, re-added to each new unit | 10 deaths of 10 kept the same mission: a live player within 0.01 to 0.04 s of each death, the mission start count unchanged, the agent answering throughout[^spikes]. |
| `endMission "END1"` | The next entry of the `-autotest` list was ready, its agent reporting the next mission start count, in 1.63 to 1.99 s, 10 of 10[^spikes]. |

**Forces:** the first fallback of `SPK-05`, which changes no promise: the agent registers the `Killed` handler at every singleplayer mission start (`RST-05`), the new unit joins the dead unit's group so the session keeps the groups it started with, and the body goes on the entity recorder's list for the next reset to delete. `endMission` under `-autotest` works as `P-08` level B assumes, in under 3 s; the second fallback (`-init=playMission`) is not needed.

## SPK-04, singleplayer half: passed

Twenty checks ran in one singleplayer session, each recording the state it started from and then creating a soldier, a quad bike and a box through the API, a soldier in a new group and a container through its own SQF, setting another date and an overcast of 0.9 with fog and rain, and changing the player one of four ways: killing it; putting it in the driver's seat of an MRAP; moving it, hurting it and taking its weapons and backpack; or giving it a backpack, a prone animation, another direction and position and damage. Every check saw the same start state as the first: position, direction, damage, alive, on foot, loadout, animation state, date, weather, and the counts of vehicles, units, dead, groups and mission objects[^twenty].

| Run | Resets | Mean | Max |
|---|---|---|---|
| 1 | 20 | 0.369 s | 0.577 s |
| 2, with the unit set down again while it slides | 20 | 0.352 s | 0.426 s |
| 3, the same | 20 | 0.351 s | 0.418 s |

The first reset of a session, with nothing to undo, takes about 0.04 s; a reset after a check that changed everything takes about 0.4 s, most of it waiting for deleted objects to be gone and for the weather to read back[^twenty].

Three things the reset needed that the spec did not say, found on the way[^demo][^twenty]:

- In the first frames of a singleplayer mission the player's unit is at height 0 ASL, before the engine sets it on the terrain (the VR terrain is at 5 m ASL). A baseline captured then put the player 5 m under the ground at every reset, which the engine corrected, and the verification failed with the player 5.00 m from its baseline. The baseline is captured once the player stands on the ground.
- `switchMove ""` leaves a unit that came out of a vehicle with its rifle raised, where the mission started it lowered. The reset puts the unit back in the animation state the baseline recorded.
- A unit just pulled out of a vehicle can still slide after it is set down: one check started 9 cm from the baseline, inside the 0.1 m the verification allows. While the unit is more than 2 cm from its baseline position, the reset sets it there again, for up to a second.

**Forces:** `RST-02` lists the animation state and says the agent captures the baseline once its player stands on the ground; `P-02` step 7 and `P-06` step 4 say so. `NFR-PF-04`'s 2 s holds in singleplayer by a wide margin; its dedicated half runs in M2. Nothing the singleplayer reset was asked to restore is left for `@reset mission`.

## On the way: CBA's keybinds and a created vehicle

- `CBA_fnc_getKeybind` in CBA 3.19.0 returns `[mod, action, display name, down code, up code, [DIK, [shift, ctrl, alt]], hold, hold delay, [[DIK, [shift, ctrl, alt]], ...]]`, the last element every key bound to it, and matches the mod and action names case-insensitively; for a keybind that is not registered it returns nothing[^probe].
- CBA keeps its keybinds in the namespace `cba_keybinding_actions`, a location, one variable per keybind named `<mod>$<action>` in lower case, such as `grappling hooks$agh_letgo`. Its public functions list none of them, so naming the keybinds a mod has (`ACT-03`) reads those variable names[^probe].
- Three `C_Offroad_01_F` made with `createVehicle` had the same textures, the default `offroad_01_ext_co.paa`: no randomisation showed in three tries[^probe].

[^spikes]: `crates/shakedown/tests/spikes_m01.rs`, `spk_12_spk_10_spk_05_singleplayer_p1`, 2026-11-05.
[^probe]: `crates/shakedown/tests/spikes_m01.rs`, `int_cba_01_keybind_registry_probe`, 2026-11-05.
[^twenty]: `crates/shakedown/tests/m01.rs`, `nfr_rl_02_spk_04_twenty_checks_each_see_the_start_state`, and `runner.log`, 2026-11-05.
[^demo]: The demo runs against one session of `e2e/grappling-hook` and their reset lines in `runner.log`, 2026-11-05.
