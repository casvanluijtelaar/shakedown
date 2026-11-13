---
type: User Story
title: "A check sends a client to another server"
description: "When code a check sends makes a client join a server other than the session's own, the client is ended and the check fails as the author's mistake."
status: draft
id: US-088
role: test-author
kind: mistake
requirements: [SEC-05, DM-04, NFR-LG-02]
---

# US-088 A check sends a client to another server

As a test author, I want the framework to keep every client on the session's own loopback server, so that no check of mine ever plays against a public server.

## Acceptance criteria

1. Code sent to `p2` that makes it join another server ends `p2`'s process as soon as its agent starts in the other mission, and the check ends `fail` / `check-error` saying a client may join only the session's own server.
2. The session is `broken` after it, with that reason, and a `game-restart` reload brings every machine back.

## Next to

- [US-085](US-085-environment-checked-in-full.md) (feature): the environment is checked in full.
