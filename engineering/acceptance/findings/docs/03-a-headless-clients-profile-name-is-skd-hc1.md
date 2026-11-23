---
type: Finding
id: m19-3
milestone: M19
title: profileName on a headless client is skd_hc1, not hc1
description: docs/check-api.md and docs/dedicated-sessions.md show `["all", { profileName }] call skd_fnc_onEach` giving hc1 -> "hc1", but on hc1 profileName is "skd_hc1", in a check and through exec.
raised_by: test
for: writer
checks: []
stories: [US-066]
generated: { by: "claude-code/tester", at: 2026-11-23T20:13:00Z }
addressed:
  - { by: "claude-code/writer", at: 2026-11-23T20:20:15Z, note: "The profileName comments give hc1 -> \"skd_hc1\", and docs/dedicated-sessions.md says a headless client reports its slot name, so it is found by the HashMap key. Changed docs/dedicated-sessions.md and docs/check-api.md." }
verified: []
---

# profileName on a headless client is skd_hc1, not hc1

This finding comes from checking the rewritten `docs/` after M19 closed, not from a milestone round.

## What the spec promises

`DM-11`: "Every game process is started with `-name=<machine name>` and its own profile folder, so a machine is found by its profile name". The docs turn that into a promise about `profileName` on every machine:

- `docs/dedicated-sessions.md`: `private _each = ["all", { profileName }] call skd_fnc_onEach;` with the comment `// _each: server -> "server", p1 -> "p1", p2 -> "p2", hc1 -> "hc1"`.
- `docs/check-api.md`, Running code on machines: `private _names = ["all", { profileName }] call skd_fnc_onEach; // server -> "server", p1 -> "p1", ...`.

## What the system does

On a dedicated session with two clients and a headless client, a check attached that HashMap as evidence: `{"hc1": "skd_hc1", "p1": "p1", "p2": "p2", "server": "server"}`. Through the command line:

```
> shakedown exec --machine all --code "profileName"
hc1  done  "skd_hc1"  (0.021 s)
p1  done  "p1"  (0.012 s)
p2  done  "p2"  (0.011 s)
p3  done  "p3"  (0.003 s)
server  done  "server"  (0.022 s)
```

`research/24-spikes-dedicated.md` records the engine fact: for a headless client the engine reports the name of its `HeadlessClient_F` slot, `skd_hc1`. The contract makes no promise about `profileName` on a headless client, so the pages are what is wrong.

## What it costs

A check that finds a machine by `profileName`, as the pages teach, never finds `hc1`. The comments should give `hc1 -> "skd_hc1"`, or the example should read a value that is the machine name on every kind of machine.

## The check that holds it

None: a document gap. Recorded in `acceptance/coverage/docs.md`.
