---
type: User Story
title: "A real key to a machine with no window"
description: "Sending a real key to the server or a headless client ends the check as the author's mistake, naming the game clients that have a window."
status: deprecated
id: US-092
role: test-author
kind: mistake
requirements: [ACT-05, DM-04]
---

# US-092 A real key to a machine with no window

As a test author, I want a key sent where no window exists to be refused, so that a check never passes on a key that went nowhere.

Deprecated with `ACT-05` (`SPK-11`, research `35-spikes-ace-and-keys.md`).

## Acceptance criteria

1. Sending a key to `server` or `hc1` ends the check `fail` / `check-error` naming the game clients of the session.

## Next to

- [US-091](US-091-send-real-key.md) (feature): send a real key to one client's window.
