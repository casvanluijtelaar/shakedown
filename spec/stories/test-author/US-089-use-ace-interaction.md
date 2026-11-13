---
type: User Story
title: "Make a player use a mod's ACE interaction action"
description: "A check makes the player of a named machine use an action of a mod's ACE interaction menu, on an object or on itself, found by its path, through ACE's own path."
status: draft
id: US-089
role: test-author
kind: feature
requirements: [ACT-04, INT-ACE-03, RES-01]
---

# US-089 Make a player use a mod's ACE interaction action

As a test author, I want to use the mod's actions as an ACE player reaches them, so that a promise about ACE interaction is tested through ACE, not around it.

## Acceptance criteria

1. With ACE loaded, a check makes `p1` use the fixture's ACE action on a crate by its path, and the action's effect is seen on the server and on every client.
2. The same holds for the fixture's self action on `p1`, in singleplayer and on a dedicated session with two clients.
3. The result file records the ACE version each machine loaded.

## Next to

- [US-090](US-090-ace-action-cannot-be-used.md) (error): the ACE action cannot be used.
