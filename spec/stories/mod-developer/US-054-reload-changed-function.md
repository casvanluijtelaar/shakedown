---
type: User Story
title: "Reload a changed function and rerun the checks a finding names"
description: "A mod developer changes one function, reloads it into the running session in one step, and reruns the checks a finding names, with the verdict in about a game start."
status: stable
id: US-054
role: mod-developer
kind: feature
requirements: [RLD-01, RLD-02, RLD-03, RLD-06, RUN-01, NFR-PF-02]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-054 Reload a changed function and rerun the checks a finding names

As a mod developer, I want to reload one changed function into the running session and rerun the three checks a finding names, so that I know soon after my fix whether it works, without stopping and starting the session myself.

## Acceptance criteria

1. With `@skd_fixture` watched in a staged copy of its sources and a `rebuild` that copies its `fn_version.sqf` into the installed copy (`FIX-03`), after the staged file is changed from `"1"` to `"2"`, `shakedown reload` with no files exits 0 and reports level `game-restart`, the file with its reason, every machine of the session reached, and the time taken.
2. Then `["all", {call skdfx_fnc_version}] call skd_fnc_onEach` returns `"2"` on every machine, every machine has a start number one higher, and the result file of the rerun marks the check `reload` `full`.
3. With a `ready` session, the time from `reload` to the outcome of a check tagged with the finding, whose body takes under 10 s, is within `NFR-PF-02`'s target plus 10 s, in singleplayer and on a dedicated session with two clients.

## Next to

- [US-056](US-056-change-needs-rebuild.md) (mistake): the change needs a rebuild and a game restart.
- [US-057](US-057-reload-fails.md) (system fail): the reload does not complete.
- [US-058](US-058-reload-during-run.md) (mistake): the reload is asked while a run is active.
