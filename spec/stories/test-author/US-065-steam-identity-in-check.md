---
type: User Story
title: "A check puts the Steam ID or the Steam name into its result"
description: "When a check attaches, logs or fails with the Steam ID or the Steam name of the account the game runs on, the result file and the logs show placeholders in their place."
status: draft
id: US-065
role: test-author
kind: mistake
requirements: [LOG-02, NFR-PV-01, NFR-PV-02]
---

# US-065 A check puts the Steam ID or the Steam name into its result

As a test author, I want a value I read from the game to be redacted even when my own check is what wrote it out, so that a careless line in a check cannot leak who I am.

## Acceptance criteria

1. A check that attaches `getPlayerUID player` and `profileNameSteam` with `skd_fnc_evidence`, then calls `[format ["uid %1 name %2", getPlayerUID player, profileNameSteam]] call skd_fnc_fail`, ends `fail` / `explicit`; in the result file the attached values read `<steamid>` and `<steamname>`, and the message reads `uid <steamid> name <steamname>`.
2. A step of that check that writes the same two values with `diag_log` leaves `<steamid>` and `<steamname>` in their place in the check's log excerpt in the run folder and in `shakedown logs --machine p1 --last 50`.

## Next to

- [US-064](US-064-no-steam-identity.md) (feature): a run's result file and stored logs hold no Steam identity.
