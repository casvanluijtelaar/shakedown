---
type: Finding
id: m02-2
milestone: M2
title: A start past its ceiling of 30 s returns more than a minute late
description: A dedicated setup with start_timeout = 30 ends failed with the right message, but in four starts of six session start returns after 104 s, and status shows the session starting until then.
raised_by: test
for: developer
checks: ["US-004 SES-04 a setup with start_timeout 30 fails at about 30 s with exit 4, naming each machine not ready and its state"]
stories: [US-004]
generated: { by: "claude-code/tester", at: 2026-11-06T02:54:30-08:00 }
addressed:
  - { by: "claude-code/tester", at: 2026-11-06T04:59:00-08:00, note: "A mistake in the check: it polled status without reading the background session start's stdout and stderr, so the failure output, over the 4 KB a Windows pipe holds, kept the command from exiting until the check stopped polling. The harness reads both while it polls (acceptance/docs/harness.md); a start with pipes read returned at 31.2 s, status gave no session and no game process ran from 31.6 s, and the named check passed 3 of 3 at 31.4 to 31.5 s." }
verified:
  - { by: "claude-code/tester", at: 2026-11-06T04:59:30-08:00 }
---

# A start past its ceiling of 30 s returns more than a minute late

## What the spec promises

`US-004`: "As a test author, I want to have a start that hangs or loses a process end at a ceiling, with the machine that was not ready and its last log lines, so that a stuck start never holds the machine and I know where to look." Criterion 2: "A setup with `start_timeout = 30` whose session is not `ready` within 30 s returns `failed` with a reason that names each machine that was not `ready` and its state." `SES-04`: "A start has a ceiling of 180 s from the first process launch to `ready`, settable per setup with `start_timeout`. At the ceiling ... the session is `failed`, every process it started is stopped".

## What the system does

A setup `ded30`: `mode = "dedicated"`, `clients = 2`, `headless = 1`, `start_timeout = 30`, mods CBA, both fixtures and arma-grappling-hook. A start of this setup with a server, two clients and a headless client is ready in about 46 s when it has no ceiling, so it cannot meet 30 s.

`shakedown session start --setup ded30 --json`, six times, each from nothing running, with `shakedown status --json` twice a second beside it:

| Start | `session start` returned after | exit |
|---|---|---|
| 1 | 31.1 s | 4 |
| 2 | 103.9 s | 4 |
| 3 | 103.9 s | 4 |
| 4 | 31.3 s | 4 |
| 5 | 103.6 s | 4 |
| 6 | 103.6 s | 4 |

Every one ends `failed` with the message `the session was not ready within the start ceiling of 30 s: p1 was launched; p2 was launched; hc1 was launched`, and `reason.machine` `p1`. In the slow ones `reason.rpt_tail` holds lines stamped about 27 s after the start (2:52:03 for a start at 2:51:36), so the ceiling was seen in time; yet `status` keeps the session `starting`, the server `ready` and the other three `launched`, from 18 s into the start until `session start` returns at 104 s. No game process of the start runs after it returns.

## What it costs

The test author set a ceiling of 30 s and waits 104 s, more than three times as long, with `status` saying all that time that the start is still going. A stuck start holds the machine, its memory and its game ports for over a minute past the ceiling the author chose, which is what the story's ceiling is there to prevent; an agent that polls `status` cannot tell a failed start from a slow one until it returns.

## The check that holds it

`US-004 SES-04 a setup with start_timeout 30 fails at about 30 s with exit 4, naming each machine not ready and its state`. It allows the start up to 75 s.
