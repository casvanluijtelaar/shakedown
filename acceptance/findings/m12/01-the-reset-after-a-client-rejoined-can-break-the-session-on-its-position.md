---
type: Finding
id: m12-1
milestone: M12
title: The reset after a client was joined again by the restore can break the session on that client's position
description: In one of four runs of M11's dedicated part, the reset before the check after the one in which p2 was joined again by the restore failed with p2 0.60 m from its baseline position, which broke the session and ended every later check of that session.
raised_by: test
for: developer
checks: ["US-067 RST-01 MCH-01 after a check that left p2 out the next check starts with p2 joined again", "US-068", "US-069 after each wrong join or leave"]
stories: [US-067, US-008]
generated: { by: "claude-code/tester", at: 2026-11-16T01:04:58-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-16T01:49:56-08:00, note: "The unit a client left behind stayed at its start position as a body, never deleted, so each rejoin put the new unit into a growing pile of bodies that could push it off its baseline; the server now records that unit on HandleDisconnect and the next reset deletes it (P-06, MCH-01), and a failed reset names the machine that did not verify. The game test us_067_rst_01_the_unit_a_client_left_is_deleted_by_the_next_reset_5_of_5 in crates/shakedown/tests/m11.rs proves it, 20 of 20 checks; the finding's checks passed 18 of 18 with --repeat 3." }
verified:
  - { by: "claude-code/tester", at: 2026-11-16T02:09:36-08:00 }
---

# The reset after a client was joined again by the restore can break the session on that client's position

## What the spec promises

`RST-01`: "Before every check whose reset mode is `in-place`, the framework resets the session as process `P-06` says: ... restores every player's unit (alive, position, direction, ...). The next check sees the baseline state on every machine." `P-06` step 4 has each machine put its own player back on its baseline position "while its own player is more than 2 cm from its baseline position", for up to 1.5 s, before the verification of 0.1 m.

`US-067` criterion 4: "After a check that left `p2` out, the next check starts with `p2` joined again, `ready`, and the reset verified on every machine."

## What the system does

The M11 dedicated part (project `m11` of the harness: a dedicated server with two clients, `late = 1` and a headless client, `@skd_fixture` alone) ran, in one session started at 08:25:16Z on 2026-11-16 with `shakedown 0.1.2`:

1. `run US-067-leave-only after`: `US-067-leave-only` left `p2` out and passed; the restore before `after` joined `p2` again as start 3 and `after` passed (27.4 s, most of it the join). The result file lists `p2` start 2 `stopped` and start 3 `ready`.
2. `run US-068-join-hangs after`, right after it: the reset before `US-068-join-hangs` ran for 15 s and failed. Both checks ended `invalid`:

   - `US-068-join-hangs`: `reset-failed`, machine `server`: "The reset before the check failed: p2: the player is 0.60 m from its baseline position."
   - `after`: `session-broken`.

3. Every later run of the session was refused, exit 4: "Session s-20261116-082516 is broken (the reset before check US-068-join-hangs failed: p2: the player is 0.60 m from its baseline position), so it cannot run checks."

The same part ran three more times that hour, and in the M11 round three times, with no such failure: one in seven runs of the part.

## What it costs

A test author whose suite leaves a client out of a check, or whose check fails before it brings the client back, loses the rest of the run to a broken session now and then, with a message about a position the author never set. In an unattended run (`US-033`) every check after it is `invalid`, and only a game restart brings the session back.

## The check that holds it

`US-067 RST-01 MCH-01 after a check that left p2 out the next check starts with p2 joined again, ready, and the reset verified on every machine`, and the checks of `US-068` and `US-069` that run in the same session after it. They passed in this round's last three runs; the failure shows only in some runs, so a fix round runs them with `--repeat 3` or more. The run folder of the failure is `r-20261116-082808-8` in the work folder of the round.
