---
type: Index
okf_version: "0.2"
title: Acceptance coverage
description: The result of the last full acceptance run of each milestone, one Coverage concept per milestone.
---

# Acceptance coverage

* [M0 Walking skeleton: coverage](m00.md) - The last full acceptance run of M0: every check of US-037 and US-038, and of the demo's requirements, passed three times in one singleplayer session.
* [M1 A grappling-hook check in singleplayer: coverage](m01.md) - The last full acceptance run of M1: all 65 checks passed, every acceptance criterion of US-018, US-021, US-022 and US-039, the demo ten times in one run with a second check, 20 resets in one session, and the checks of findings m01-1, m01-2 and m01-3.
* [M2 Dedicated sessions: coverage](m02.md) - The last full acceptance run of M2: all 266 check executions passed, every check of the nine M2 stories three times in one dedicated session, the demo 30 of 30, both failing starts of US-004 in time, and every M0 and M1 check beside them.
* [M3 Script errors and logs: coverage](m03.md) - The last full acceptance run of M3: all 117 checks passed, every criterion of the seven M3 stories on a dedicated session with two clients and a headless client, the demo, no Steam identity in anything the runs left, and every script error the RPTs showed inside a check execution in its result file.
* [M4 Agent tools: coverage](m04.md) - The last full acceptance run of M4: all 171 checks passed, every story of M4 driven through the command line and through the shakedown MCP server over its protocol, NFR-PF-03 on every machine kind, US-003's Steam criterion with Steam stopped, and US-046 without its Mac criterion.
* [M5 A session that outlives its client: coverage](m05.md) - The last full acceptance run before M5 closes: 190 of 191 checks of every milestone passed; the one failure is US-052's runner.json ACL (m05-1). In the M5 rounds every other M5 check passed 3 of 3, the runner was killed 20 times during a run on a dedicated session with the game gone within 0.9 s, and US-007 passed 3 of 3 under headless Claude Code.
* [M6 Reload by restart: coverage](m06.md) - The full acceptance run before M6 closes: 207 of 207 checks of every milestone passed, with the 16 M6 checks among them; in the M6 round every M6 check passed 3 of 3 in one dedicated session. An earlier full run lost its dedicated session to a reset that failed once on p2's direction (m06-1, open).
* [M7 Reload inside the mission: coverage](m07.md) - The full acceptance run after the fix of m07-1: 227 of 227 checks of every milestone passed, the 20 M7 checks among them; m07-1 is verified, its two checks 3 of 3 on a dedicated session with two clients and in singleplayer.
* [M8 Walk away: coverage](m08.md) - The full acceptance run before M8 closes: 243 of 243 checks of every milestone passed, the 16 M8 checks among them; in the M8 round every M8 check passed 3 of 3, with no finding.
* [M9 Invalid, not failed: coverage](m09.md) - In the M9 round every M9 check passed 3 of 3 but one, US-044's inspect of a client naming the step (m09-1); the full acceptance run passed 275 of 277, the other failure US-060's resumed runner, which since M9 loses every machine (m09-2).
