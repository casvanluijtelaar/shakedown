---
type: User Story
title: "A run reaches its runner ceiling"
description: "A run that outlasts the sum of its checks' timeouts plus 60 s is stopped at that ceiling, reports the check it was on as invalid and the rest as not run, writes its result file, and leaves nothing of its own running."
status: stable
id: US-063
role: test-author
kind: system fail
requirements: [RUN-06, RUN-05, RES-01, STP-02, DM-05]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# US-063 A run reaches its runner ceiling

As a test author, I want a run that takes far longer than its checks allow to be stopped and reported, so that a suite I walked away from ends with a result instead of hanging.

## Acceptance criteria

1. A run of eight checks, each with `// @timeout 1` and each registering with `skd_fnc_cleanup` a cleanup that runs for 14 s of real time, has a runner ceiling of 68 s and stops between 68 s and 80 s after its first check started: the check it was preparing or executing ends `invalid` / `run-stopped`, every check not started is in `not_run` with reason `run-stopped`, `result.json` is valid against `result.schema.json` with `run.state` `stopped` and `run.stopped_reason` `run-ceiling`, and the run exits 2.
2. With `--setup sp` and no session running, no `arma3_x64` process runs after the run ends and `status` shows no session; against a running session, that session is `ready` afterwards and every machine keeps its start number.
3. Through MCP, `run_start` with the same selection is followed by `run_status` returning state `stopped`, and `run_result` gives the same summary.

## Next to

- [US-033](US-033-full-suite-walk-away.md) (feature): run the full suite and walk away.
- [US-034](US-034-run-stopped-by-hand.md) (system fail): the run is stopped by hand.
