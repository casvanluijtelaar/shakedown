---
type: User Story
title: "Code sent to a machine raises a script error"
description: "Code that raises a script error on the machine it was sent to still returns its value; the error fails the check it ran in, naming the machine and the step, and comes back beside the value from exec."
status: stable
id: US-061
role: test-author
kind: error
requirements: [CTL-05, ERR-01, ERR-03, ERR-05]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# US-061 Code sent to a machine raises a script error

As a test author, I want a script error in code I send to a machine to come back with the result, so that I see what broke where instead of a value that silently went wrong.

## Acceptance criteria

1. A check whose step `throw on p1` runs `["p1", {[] call skdfx_fnc_throw; 7}] call skd_fnc_on` gets `7` back and runs on to the end of its body, then ends `fail` / `script-error` with `reason.machine` `p1`, a message holding `_skdfx_undefined` and `fn_throw.sqf`, and `evidence.failed_step` `["throw on p1"]`.
2. `shakedown exec --machine p1 --code "[] call skdfx_fnc_throw; 7" --json` exits 0 with `results.p1` in state `done` and value `7`, and `errors` holding the error with machine `p1`, its text, file and line; the MCP tool `exec` returns the same JSON and not `isError`.
3. `shakedown errors --machine p1` afterwards lists that error with its time, as an error raised outside any check.

## Next to

- [US-013](US-013-run-code-on-machine.md) (feature): run code on a machine.
- [US-014](US-014-bad-machine-or-code.md) (mistake): the code names a machine the session lacks or does not compile.
