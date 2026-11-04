---
type: User Story
title: "Each machine's log is kept with the run and can be filtered"
description: "Each machine's log is kept with the run and per check, without Steam IDs, and a test author reads a machine's lines filtered by run, check, pattern and count."
status: stable
id: US-042
role: test-author
kind: feature
requirements: [LOG-01, LOG-02, LOG-03, LOG-04, SES-09, NFR-PV-01, INS-04]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-042 Each machine's log is kept with the run and can be filtered

As a test author, I want to read the lines each machine logged during one check, without hunting through whole log files, so that the evidence of a failure is one command away.

## Acceptance criteria

1. `shakedown logs --machine p2 --check <check id> --run <run id>` prints only `p2`'s lines between that check's start and end markers, and `--grep` and `--last` narrow them; the MCP tool `logs` returns the same lines.
2. No file in the run folder, no `logs` output and no stopped session's RPT holds a 17-digit number starting with `7656119`; such numbers read `<steamid>`.
3. After `session stop`, each machine start's RPT is still in its own profile folder in the session folder.

## Next to

- [US-043](US-043-logs-of-crashed-machine.md) (system fail): the machine whose logs are needed crashed.
