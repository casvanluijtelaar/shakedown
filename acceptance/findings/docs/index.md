---
type: Index
title: Docs findings
description: The findings of checking the docs rewritten after M19's close against the built framework.
---

# Docs findings

* [The quickstart's failure step runs against no session and exits 4](01-the-quickstarts-failure-step-runs-against-no-session.md) - Step 9 of docs/quickstart.md says to run `shakedown run smoke` again after step 7, whose `run --setup dedicated` stopped every game, so the command exits 4 with "No session runs" instead of the failure the page shows.
* [Examples compare ASL positions with a height of 0, where the session's ground is about 5 m ASL](02-examples-compare-asl-positions-with-a-height-of-zero.md) - docs/waits-and-assertions.md's `~=` example compares getPosASL with [100, 104, 0] and fails, and the exec and inspect samples show heights of 0, though on the session mission's default world VR a player stands at about 5.0 m ASL.
* [profileName on a headless client is skd_hc1, not hc1](03-a-headless-clients-profile-name-is-skd-hc1.md) - docs/check-api.md and docs/dedicated-sessions.md show `["all", { profileName }] call skd_fnc_onEach` giving hc1 -> "hc1", but on hc1 profileName is "skd_hc1", in a check and through exec.
* [The ported check in docs/porting-a-suite.md stands p1 4 m from a crate whose action reaches 3 m](04-the-ported-check-stands-p1-outside-the-actions-radius.md) - The rewritten check places the crate at [100, 108] and p1 at [100, 104], so with an Open crate action of radius 3 m, as the fixture and docs/handing-over.md have it, skd_fnc_useAction fails at its ceiling.
* [An unhealthy flag of a machine's earlier start makes every check invalid after a reload restarted its game](05-an-unhealthy-flag-outlives-the-game-restart-of-a-reload.md) - After `shakedown reload` restarted the game of a session broken while p1 was flagged mission-time, every check ended invalid / unhealthy citing that old flag, though status showed the new p1 (start 2) with flag null at 150 frames a second.
* [On a dedicated session p1 starts at spawn plus 10 m in some sessions, in p2's place](06-p1-does-not-always-start-at-the-spawn-point.md) - In two of four fresh dedicated sessions with two clients, p1 started at [110, 100] and p2 at [100, 100], and the reset put each back there, though the session mission puts pN at spawn x plus 10 (N - 1).
* [Sample outputs leave out lines the commands print, and show a session mission name a client does not report](07-sample-outputs-leave-out-lines-the-commands-print.md) - The samples of session start, run --setup and run --each-setup leave out the progress lines the commands print, the quickstart's status sample leaves out the health line, and inspect of a client on a dedicated session reports the mission __cur_mp, not shakedown_<session id>.
* [A slow_step given as the string "10" is refused with a type message, not the one the pages show for it](08-a-slow-step-given-as-a-string-gets-a-type-message.md) - docs/project-file.md and docs/every-setup-and-repeats.md give one message for a slow_step of -1, 0 or "10"; for "10" the build names slow_step and its line, as the contract asks, but in serde's words "invalid type: string "10", expected f64".
