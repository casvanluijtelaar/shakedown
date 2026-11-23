---
type: User Story
title: "Run the full suite and walk away"
description: "A test author runs every check in a setup with one command that starts the session, runs, and stops it, leaving one result file, the logs, and nothing running."
status: stable
id: US-033
role: test-author
kind: feature
requirements: [RUN-04, STP-02, RES-01, RES-04, LOG-04, CFG-02, NFR-PV-03]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# US-033 Run the full suite and walk away

As a test author, I want to run the whole suite with one command and walk away, so that I come back to one result file, its logs, and a machine with nothing left running.

## Acceptance criteria

1. `shakedown run --setup dedicated-2` with no session running starts one, runs every check, stops the session and exits 0, 1 or 2 by the outcomes; afterwards no `arma3_x64` or `arma3server_x64` process runs and `status` shows no session.
2. The run folder holds one `result.json` valid against `result.schema.json`, `logs/<machine>.log` for each machine and `checks/<n>/<machine>.log` for each check execution, and every path in `result.json` is relative to the run folder and exists.
3. `.shakedown/.gitignore` exists and ignores everything in `.shakedown/`, and the framework wrote nothing else into the project.

## Next to

- [US-034](US-034-run-stopped-by-hand.md) (system fail): the run is stopped by hand.
- [US-016](US-016-game-crashes-during-check.md) (system fail): the game crashes during a check.
- [US-063](US-063-run-ceiling.md) (system fail): the run reaches its runner ceiling.
