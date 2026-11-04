---
type: User Story
title: "Code for a machine the session does not have, or code that does not compile"
description: "Code sent to a machine the session does not have, or that does not compile there, ends a check as the author's mistake, naming the machine or the engine's message, and is refused by the command line and the tools."
status: stable
id: US-014
role: test-author
kind: mistake
requirements: [CTL-04, DM-04, AGT-02]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-014 Code for a machine the session does not have, or code that does not compile

As a test author, I want to be told when I name a machine that does not exist or send code that does not compile, so that I fix my check instead of suspecting the mod.

## Acceptance criteria

1. A check calling `["p3", {1}] call skd_fnc_on` in a session with two clients ends `fail` with reason kind `check-error` and a message naming `p3` and the machines the session has.
2. A check sending code with a syntax error ends `fail` / `check-error` with the engine's compile message in the reason.
3. `shakedown exec --machine p3 --code "1"` exits 3 naming the machines that exist; the MCP tool `exec` returns `isError` with code `unknown-machine`, and for code that does not compile, `compile-error` with the engine's message.

## Next to

- [US-013](US-013-run-code-on-machine.md) (feature): run code on a machine.
