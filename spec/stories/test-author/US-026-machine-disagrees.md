---
type: User Story
title: "One machine disagrees with the others"
description: "When one machine's value differs from the others, the check fails naming that machine and showing every machine's value."
status: stable
id: US-026
role: test-author
kind: error
requirements: [OBS-06, OBS-04, RES-03]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-026 One machine disagrees with the others

As a test author, I want to see which machine disagreed and what each one saw, so that I know whether the server or one client is wrong.

## Acceptance criteria

1. With `skdfx_flag` set to `5` on the server and to `6` on `p2` alone without broadcasting, `skd_fnc_expectAgree` on `["server", "p1", "p2"]` ends `fail` / `assertion` with `reason.machine` `p2` and a message saying `p2` read `6` where `server` read `5`.
2. The result file's `evidence.observed` holds `server`: 5, `p1`: 5, `p2`: 6.

## Next to

- [US-025](US-025-machines-agree.md) (feature): every machine agrees.
