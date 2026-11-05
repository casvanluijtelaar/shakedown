---
type: User Story
title: "Run code on a chosen machine and get the result back as data"
description: "A test author runs SQF on the server, a named player's machine, a headless client or a group of machines and gets the returned value back as data."
status: stable
id: US-013
role: test-author
kind: feature
requirements: [CTL-01, CTL-02, CTL-03, DM-13, DM-14, NFR-PF-03]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-013 Run code on a chosen machine and get the result back as data

As a test author, I want to run code on the server, one player's machine, a headless client or every client, and get the value back as data, so that I can set things up and read state exactly where it lives.

## Acceptance criteria

1. In a check, `["p2", {[_this select 0, name player]}, [42]] call skd_fnc_on` returns an array whose first element is `42`, and an object returned from one machine can be passed in `args` to code on another machine and arrives there as the same object.
2. `["all", {clientOwner}] call skd_fnc_onEach` returns a HashMap with one key per machine of the session.
3. `shakedown exec --machine server --code "sleep 1; 1 + 1" --json` prints `results.server` with state `done`, value `2` and a duration of at least 1 s; the MCP tool `exec` returns the same JSON.
4. 100 `exec` calls of `1 + 1` to each kind of machine (singleplayer `p1`, dedicated `server`, a client, a headless client) each answer `2`, with a 95th percentile duration under 1 s.

## Next to

- [US-014](US-014-bad-machine-or-code.md) (mistake): the code names a machine the session lacks or does not compile.
- [US-015](US-015-machine-stops-answering.md) (system fail): the machine stops answering.
- [US-061](US-061-code-raises-error.md) (error): the code raises a script error on its machine.
