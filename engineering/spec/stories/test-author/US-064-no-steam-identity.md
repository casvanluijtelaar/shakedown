---
type: User Story
title: "A run's result file and stored logs hold no Steam name and no Steam ID"
description: "Every game process runs under its machine name, and the result file, the run folder's logs, the logs output and a stopped session's RPTs hold neither the Steam ID nor the Steam names of the account the game runs on."
status: stable
id: US-064
role: test-author
kind: feature
requirements: [NFR-PV-02, NFR-PV-01, LOG-02, INT-STM-04, DM-11]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# US-064 A run's result file and stored logs hold no Steam name and no Steam ID

As a test author, I want what a run leaves behind to hold no trace of the Steam account the game runs on, so that I can share a result or a log without publishing who I am.

## Acceptance criteria

1. After a run on a dedicated session with two clients and `session stop`, a search of `result.json`, every file under the run folder's `logs/` and `checks/`, and every machine's RPT in the session folder for the test machine's steamID64, its Steam account name and its persona name finds none of the three.
2. Every game process of the session has `-name=<machine name>` on its command line, and `name player` read on `p1` and `p2` returns `p1` and `p2`.
3. `shakedown logs --machine server --last 500` and `shakedown logs --machine p1 --last 500` print none of the three either.
4. After a dedicated start that fails once its game clients have logged in (`US-004`), the output of `session start`, the `reason` that `status` shows and every RPT of that session's machines hold none of the three.

## Next to

- [US-065](US-065-steam-identity-in-check.md) (mistake): a check itself puts the Steam ID or name into its result.
- [US-042](US-042-logs-with-run.md) (feature): each machine's log is kept with the run.
