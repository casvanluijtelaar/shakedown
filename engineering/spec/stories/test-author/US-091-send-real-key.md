---
type: User Story
title: "Send a real key to one game client's window"
description: "A check sends a real key press to one game client's window, in the background, without the desktop's focus or input, so a promise about the key itself is tested as a player presses it."
status: deprecated
id: US-091
role: test-author
kind: feature
requirements: [ACT-05, NFR-LG-02]
---

# US-091 Send a real key to one game client's window

As a test author, I want to press a real key in one client, so that I can test whether the mod consumes a key or passes it on to the game, which calling the keybind's code cannot show.

Deprecated with `ACT-05`: a key posted to the game's window does not reach the game (`SPK-11`, research `35-spikes-ace-and-keys.md`).

## Acceptance criteria

1. A check sends a key bound to the fixture's CBA keybind to `p1`'s window while another window has the focus: the keybind's code runs once on `p1`, and the focused window and the foreground are unchanged.
2. A key the fixture's handler passes on reaches the game's own action on `p1`, and a key it consumes does not.
3. It holds with the desktop shown and with no desktop shown.

## Next to

- [US-092](US-092-real-key-without-window.md) (mistake): a real key to a machine with no window.
