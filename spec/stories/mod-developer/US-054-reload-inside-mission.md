---
type: User Story
title: "Reload a changed function inside the mission and rerun the checks a finding names"
description: "A mod developer changes one function, reloads it into the running session without a restart, and reruns the checks a finding names, with the verdict in seconds."
status: stable
id: US-054
role: mod-developer
kind: feature
requirements: [RLD-01, RLD-02, RLD-03, RLD-05, RUN-01, NFR-PF-01]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-054 Reload a changed function inside the mission and rerun the checks a finding names

As a mod developer, I want to reload one changed function into the running game and rerun the three checks a finding names, so that I know within seconds whether my fix works.

## Acceptance criteria

1. After the installed fixture file `fn_version.sqf` is changed from `"1"` to `"2"`, `shakedown reload` with no files exits 0 and reports level `mission`, the file with level `mission` and its reason, every machine of the session reached, and the time taken.
2. Then `["all", {call skdfx_fnc_version}] call skd_fnc_onEach` returns `"2"` on every machine, every machine keeps its start number, and the result file of the rerun marks the check `reload` `partial` and lists the reload in `reloads`.
3. With a `ready` session, the time from `reload` to the outcome of a check tagged with the finding, whose body takes under 10 s, is under 30 s in singleplayer and on a dedicated session with two clients.

## Next to

- [US-056](US-056-change-needs-rebuild.md) (mistake): the change needs a rebuild and a game restart.
- [US-057](US-057-reload-fails.md) (system fail): the reload does not complete.
- [US-058](US-058-reload-during-run.md) (mistake): the reload is asked while a run is active.
