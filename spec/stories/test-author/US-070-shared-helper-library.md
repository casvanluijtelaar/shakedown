---
type: User Story
title: "Checks share helper code from a library folder"
description: "Helper code in the project's library folder is compiled on every machine and called by every check and by code sent to any machine, so a helper is written once instead of copied into each check."
status: draft
id: US-070
role: test-author
kind: feature
requirements: [API-06, API-04, RLD-04]
---

# US-070 Checks share helper code from a library folder

As a test author, I want to keep the helpers my checks share in one place, so that a change to one is one edit and every check uses the same code.

## Acceptance criteria

1. A file `tests/_lib/rope.sqf` whose code returns a value is callable as `call skd_lib_rope` in the body of two checks and in code sent with `skd_fnc_on` to `p1` and `p2`, in singleplayer and on a dedicated session with two clients.
2. `shakedown list` and `list` through MCP list neither `rope.sqf` nor any other file of the library folder as a check, and a run with no selection runs none of them.
3. After `tests/_lib/rope.sqf` is edited and `shakedown reload` is run, the next check uses the edited helper on every machine, without a game restart; the reload reports the library file at level `none`.
4. A project whose `shakedown.toml` sets `lib = "helpers"` takes its library from `helpers/` instead.

## Next to

- [US-071](US-071-library-file-does-not-compile.md) (mistake): a library file that does not compile.
- [US-040](US-040-edit-check-rerun.md) (feature): edit a check and rerun it with no reload.
