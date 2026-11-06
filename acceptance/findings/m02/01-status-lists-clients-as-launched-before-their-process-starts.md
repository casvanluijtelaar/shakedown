---
type: Finding
id: m02-1
milestone: M2
title: Status lists the clients and the headless client as launched before their processes start
description: While the server of a dedicated start loads its mission, status lists p1, p2 and hc1 in state launched with no process id and no process start time, for about 17 s, though launched means the process runs.
raised_by: test
for: developer
checks: ["US-004 SES-04 a client ended during the start fails it with exit 4 naming p2, its state and its last RPT lines, leaving nothing running", "INS-04 m02-1 status during a dedicated start lists a machine as launched only with its process id and process start time"]
stories: [US-002, US-004]
generated: { by: "claude-code/tester", at: 2026-11-06T02:41:24-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-06T04:37:47-08:00, note: "status and every session shape list a machine only once its process is launched, with its process id and start time, so the server is listed alone while it loads its mission (INS-04 made explicit in spec/03 and docs/dedicated-sessions.md). The e2e test us_004_a_start_that_never_becomes_ready_ends_failed_and_leaves_nothing in crates/shakedown/tests/m02.rs proves it on a dedicated start; both named checks pass." }
verified:
  - { by: "claude-code/tester", at: 2026-11-06T04:59:30-08:00 }
---

# Status lists the clients and the headless client as launched before their processes start

## What the spec promises

`INS-04`: "`status`, and every session shape the command line and the tools return, list each machine from the moment its process is launched, in every session state including `starting` and `reloading`, with: its state and start number; its operating-system process id and process start time; its profile folder; ...". The machine status machine in `spec/01-domain-model.md` says `launched` is "The process runs; its extension has not connected." `docs/dedicated-sessions.md` says `status` "lists each machine from the moment its process is launched".

## What the system does

`shakedown session start --setup ded --json` in the background, for a setup with `clients = 2` and `headless = 1`, and `shakedown status --json` five times a second beside it. Each line is a change in what `status` gives, as seconds since the start, then `(name, state, pid, process_started_at given)` per machine:

```
 0.59 starting: (server, launched, 12500, true) (p1, launched, null, false) (p2, launched, null, false) (hc1, launched, null, false)
18.01 starting: (server, ready, 12500, true) (p1, launched, 10988, true) (p2, launched, 15552, true) (hc1, launched, 16208, true)
...
49.13 starting: every machine ready
```

For the 17 s the server loads its mission, `p1`, `p2` and `hc1` are listed as `launched` with `pid` and `process_started_at` `null`. The `session start` output shows their processes start only once the server is ready ("started p1 (process 10988)" after "started server"). The run of the check printed one such entry as `{'name': 'p1', 'state': 'launched', 'pid': None, 'process_started_at': None, ...}`.

## What it costs

A test author or an AI agent watching a slow or stuck start reads that every client is already running, when none has been started and the start is waiting on the server. A tool that ends or suspends a chosen machine by its process id, as `INS-04` and `fixture-mod.md` intend, finds no id for a machine `status` says runs. The state and the id disagree, so neither can be trusted in the first part of every dedicated start.

## The check that holds it

`INS-04 m02-1 status during a dedicated start lists a machine as launched only with its process id and process start time`. It reads the `status` answers the check `US-004 SES-04 a client ended during the start fails it with exit 4 naming p2, its state and its last RPT lines, leaving nothing running` records while it watches its start, so both are run together: `--check US-004 --check "INS-04 m02-1"`.
