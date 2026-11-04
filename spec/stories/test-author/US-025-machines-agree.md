---
type: User Story
title: "Assert that every machine agrees with the server"
description: "A test author reads one value on several machines and asserts that each meets an expectation, or that every machine agrees with the first one named."
status: draft
id: US-025
role: test-author
kind: feature
requirements: [OBS-06, CTL-01, API-03]
---

# US-025 Assert that every machine agrees with the server

As a test author, I want to assert in one call that the server and every client see the same value, so that a replication defect shows up as a failed check.

## Acceptance criteria

1. After `[5] call skdfx_fnc_setFlag` on the server, `[["server", "p1", "p2"], {missionNamespace getVariable "skdfx_flag"}, "flag agrees"] call skd_fnc_expectAgree` passes and returns a HashMap with `5` for each machine.
2. `["clients", {missionNamespace getVariable "skdfx_flag"}, "==", 5, "clients see 5"] call skd_fnc_expectEach` passes, and positions agree within the given `tolerance` in metres.

## Next to

- [US-026](US-026-machine-disagrees.md) (error): one machine disagrees.
