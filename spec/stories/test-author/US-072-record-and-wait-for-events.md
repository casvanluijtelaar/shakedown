---
type: User Story
title: "Record a mod's events on every machine and wait for one"
description: "A check records a named CBA event or engine mission event on chosen machines with the clocks of each, waits for one with a ceiling, and finds the events in the result file; the recording ends with the check."
status: draft
id: US-072
role: test-author
kind: feature
requirements: [EVT-01, OBS-05, RES-01]
---

# US-072 Record a mod's events on every machine and wait for one

As a test author, I want the framework to record the mod's events for me, so that my check asserts on what the mod announced, on each machine and in order, without installing and clearing handlers of its own.

## Acceptance criteria

1. A check records the fixture's CBA event on `server`, `p1` and `p2`, fires the fixture's action on `p1`, and waits for the event on `p2` with a ceiling; the wait returns the event's arguments as `p2` received them.
2. The check reads every recorded event of that name per machine, each with its arguments and the clocks of `OBS-05`, and the result file holds them under the check.
3. An engine mission event, `EntityKilled`, is recorded on the server the same way.
4. In the next check no handler the recording added is left on any machine: the fixture's event fired again records nothing until the check records it.

## Next to

- [US-073](US-073-awaited-event-never-comes.md) (error): an awaited event never comes.
