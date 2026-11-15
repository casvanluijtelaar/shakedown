---
type: User Story
title: "The game still holds the changed file"
description: "When a machine still holds a changed file after the reload has made it read the framework's release files, the reload applies a game restart instead and reports why, so no check runs on code the game did not take."
status: draft
id: US-082
role: mod-developer
kind: system fail
requirements: [RLD-02, RLD-03, P-08]
---

# US-082 The game still holds the changed file

As a mod developer, I want a reload that could not free my changed file to fall back and say so, so that I never read a verdict about my old code.

## Acceptance criteria

1. When a machine still holds the changed file after it has been made to read the framework's release files twice, `reload` applies level `game-restart`, and its result names the file, the machine holding it and that reason.
2. The check after it sees the new value on every machine.

## Next to

- [US-081](US-081-reload-without-game-restart.md) (feature): a changed function reaches a running session without a game restart.
