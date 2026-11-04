---
type: User Story
title: "A reset that cannot complete makes the next check invalid"
description: "When the reset before a check does not complete within its ceiling, that check is invalid with what did not match, and the session is broken instead of running checks on a dirty state."
status: draft
id: US-010
role: test-author
kind: system fail
requirements: [RST-04, RST-03, P-07]
---

# US-010 A reset that cannot complete makes the next check invalid

As a test author, I want to have a reset that cannot restore the start state stop the run with what did not match, so that no check runs on a state left over from another and fails for it.

## Acceptance criteria

1. A check registers a cleanup with `skd_fnc_cleanup` that never ends; the next check of the run ends `invalid` with reason kind `reset-failed` and a message that names the cleanup that did not end, and each following check ends `invalid` / `session-broken`.
2. `status` then shows the session `broken` with that reason, and `shakedown reset` on it exits 4 saying the session is broken and naming `reload` (game restart) and `session stop` as the ways out.
3. `shakedown reset` on a `ready` session exits 0 and reports state `done` and its duration; a failed one reports state `failed` and lists each mismatch.

## Next to

- [US-008](US-008-check-starts-from-baseline.md) (feature): every check starts from the session's start state.
