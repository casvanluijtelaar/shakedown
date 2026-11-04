---
type: User Story
title: "A check that changes what a reset cannot name gets a mission restart"
description: "A check whose header says `@reset mission` starts in a freshly restarted session mission, for state such as global variables and mod settings that a reset in place cannot restore."
status: stable
id: US-011
role: test-author
kind: feature
requirements: [RST-06, RST-02, API-01]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-011 A check that changes what a reset cannot name gets a mission restart

As a test author, I want to ask for a mission restart before a check that depends on mission-wide state, so that global variables and mod settings left by earlier checks cannot reach it.

## Acceptance criteria

1. A check that sets a public `missionNamespace` variable is followed by a check with `// @reset mission` that asserts the variable is `nil` on every machine; it passes.
2. On a dedicated session, the result file of that run shows the clients and headless clients with a start number one higher for the check after the mission restart, and every machine `ready` at the end.

## Next to

- [US-012](US-012-mission-reset-fails.md) (system fail): the mission restart does not bring every machine back.
