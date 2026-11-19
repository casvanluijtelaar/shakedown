---
type: Finding
id: m15-1
milestone: M15
title: The look-for box covers the action menu
description: "In p1's window the box titled Shakedown: look for lies over the game's action menu, so the manual tester cannot see which action they select."
raised_by: developer
for: developer
checks:
  - "crates/shakedown/tests/m15.rs us_083_m15_1_m15_2_man_01_the_box_leaves_the_action_menu_free_and_p1_opens_the_crate"
  - "US-083 MAN-01 m15-1 the look-for box on p1's screen, read through exec, lies in MAN-01's top-centre column: from 11.5 grid units off the safe zone's left edge to 15.3 off its right, at most 24 wide and centred, half a unit below the top, apart from the default layout's action menu, communication menu, vehicle info, weapon info, stance, stamina bar and hints, in dedicated and in singleplayer; the reset removes it"
stories: [US-083]
generated: { by: "claude-code/developer", at: 2026-11-19T12:25:01-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-19T12:35:20-08:00, note: "Commits 9e490e1 and 06ec6f5: the box stands at the top centre, between the default layout's vehicle info and weapon info in Arma 3's GUI grid units (from 11.5 units off the safe zone's left edge to 15.3 off its right, at most 24 wide), clear of the action menu, communication menu, weapon info, stance, stamina bar and hints at any aspect ratio and height; MAN-01, spec/contract/check-api.md and docs/handing-over.md say so. crates/shakedown/tests/m15.rs us_083_m15_1_m15_2_man_01_the_box_leaves_the_action_menu_free_and_p1_opens_the_crate proves it on p1 in a dedicated session." }
verified:
  - { by: "claude-code/tester", at: 2026-11-19T12:50:05-08:00 }
---

# The look-for box covers the action menu

## What the spec promises

`MAN-01` (should): "Put a manual tester at the start of a situation set up by a check, with a list of what to look for, and leave the session up for them". `US-083`: "As a manual tester, I want the framework to set up the situation and tell me what to look for, so that I judge how it looks and feels instead of pasting into the debug console to get there." The list is there to help the person judge the situation in the game, and judging it means using the game's own HUD, the action menu first of all.

## What the system does

The owner, `human:casvanluijtelaar`, ran the M15 demo look (exit criterion 3) on 2026-11-19, `cd e2e; ..\target\shakedown\shakedown.exe run --setup dedicated handover/US-083-crate`, and looked through `p1`'s window over Shadow PC. In their words:

> "the top left window overlays the scroll menu, so hard to see what I selected."

Read through `shakedown exec --machine p1` in the same situation, the box spans `x` from -0.077 to 0.324 and `y` from 0.053 to 0.235 in screen coordinates, while the game's default layout puts the action menu at `x` -0.051 to 0.062 from `y` 0.078 down: the box lies over it.

## What it costs

The manual tester is handed a situation in order to use it: open the crate, fire the hook, climb. With the box over the action menu they cannot read which entry is selected, so the first item of nearly every list ("the action menu offers ...") is the one they cannot judge.

## The check that holds it

- `crates/shakedown/tests/m15.rs us_083_m15_1_m15_2_man_01_the_box_leaves_the_action_menu_free_and_p1_opens_the_crate`, a framework test in a dedicated session. The tester adds the acceptance checks.
