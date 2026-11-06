---
type: Index
title: M2 findings
description: The findings of M2 Dedicated sessions.
---

# M2 findings

* [Status lists the clients and the headless client as launched before their processes start](01-status-lists-clients-as-launched-before-their-process-starts.md) - While the server of a dedicated start loads its mission, status lists p1, p2 and hc1 in state launched with no process id and no process start time, for about 17 s, though launched means the process runs.
* [A start past its ceiling of 30 s returns more than a minute late](02-a-start-past-its-ceiling-returns-over-a-minute-late.md) - A dedicated setup with start_timeout = 30 ends failed with the right message, but in four starts of six session start returns after 104 s, and status shows the session starting until then.
