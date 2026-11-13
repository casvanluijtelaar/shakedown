---
type: Index
title: M9 findings
description: The findings of M9 Invalid, not failed.
---

# M9 findings

* [Inspect of a client that runs the wait of a check's step names the check but no step](01-inspect-of-a-client-running-a-steps-order-names-no-step.md) - While a check's step waits on p1, shakedown inspect --machine p1 names the run and the check but gives step_path [], while inspect of the lead names the step; INS-01 and US-044 promise the step running.
* [A runner suspended for some seconds and resumed marks every machine lost and the session broken](02-a-suspended-runner-loses-every-machine-when-resumed.md) - With the runner's process suspended for about 25 s and then resumed, status shows the session broken and every machine lost for "no heartbeat came for 26 s", where US-060 promises the same session ready with every machine at its start number.
* [The server logs Object not found in the reset after a check killed p2](03-the-server-logs-object-not-found-in-the-reset-after-a-client-was-killed.md) - In the full run of 2026-11-13, the reset after US-009's check that kills p2 on a dedicated session made the server log Server Object 2:89 not found (message Type_143), the line DEF-09 says the reset no longer causes; it came once in 78 executions and not in a rerun of three kills.
