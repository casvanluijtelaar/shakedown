---
type: Finding
id: m09-2
milestone: M9
title: A runner suspended for some seconds and resumed marks every machine lost and the session broken
description: With the runner's process suspended for about 25 s and then resumed, status shows the session broken and every machine lost for "no heartbeat came for 26 s", where US-060 promises the same session ready with every machine at its start number.
status: draft
raised_by: test
for: developer
checks:
  - "US-060 SES-06 once the runner is resumed status shows the same session ready with every machine at the start number it had"
stories: [US-060]
generated: { by: "claude-code/tester", at: 2026-11-12T13:45:28-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-12T14:05:00-08:00, note: "The heartbeat gap counts only time the runner runs: a watch round 2 s or more late judges nothing and counts every machine's gap from then (HLT-02, P-07, docs/outcomes-and-broken-sessions.md). The m05 game test us_052_us_053_us_060_us_007 holds the runner suspended 30 s and sees the session ready for 15 s after the resume; the named check passes." }
verified: []
---

# A runner suspended for some seconds and resumed marks every machine lost and the session broken

## What the spec promises

`US-060`, criterion 3: "Once the runner's process is resumed, the next `shakedown status` shows the same session id `ready`, with every machine at the start number it had." `SES-06`: "A session belongs to the runner, not to the client that started it." `HLT-02` and `P-07` make a machine whose heartbeat has not come for 10 s `lost`: the heartbeat is the machine's, and its silence is meant to show that the machine stopped answering, not that the runner did. The check passed in the M8 full run (243 of 243); it fails since M9 added the heartbeat watch.

## What the system does

On a ready dedicated session of a server, two clients and a headless client (setup `ded` of the generated project), the runner's process is suspended by the `runner_pid` that `status` gave, while `status`, `run`, and the tools `run_start`, `exec` and `status` are asked, about 25 s in all; each answers as `US-060` criteria 1 and 2 say. The runner is then resumed, and `status` is polled twice a second for 30 s:

```
$ shakedown status --json
{ ..., "session": { "id": "s-20261112-214221", "state": "broken",
    "reason": { "machine": "hc1", "message": "hc1 stopped answering: no heartbeat came for 26 s, its last answer at 2026-11-12T21:43:15.570Z; it was asked nothing at the time", ... },
    "machines": [ server lost, p1 lost, p2 lost, hc1 lost ] } }
```

Every game process kept running throughout; only the runner was stopped. Seen in the full run of 2026-11-12 and in a run of the check alone after it.

## What it costs

A test author whose runner stalls for more than 10 s, whether suspended by a tool, paused by a debugger or starved by the machine, loses the whole session and has to restart the game, though every machine answered all along. `US-060` exists so the session survives exactly that. A test author reading the result would also be told four machines "stopped answering", which is the environment blamed for the runner's own pause.

## The check that holds it

`US-060 SES-06 once the runner is resumed status shows the same session ready with every machine at the start number it had`
