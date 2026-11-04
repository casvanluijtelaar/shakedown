---
type: User Story
title: "The mod throws a script error on one client only"
description: "A script error on one machine during a check fails it, and the result names the machine, the error, its file and line, and the step that caused it."
status: stable
id: US-028
role: test-author
kind: error
requirements: [ERR-01, ERR-03, OBS-05, RES-02]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-028 The mod throws a script error on one client only

As a test author, I want to have a script error on any one machine fail the check and name the machine, the error and the step that caused it, so that an error only one client sees is never missed.

## Acceptance criteria

1. A check whose step `trigger on p2` runs `[] call skdfx_fnc_throw` on `p2` through `skd_fnc_on` ends `fail` with reason kind `script-error`, `reason.machine` `p2`, and a message with the error's text and `fn_throw.sqf`; the run exits 1.
2. The result file's `errors` lists that error with machine `p2`, its text, file, line and stack, `step_path` `["trigger on p2"]`, `source` `handler`, and the four clocks.
3. The same holds when the mod's function is run with `spawn` or reaches `p2` through `remoteExec` from the server.

## Next to

- [US-037](US-037-write-check-in-sqf.md) (feature): write a check in SQF with steps.
