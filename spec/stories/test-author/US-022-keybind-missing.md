---
type: User Story
title: "A keybind that is not registered, or a setup without CBA"
description: "Pressing a keybind that is not registered, or on a machine without CBA, ends the check as the author's mistake, listing the keybinds that exist; checks that press no key run without CBA."
status: draft
id: US-022
role: test-author
kind: mistake
requirements: [ACT-03, API-05, NFR-US-01]
---

# US-022 A keybind that is not registered, or a setup without CBA

As a test author, I want to be told when the keybind I name does not exist, with the ones that do, so that a misspelled action name is fixed in the check and not chased in the mod.

## Acceptance criteria

1. `["p1", "skd_fixture", "signl"] call skd_fnc_pressKey` ends `fail` / `check-error` with a message listing the keybinds registered for `skd_fixture`, including `signal`.
2. In a setup without CBA, a check that calls `skd_fnc_pressKey` ends `fail` / `check-error` saying CBA is not loaded on `p1`, and a check that presses no key passes.

## Next to

- [US-021](US-021-press-cba-keybind.md) (feature): press a mod's CBA keybind.
