---
type: User Story
title: "A run is stopped by hand"
description: "A run stopped with Ctrl+C or `run stop` ends its executing check invalid, lists the rest as not run, writes its result file, and leaves nothing of its own running."
status: stable
id: US-034
role: test-author
kind: system fail
requirements: [RUN-05, RES-01, STP-02]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-034 A run is stopped by hand

As a test author, I want to stop a run at any moment and still get a result file, so that stopping a run never costs me the results it already had or leaves the game in my way.

## Acceptance criteria

1. Ctrl+C on `shakedown run` during the second of three checks ends that check `invalid` with reason kind `run-stopped`, lists the third in `not_run` with reason `run-stopped`, writes a valid result file with `run.state` `stopped`, and exits 2.
2. `run_stop` through MCP does the same and returns state `stopped`.
3. A run that started its own session with `--setup` leaves no game process after it is stopped; a run against an existing session leaves that session `ready`.

## Next to

- [US-033](US-033-full-suite-walk-away.md) (feature): run the full suite and walk away.
