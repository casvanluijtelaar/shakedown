---
type: User Story
title: "An awaited event never comes"
description: "A wait for an event that never fires ends the check fail at its ceiling, naming the event and the machine and listing the events of any name the check recorded there."
status: draft
id: US-073
role: test-author
kind: error
requirements: [EVT-01, OBS-04, DM-03]
---

# US-073 An awaited event never comes

As a test author, I want a missing event to fail with what did arrive, so that I can tell an event the mod never sent from one it sent under another name or to another machine.

## Acceptance criteria

1. A wait for the fixture's CBA event on `p2` with a ceiling of 3 s, when only `p1` raised it locally, ends `fail` / `wait-ceiling` after 3 s naming the event and `p2`, with the events recorded on `p2` (none) and on `p1` (one).
2. A wait for an event the check did not record ends `fail` / `check-error` saying the event is not recorded on that machine.

## Next to

- [US-072](US-072-record-and-wait-for-events.md) (feature): record a mod's events and wait for one.
