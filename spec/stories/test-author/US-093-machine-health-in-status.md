---
type: User Story
title: "See each machine's frame rate and answer time"
description: "status and inspect show each machine's frame rate and the time it takes to answer, beside its clocks, so a person or an AI agent sees a struggling machine before it is lost."
status: draft
id: US-093
role: test-author
kind: feature
requirements: [HLT-01, INS-04, INS-01]
---

# US-093 See each machine's frame rate and answer time

As a test author, I want each machine's frame rate and answer time at a glance, so that I see a slow environment for what it is.

## Acceptance criteria

1. `status` lists, for each machine of a dedicated session with two clients, its frame rate over the last 5 s and its last answer time beside the clocks of its last heartbeat.
2. `status` through MCP gives the same fields.

## Next to

- [US-094](US-094-frame-rate-collapses.md) (system fail): a machine's frame rate collapses during a check.
