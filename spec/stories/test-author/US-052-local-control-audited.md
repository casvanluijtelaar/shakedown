---
type: User Story
title: "Only the local user controls the session, and every piece of code sent to it is recorded"
description: "The framework listens on loopback only, keeps its tokens out of every command line, file and output, and records every piece of code sent to a machine."
status: draft
id: US-052
role: test-author
kind: feature
requirements: [SEC-01, SEC-02, SEC-04, NFR-SC-01, NFR-SC-03, SEC-06]
---

# US-052 Only the local user controls the session, and every piece of code sent to it is recorded

As a test author, I want to know that only I can run code in my test session and see every piece of code that was run in it, so that a test machine is not a door into my game.

## Acceptance criteria

1. During a session, `netstat` shows every socket the runner and the game processes' extensions opened for Shakedown bound to `127.0.0.1` only.
2. Every `exec` from the command line, from MCP and from a check appears in the session folder's `audit.jsonl` with its time, its source (command line, MCP or the check id), its target machine and its code.
3. No session token or runner token appears in any process's command line, in the result file, in a log the framework writes or in any tool output.

## Next to

- [US-053](US-053-no-token-no-code.md) (malicious intent): a process without the token tries to run code.
