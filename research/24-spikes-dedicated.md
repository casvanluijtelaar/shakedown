---
type: Research
title: "Spikes SPK-01, SPK-03 and SPK-04 on a dedicated session"
description: "What the round trip through the extension and its callback and the session token through the environment measured on a dedicated server, a game client and a headless client, and what the reset in place measured over 20 checks on a dedicated session with two clients and a headless client, on the test machine in M2, with the engine facts found on the way."
status: stable
generated: { by: "claude-code/developer", at: 2026-11-06T10:45:00+01:00 }
sources:
  - { id: spikes, resource: "crates/shakedown/tests/spikes_m02.rs, spk_01_spk_03_dedicated_server_client_headless, run of 2026-11-06 on the Windows test machine (session s-20261106-091201), Arma 3 2.22.154089 stable", read: 2026-11-06 }
  - { id: twenty, resource: "crates/shakedown/tests/m02.rs, nfr_rl_02_nfr_pf_04_spk_04_twenty_checks_on_a_dedicated_session, and the reset lines of %LOCALAPPDATA%\\Shakedown\\runner.log, runs of 2026-11-06", read: 2026-11-06 }
  - { id: m01, resource: "crates/shakedown/tests/m01.rs, nfr_rl_02_spk_04_twenty_checks_each_see_the_start_state, run of 2026-11-06 with the reset of M2", read: 2026-11-06 }
---

# Spikes SPK-01, SPK-03 and SPK-04 on a dedicated session

M2 ran the server, client and headless client parts of `SPK-01` and `SPK-03` and the dedicated half of `SPK-04` (`spec/06-open-questions.md`) on the Windows test machine, Arma 3 2.22.154089 stable, in a dedicated session the runner started: `arma3server_x64.exe` as `server` with `-autoInit`, then two `arma3_x64.exe` game clients `p1` and `p2` and one headless client `hc1` once the server was ready. The owner was connected through Shadow PC, so the client windows rendered. Each runs again after `python tools/install.py` with `cargo test -p shakedown --test spikes_m02 -- --ignored --nocapture` and, for `SPK-04`, `cargo test -p shakedown --test m02 nfr_rl_02 -- --ignored --nocapture`.

## SPK-01, server, client and headless client: passed

The runner sent `1 + 1` as an `exec` order 100 times in a row to each machine of one session[^spikes].

| Machine | Kind | Answered `2` | Lost | Median | 95th percentile | Max |
|---|---|---|---|---|---|---|
| `server` | dedicated server | 100 | 0 | 42.7 ms | 44.1 ms | 44.6 ms |
| `p1` | game client | 100 | 0 | 11.6 ms | 16.8 ms | 17.4 ms |
| `p2` | game client | 100 | 0 | 13.2 ms | 16.9 ms | 16.9 ms |
| `hc1` | headless client | 100 | 0 | 41.9 ms | 43.2 ms | 43.7 ms |

No RPT line of any machine names a 301 warning; each RPT names the extension once, loaded from `<game>\@shakedown\shakedown_x64.dll` through the junction of `INT-AR-06`[^spikes]. The callback fires on every kind of machine, so no machine kind needs the `remoteExec` fallback. The server and the headless client answer in about 43 ms where a client answers in about 12 ms: a process without a window runs at a lower frame rate, and the agent fetches an order in a scheduled script on the next frame.

**Forces:** `P-03` stands as written for every machine kind; `NFR-PF-03` holds with a wide margin. The part across a mission restart runs in M6.

## SPK-03, server, client and headless client: passed

Each game process the runner started read `SHAKEDOWN_RUNNER`, `SHAKEDOWN_SESSION`, `SHAKEDOWN_MACHINE` and `SHAKEDOWN_TOKEN` from its environment: each extension's hello carried its session id, its machine name and the session token, and the runner accepted it. No process's command line, read through `Win32_Process`, holds an `skd_s_` token[^spikes].

**Forces:** `SEC-02` stands as written for every machine kind. The server's `password` and `serverCommandPassword` are random values of their own, not parts of the session token, because a headless client and a game client take the password on their command line.

## SPK-04, dedicated half: passed

Twenty checks ran in one dedicated session with two clients and a headless client, after a first check that recorded every machine's start state. Each check first asserted that nothing an earlier check created existed by net id on any machine and that every machine showed the start state; then it created a soldier, a quad bike, a box and an MRAP through the API, a container through `createVehicle` on `p2` and a soldier in a new group on `hc1`, hurt `p1`, took its weapons and put it in the MRAP's driver's seat, killed `p2`, and set another date and an overcast of 0.9 with fog and rain. Each machine's own player (position, direction, damage, life, on foot, loadout, animation state), the date, the overcast and the counts of vehicles, units, dead and groups matched the start on every machine, 21 checks of 21[^twenty].

| Run | Resets | Mean | Max | Checks that saw the start state |
|---|---|---|---|---|
| 1 | 21 | 0.318 s | 0.534 s | 13 of 21 |
| 2, with the two fixes below | 21 | 0.687 s | 0.760 s | 21 of 21 |
| 3, the same, on the build M2 ends with | 21 | 0.760 s | 1.503 s | 21 of 21 |

The first reset of a session, with nothing to undo, takes about 0.45 s; a reset after a check that changed everything about 0.75 s, of which 0.3 s is the wait for the players to stand still. Two resets of run 3 took 1.18 s and 1.50 s, the rest under 0.84 s[^twenty].

The first run found two things the reset needed[^twenty]:

- A joining client's baseline was captured while its unit was between two animation states, `amovpercmstpslowwrfldnon_amovpercmstpsraswrfldnon`. The reset put the unit back into that move, which then ran on into `amovpercmstpsraswrfldnon`, so a check that read the state soon after the reset saw the baseline's animation and one that read it later did not. The baseline is captured once the player's animation state is not such a move (its name holds no `_`).
- A unit pulled out of the MRAP on a client stood within 2 cm of its baseline when the verification first read it and then went on moving 0.2 to 0.4 m as its get-out finished, after the reset had reported done. The verification ends once the player has stayed within 2 cm for 0.3 s, setting it back while it is further, for up to 1.5 s.

With the same reset, singleplayer's 20 checks of `NFR-RL-02` all passed again, with resets of 0.40 to 0.93 s, mean 0.73 s[^m01]; the wait for the player to stand still is most of the rise from M1's 0.37 s.

**Forces:** `NFR-PF-04`'s 2 s holds on a dedicated session with two clients and a headless client, with a margin of 0.5 s at the slowest of 63 resets; nothing the reset was asked to restore is left for `@reset mission`, and `SPK-04`'s fallback is not needed. The reset is driven by the runner in phases on every machine at once (`P-06`): the cleanups on the lead; then each machine frees its player; then each machine deletes what its own recorder saw, restores its own player and sets the server's baseline date and weather itself; then each machine verifies, the server also every player's position as it sees it. `RST-02` and `P-06` say so.

## Engine facts found on the way

- **`getUserInfo` on the server** returns, for a game client, `[playerID, ownerId, playerUID, profileName, profileName, steamName, clientState, isHeadless, adminState, ...]`: `p1`'s entry was `["181515371", 5, "<steamid>", "p1", "p1", "<steamname>", 10, false, 0]`. For a headless client the fourth and fifth elements are the name of its `HeadlessClient_F` slot in the mission (`skd_hc1`), not its profile name, and its UID is `HC<process id>`[^spikes]. The server finds a client by the profile name in the fourth element, and a client is ready at state 10 (`INT-AR-09`).
- **`name` of a player's unit** on every machine is the profile name given with `-name`, so the lead finds the unit of `pN` among `allPlayers` by its name (`DM-11`)[^twenty].
- **A dedicated start** of a server, two clients and a headless client took 43 to 45 s from the first launch to `ready`: the server about 15 s to its mission, then the clients and the headless client together about 28 s[^spikes][^twenty]. A `start_timeout` of 30 s cannot hold such a start.
- **Memory**: with the server, two game clients and the headless client up, the machine had 12,287 MB of its 16 GB in use, 6,822 MB of it in the four game processes[^spikes].
- **A respawned player** on a dedicated session with `respawn = 3` and `respawnDelay = 0` is a new unit at the `respawn_west` marker at once; the `EntityRespawned` mission event gives the new and the old unit, and the old one is the body the reset deletes (`RST-05`)[^twenty].

[^spikes]: crates/shakedown/tests/spikes_m02.rs, spk_01_spk_03_dedicated_server_client_headless, run of 2026-11-06 on the Windows test machine (session s-20261106-091201), Arma 3 2.22.154089 stable
[^twenty]: crates/shakedown/tests/m02.rs, nfr_rl_02_nfr_pf_04_spk_04_twenty_checks_on_a_dedicated_session, and the reset lines of %LOCALAPPDATA%\Shakedown\runner.log, runs of 2026-11-06
[^m01]: crates/shakedown/tests/m01.rs, nfr_rl_02_spk_04_twenty_checks_each_see_the_start_state, run of 2026-11-06 with the reset of M2
