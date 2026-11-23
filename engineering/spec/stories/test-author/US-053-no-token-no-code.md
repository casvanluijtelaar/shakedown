---
type: User Story
title: "A process without the token cannot run code in the session"
description: "A local process that connects to the runner or poses as a game process without the right token, or from a process the runner did not start, runs no code, and the refusal is logged."
status: stable
id: US-053
role: test-author
kind: malicious intent
requirements: [SEC-01, SEC-03, NFR-SC-02]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-053 A process without the token cannot run code in the session

As a test author, I want to have any other program on my machine refused when it tries to drive my test session, so that only the framework's own clients and game processes run code in it.

## Acceptance criteria

1. A process that connects to the runner's port and sends a request with a wrong runner token, or a hello as an extension with a wrong session token, is disconnected, no order reaches any machine, and `runner.log` records the refusal.
2. A hello with the right session token from a process id the runner did not start for that machine is refused the same way.

## Next to

- [US-052](US-052-local-control-audited.md) (feature): only the local user controls the session.
