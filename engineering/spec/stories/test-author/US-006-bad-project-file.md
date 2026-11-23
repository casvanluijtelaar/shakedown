---
type: User Story
title: "A malformed project file or an unknown setup is refused with the key at fault"
description: "A `shakedown.toml` with an unknown key, a missing required key or a wrong type, or a start that names a setup the file does not have, is refused before anything starts, naming the key and its line."
status: stable
id: US-006
role: test-author
kind: mistake
requirements: [CFG-01, NFR-US-01]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-006 A malformed project file or an unknown setup is refused with the key at fault

As a test author, I want to be told which key on which line of my project file is wrong, so that I fix the file instead of guessing.

## Acceptance criteria

1. A project file with an unknown key, a missing `game`, or `clients = "two"` makes `session start`, `run` and `list` exit 3 with a message naming the key and its line; the MCP tools return `isError` with code `config-error`.
2. `session start --setup nosuch` exits 3 naming the setups the file declares; `session_start` returns `isError` with code `unknown-setup`.
3. After either refusal, no game process was started.

## Next to

- [US-001](US-001-singleplayer-session.md) (feature): start a singleplayer session.
