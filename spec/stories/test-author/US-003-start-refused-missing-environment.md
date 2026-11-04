---
type: User Story
title: "A start is refused before any process starts when Steam, the game or a mod is missing"
description: "A session start whose environment is incomplete is refused before any game process starts, naming exactly what is missing."
status: draft
id: US-003
role: test-author
kind: system fail
requirements: [SES-03, INT-STM-01, NFR-US-01]
---

# US-003 A start is refused before any process starts when Steam, the game or a mod is missing

As a test author, I want to be told exactly what is missing from my machine before the game starts, so that I fix it instead of reading a log of a game that exited.

## Acceptance criteria

1. With a setup that names a mod whose `path` does not exist, `session start` exits 4 and its message names the mod and the folder it looked for; `session_start` returns `isError` with code `precondition-failed` and the same text.
2. With no `steam` process running, `session start` exits 4 with a message that says Steam is not running and must be started and logged in.
3. After either refusal, `status` shows no session, and no `arma3_x64` or `arma3server_x64` process was started.

## Next to

- [US-001](US-001-singleplayer-session.md) (feature): start a singleplayer session.
- [US-002](US-002-dedicated-session.md) (feature): start a dedicated session.
