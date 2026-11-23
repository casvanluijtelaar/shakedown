---
type: Finding
id: m15-2
milestone: M15
title: The hand-over demo's crate offers no Open crate and gives no first aid kit
description: In the owner's look at e2e's handover/US-083-crate, p1's action menu offers only the game's Inventory at the crate, and nothing puts a first aid kit in p1's inventory, though the check's list promises both.
raised_by: developer
for: developer
checks:
  - "crates/shakedown/tests/m15.rs us_083_m15_1_m15_2_man_01_the_box_leaves_the_action_menu_free_and_p1_opens_the_crate"
  - "US-083 MAN-01 m15-2 each of the three items holds on p1 at the hand over: one crate stands 2 m in front of p1, its action Open crate has p1 in its radius, p1 carries no first aid kit, and firing Open crate as the action menu does puts one first aid kit in p1's inventory"
stories: [US-083]
generated: { by: "claude-code/developer", at: 2026-11-19T12:25:01-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-19T12:35:20-08:00, note: "Commits 9e490e1 and 06ec6f5: the framework and fixture were right (the action is on p1); the demo check stood p1 3.01 m from the crate, past the action's 3 m radius, on the clients' shared start spot, with a first aid kit already in its loadout. e2e/tests/handover/US-083-crate.sqf now stands p1 2 m from the crate away from the start spot, takes its kit out, waits on p1 for the action in reach, and lists items that hold; docs/handing-over.md says how. crates/shakedown/tests/m15.rs us_083_m15_1_m15_2_man_01_the_box_leaves_the_action_menu_free_and_p1_opens_the_crate fires the action on p1 and sees the kit arrive." }
verified:
  - { by: "claude-code/tester", at: 2026-11-19T12:50:05-08:00 }
---

# The hand-over demo's crate offers no Open crate and gives no first aid kit

## What the spec promises

`US-083` criterion 2: "`p1`'s window shows the three items on screen, and `p1` stands where the check put it". M15's demo: "the owner, in `p1`'s window, finds the three things to look for on screen and in the command's output". The demo check `e2e/tests/handover/US-083-crate.sqf` lists them:

> 1. An ammo crate stands 3 m in front of you.
> 2. Looking at it, the action menu (mouse wheel) offers Open crate.
> 3. Opening it puts a first aid kit in your inventory.

`spec/contract/fixture-mod.md`: `skdfx_Crate` "adds on every machine with an interface an `addAction` titled exactly `Open crate`, with radius 3 m", and opening it "adds one `FirstAidKit` to the caller's inventory".

## What the system does

The owner, `human:casvanluijtelaar`, ran the M15 demo look (exit criterion 3) on 2026-11-19, `cd e2e; ..\target\shakedown\shakedown.exe run --setup dedicated handover/US-083-crate`, and looked through `p1`'s window over Shadow PC. In their words:

> "theres no option "open crate" just a "inventory" when pressed it opens it but gives me nothing."

Read through `shakedown exec --machine p1` in the same situation: the crate carries the action `Open crate` on `p1`, `p1` stands 3.01 m from the crate (`player distance crate`), past the action's radius of 3 m, and `p1` already carries a `FirstAidKit` in its loadout before the crate is opened. The other game client, `p2`, stands at the same position as `p1`.

## What it costs

The owner's look of M15 is the demo of the milestone: a manual tester handed a situation whose list they cannot confirm on screen cannot tell a broken mod from a broken scene, and items 2 and 3 of the demo cannot be judged at all.

## The check that holds it

- `crates/shakedown/tests/m15.rs us_083_m15_1_m15_2_man_01_the_box_leaves_the_action_menu_free_and_p1_opens_the_crate`, a framework test in a dedicated session that reads, on `p1`, that the action is in reach, fires it and sees the first aid kit arrive. The tester adds the acceptance checks.
