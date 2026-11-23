---
type: User Story
title: "A mod loaded in another version than the project names"
description: "When a machine loads a mod in another version than the project file names, the start fails before any check runs, naming the mod, the version found and the version wanted."
status: draft
id: US-086
role: test-author
kind: error
requirements: [ENV-01, INT-STM-03, SES-04]
---

# US-086 A mod loaded in another version than the project names

As a test author, I want a Workshop update of a mod my checks depend on to stop the start, so that no verdict is read against a version I did not choose.

## Acceptance criteria

1. With `version` of the fixture mod set in `shakedown.toml` to a version other than the one installed, `session start` ends `failed` naming the mod, the version found and the one wanted, and no game process runs after it.
2. With the right version, or with no `version` key, the start is `ready` as before.

## Next to

- [US-085](US-085-environment-checked-in-full.md) (feature): the environment is checked in full.
