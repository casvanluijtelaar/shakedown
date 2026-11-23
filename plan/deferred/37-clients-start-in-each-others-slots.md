---
type: Deferred
title: "Dedicated clients can start in each other's slots"
description: "A dedicated client takes the first free playable slot in the order the clients joined, so in about half of the starts with two clients p1 stands at spawn plus 10 m and p2 at the spawn point, and the reset keeps it; setting each player at its own position from the agent fixed the positions but let two soldiers kill each other in some starts, so it is not built (finding m19-6)."
id: DEF-37
state: open
milestone: M19
status: stable
---

# Dedicated clients can start in each other's slots

## Summary

`INT-AR-04` puts `pN` at `[spawn x + 10 × (N - 1), spawn y]`. With `skipLobby = 1` a client takes the first free playable slot in the order the clients joined (research `14-sessions.md`), and the clients of a dedicated session join in no fixed order, so `p2` can get the slot at the spawn point and `p1` the one 10 m east. The baseline (`RST-02`) then holds the swapped positions, and every reset puts them back there. Finding m19-6 reports it against `US-008`.

## Current behaviour

In the game test `us_008_int_ar_04_every_client_starts_at_its_own_start_position` of `crates/shakedown/tests/m19_findings.rs` (six dedicated starts with two clients, positions read after the start and after a reset), the build at the time of the finding swapped `p1` and `p2` in the fourth start; the test fails there. Over 24 dedicated starts watched while working on the finding, the clients took swapped slots in 11.

## The gap

A check that leans on the documented start positions passes in one session and fails in the next on a correct mod.

## Options

1. **The agent places its own player.** Before the baseline, each client's agent sets its player at `pN`'s position, read from the `respawn_west` marker. Tried: the positions were right in every start in which no soldier died, but where the two clients swap, the soldiers meet on the way and one is killed in some starts (a ragdoll in the server's RPT, then a body, a respawned unit and a dropped weapon that the next reset cannot delete, so the reset fails and the session breaks). Stepping 5 m aside first, waiting for the target spot to be free, and `allowDamage false` while placing each still left a death in some swapped starts: of the 5 swapped starts of the last three test runs, 2 ended this way. The placement also has to wait until the engine has set the unit on the terrain; set earlier, the engine puts the unit back on its slot.
2. **Fix the join order in the runner.** Launch the clients as now, but let `p(N+1)` connect to the server only once `p N` holds a slot, for example by starting each client's process only after the one before reaches client state 10 (`INT-AR-09`). The slots then follow the machine names and no unit moves. Costs start time: the clients' joins run one after another instead of together.
3. **Give each slot to its machine on the server.** Not possible with `skipLobby`: the engine assigns the role, and moving a player to another unit in multiplayer (`selectPlayer`) is not reliable.

## Recommendation

Option 2: it removes the cause instead of moving soldiers that collide. Measure the start time it costs on a dedicated session with two clients and a headless client against `NFR-PF` before choosing it.

## Acceptance criteria

- The game test `us_008_int_ar_04_every_client_starts_at_its_own_start_position` passes, run at least twice, with no `Ragdoll` line in any server RPT of its sessions.
- The dedicated start time stays within its `NFR-PF` ceiling.
- Finding m19-6 gets its `addressed` entry naming the test.
