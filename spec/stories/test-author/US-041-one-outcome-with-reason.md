---
type: User Story
title: "Every check ends pass, fail or invalid with its reason"
description: "Every check execution ends with exactly one outcome, and every outcome that is not pass carries a reason kind and a message, so a test author knows whether the mod or the environment is at fault."
status: draft
id: US-041
role: test-author
kind: feature
requirements: [OUT-01, DM-01, DM-02, DM-05, DM-06]
---

# US-041 Every check ends pass, fail or invalid with its reason

As a test author, I want to get pass, fail or invalid with a reason for every check, and an exit code that says which, so that I trust a fail to mean the mod and an invalid to mean the environment.

## Acceptance criteria

1. A run of a passing check, a check that calls `["broken on purpose"] call skd_fnc_fail` and a check that calls `["no network"] call skd_fnc_invalid` gives `pass`, `fail` / `explicit` and `invalid` / `explicit` with those messages; `summary` counts 1, 1 and 1 with `total` 3, and the run exits 1.
2. A run whose only non-pass outcome is `invalid` exits 2; a run of only passing checks exits 0.
3. When an assertion fails and a machine is lost after it, the check stays `fail`; when the machine is lost first, it is `invalid`.

## Next to

- [US-015](US-015-machine-stops-answering.md) (system fail): a machine stops answering.
- [US-016](US-016-game-crashes-during-check.md) (system fail): the game crashes.
