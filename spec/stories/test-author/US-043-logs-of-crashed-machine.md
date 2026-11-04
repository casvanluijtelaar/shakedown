---
type: User Story
title: "The logs of a machine that crashed are kept"
description: "When a machine's process exits during a run, its log lines up to the exit and its crash dump are kept with the run."
status: draft
id: US-043
role: test-author
kind: system fail
requirements: [LOG-01, LOG-04, P-09]
---

# US-043 The logs of a machine that crashed are kept

As a test author, I want to still have the log and the crash dump of a machine that crashed, so that I can report the crash without reproducing it.

## Acceptance criteria

1. After `p2`'s process is ended during a check, the run folder holds `p2`'s log excerpt for that check up to its last line, and `evidence.logs` points to it.
2. Any `.mdmp` or `.bidmp` the crash left in `p2`'s profile folder is copied into the run folder, and the result file's `lost.dumps` for `p2` gives its path.

## Next to

- [US-042](US-042-logs-with-run.md) (feature): each machine's log is kept.
