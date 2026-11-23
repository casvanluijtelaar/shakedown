---
type: User Story
title: "An upgrade while Shakedown runs"
description: "Running the one-line script while a runner or an AI agent's shakedown mcp runs from the install folder is refused before any file changes, naming each process and saying to stop it first."
status: draft
id: US-098
role: test-author
kind: mistake
requirements: [DST-03]
---

# US-098 An upgrade while Shakedown runs

As a test author, I want an upgrade that finds Shakedown still running to stop and tell me, so that my runner, which outlives my sessions, and my AI agent's MCP server never leave me with a mix of two versions.

## Acceptance criteria

1. With a runner running a session, and with an AI agent's `shakedown mcp` running, from the install folder, the one-line script refuses, names each `shakedown.exe` process of that folder by its id and command, and says to stop them first; every file of the install is unchanged and the running session still answers `status`.
2. Once those processes are stopped, the same command upgrades.
3. `docs/install.md` says to stop the runner and the AI agent sessions before `winget upgrade`, and what winget does when they still run.

## Next to

- [US-096](US-096-upgrade-or-remove.md) (feature): upgrade or remove an installed Shakedown.
