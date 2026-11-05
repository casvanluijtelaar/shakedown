---
type: User Story
title: "A wait's condition raises a script error or returns no Boolean"
description: "A wait whose condition raises a script error or returns something other than a Boolean ends at once with that cause, instead of polling a broken condition to its ceiling."
status: stable
id: US-062
role: test-author
kind: error
requirements: [OBS-01, ERR-03, DM-04]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# US-062 A wait's condition raises a script error or returns no Boolean

As a test author, I want a wait on a broken condition to stop at once and say why, so that I do not wait out a ceiling and then read a failure that blames the mod.

## Acceptance criteria

1. `["p1", {_skdfx_undefined > 1}, "flag above one", 10] call skd_fnc_waitUntil` ends the check `fail` / `script-error` within 2 s, not at its 10 s ceiling, naming `p1` and the error, with the error listed once in the result file.
2. `["p1", {"yes"}, "a string, not a Boolean"] call skd_fnc_waitUntil` ends the check `fail` / `check-error` at once with a message naming the description and saying the condition returned a String, not a Boolean; `skd_fnc_tryWaitUntil` with either condition ends the check the same way instead of returning `false`.
3. `shakedown wait --machine p1 --condition "_skdfx_undefined > 1" --json` exits 0 within 2 s with `met` `false` and an `error` naming the script error; the MCP tool `wait_until` returns the same.

## Next to

- [US-023](US-023-wait-with-ceiling.md) (feature): wait for an outcome.
