---
type: Finding
id: m09-3
milestone: M9
title: The server logs Object not found in the reset after a check killed p2
description: In the full run of 2026-11-13, the reset after US-009's check that kills p2 on a dedicated session made the server log Server Object 2:89 not found (message Type_143), the line DEF-09 says the reset no longer causes; it came once in 78 executions and not in a rerun of three kills.
status: draft
raised_by: test
for: developer
checks:
  - "DEF-09 RST-02 the server of the dedicated session logged no Object not found line over the session's resets"
  - "US-009 RST-05 after a check that kills p2 on a dedicated session the next check sees p2 alive at its start, and p2 keeps start 1"
stories: [US-009]
generated: { by: "claude-code/tester", at: 2026-11-13T02:05:00-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-13T05:35:00-08:00, note: "Deferred as DEF-25, outside M9: the body a dead player left is local to its client, which deletes it in the reset (RST-05, DEF-09) while its ragdoll is still awake, and about once in 40 kills an update for it reaches the server after the deletion; reproduced once in 40 kills with CBA, the fixture and arma-grappling-hook (Object 6:49 not found, message Type_467, the body of the check just reset), 0 in 32 with the fixture alone. DEF-09 holds for the objects a check creates; no spec promise changes. The named checks are not run: no framework change." }
verified: []
---

# The server logs Object not found in the reset after a check killed p2

## What the spec promises

`DEF-09` (state `done`, M3), acceptance criterion 2: "Twenty resets on a dedicated session with two clients and a headless client, after checks that create and delete objects on each machine, leave no `Object ... not found` line in the server's RPT." Its summary: "The owner wants the cause found and fixed at its source, so that the line is not logged, rather than concealed." `RST-05`: on a dedicated session "the body it left, which the `EntityRespawned` mission event names on every machine, is deleted by the reset". `US-009` is the story of that reset.

## What the system does

In the full acceptance run of 2026-11-13 against the framework installed from `ba0ae21`, on the `ded` session (a server, two clients and a headless client loading CBA, both fixtures and arma-grappling-hook), one run of 16 checks ran `us009/US-009-kill-p2` as its 13th check and `us009/US-009-p2-alive` as its 14th. Both passed. The server's RPT around the reset between them:

```
 0:59:44 Ragdoll - loading of ragdoll source "Soldier" started.
 0:59:45 [shakedown] r-20261113-085905-19#13 end 2026-11-13T08:59:45.607Z
 0:59:46 [shakedown] r-20261113-085905-19#14 start 2026-11-13T08:59:46.055Z
 0:59:46 Server: Object 2:89 not found (message Type_143)
```

That was the only such line over the 78 check executions of the session. A rerun of the `US-009` check that kills `p2`, three times in one `ded` session, left no such line over its six executions. The full run of 2026-11-12, against `2ab3b28`, had none either. The line is intermittent, and the one seen falls in the reset that deletes a killed client's body. Its message number, `Type_143`, is not the `Type_189` that `DEF-09` names.

## What it costs

Little for now: no check's outcome changed. A person or an AI agent who reads the server's RPT after a failure has to recognise the line as harmless. `DEF-09` was opened because the owner does not want this line logged. It may also mean the reset deletes the body in an order the engine does not expect, which `DEF-09` names as the risk. This may be deferred.

## The check that holds it

`DEF-09 RST-02 the server of the dedicated session logged no Object not found line over the session's resets`, over a session that runs `US-009 RST-05 after a check that kills p2 on a dedicated session ...`. One run in several shows the line.
