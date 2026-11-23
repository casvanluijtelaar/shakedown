---
type: User Story
title: "Make a player press a mod's CBA keybind"
description: "A test author makes the player of a named machine press a keybind a mod registered with CBA, running the mod's own key-down and key-up code on that machine."
status: stable
id: US-021
role: test-author
kind: feature
requirements: [ACT-02, API-03]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-021 Make a player press a mod's CBA keybind

As a test author, I want to fire a mod's keybind on a chosen player's machine, so that a feature started by a key is tested through the mod's own key handler.

## Acceptance criteria

1. With the fixture `@skd_fixture_cba` loaded, `["p1", "skd_fixture", "signal"] call skd_fnc_pressKey` returns `true`, and the server then reads `skdfx_signalDown` and `skdfx_signalUp` of 1 on `p1`'s unit.
2. With mode `"down"`, only `skdfx_signalDown` goes up; a later call with mode `"up"` raises `skdfx_signalUp`.

## Next to

- [US-022](US-022-keybind-missing.md) (mistake): the keybind is not registered or CBA is missing.
