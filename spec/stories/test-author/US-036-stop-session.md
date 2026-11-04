---
type: User Story
title: "Stopping a session leaves nothing running"
description: "`session stop` stops every process of the session and returns only when none is left, with no port held and no mission copy left."
status: stable
id: US-036
role: test-author
kind: feature
requirements: [SES-07, STP-04]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-036 Stopping a session leaves nothing running

As a test author, I want to stop the session and know that nothing of it is left, so that the machine is free for the next session or for the owner.

## Acceptance criteria

1. `shakedown session stop` exits 0, and afterwards no `arma3_x64` or `arma3server_x64` process the session started runs, no port from the setup's base port to base port plus 4 is held, and `<game>\MPMissions` holds no mission copy of the session.
2. `session_stop` through MCP returns the session with state `stopped`, and `status` then shows no session.

## Next to

- [US-035](US-035-runner-dies.md) (system fail): the runner dies.
