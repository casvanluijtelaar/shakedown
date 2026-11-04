---
type: Research
title: "Sessions: starting, keeping and resetting the game"
description: How singleplayer, a dedicated server, clients and headless clients start from the command line, how many checks can run in one session and how its state is reset, what joining, leaving and mission changes do, and how fast each path is.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T21:52:33+01:00 }
sources:
  - { id: startup, resource: "https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters (oldid 378577)", read: 2026-11-04 }
  - { id: servercfg, resource: "https://community.bistudio.com/wiki/Arma_3:_Server_Config_File (oldid 379176)", read: 2026-11-04 }
  - { id: srvcmds, resource: "https://community.bistudio.com/wiki/Multiplayer_Server_Commands (oldid 378924)", read: 2026-11-04 }
  - { id: servercommand, resource: "https://community.bistudio.com/wiki/serverCommand", read: 2026-11-04 }
  - { id: hc, resource: "https://community.bistudio.com/wiki/Arma_3:_Headless_Client (oldid 352522)", read: 2026-11-04 }
  - { id: ds, resource: "https://community.bistudio.com/wiki/Arma_3:_Dedicated_Server (oldid 378682)", read: 2026-11-04 }
  - { id: mps, resource: "https://community.bistudio.com/wiki/Multiplayer_Scripting, section on testing on one machine", read: 2026-11-04 }
  - { id: time, resource: "https://community.bistudio.com/wiki/time (oldid 379479)", read: 2026-11-04 }
  - { id: meh, resource: "https://community.bistudio.com/wiki/Arma_3:_Mission_Event_Handlers (oldid 379196)", read: 2026-11-04 }
  - { id: amo, resource: "https://community.bistudio.com/wiki/allMissionObjects", read: 2026-11-04 }
  - { id: cstate, resource: "https://community.bistudio.com/wiki/getClientStateNumber (oldid 348456)", read: 2026-11-04 }
  - { id: t1, resource: "research/10-prior-harness.md", read: 2026-11-04 }
  - { id: hdoc, resource: "arma-grappling-hook@3f4f7bd:docs/harness.md", read: 2026-11-04 }
  - { id: hrun, resource: "arma-grappling-hook@3f4f7bd:harness/run.ps1", read: 2026-11-04 }
  - { id: r14, resource: "arma-grappling-hook@3f4f7bd:research/14-test-automation.md", read: 2026-11-04 }
  - { id: tjip, resource: "arma-grappling-hook@3f4f7bd:research/tests/mp-jip-state/README.md", read: 2026-11-04 }
  - { id: tdc, resource: "arma-grappling-hook@3f4f7bd:research/tests/mp-disconnect/README.md", read: 2026-11-04 }
  - { id: tsmoke, resource: "arma-grappling-hook@3f4f7bd:research/tests/harness-smoke/README.md", read: 2026-11-04 }
  - { id: amiss, resource: "arma-grappling-hook@3f4f7bd:acceptance/docs/m00-missions.md", read: 2026-11-04 }
  - { id: f004, resource: "arma-grappling-hook@3f4f7bd:acceptance/findings/m00/004-a-let-go-8-m-up-now-and-then-does-more-leg-damage-than-the-game-s-own-fall-from-there.md", read: 2026-11-04 }
  - { id: prd, resource: "spec/client-requirements.md", read: 2026-11-04 }
---

# Sessions: starting, keeping and resetting the game

A session is the set of game processes a run keeps up while many checks run against it: one singleplayer game, or a dedicated server with game clients and headless clients (`CR-01`)[^prd]. This topic covers how each process starts, how state is reset between checks (`CR-02`), what joining, leaving and mission changes do (`CR-04`), how several clients share one machine, and what each path costs. Commands and parameters are checked on the Bohemia Interactive Community Wiki for game 2.22, the current stable branch; none needs the profiling or development branch. Timings come from `arma-grappling-hook`'s harness on the test machine[^t1]; any other timing is marked **needs a spike**.

## 1. Starting each process

| Process | Executable and the parameters that matter |
|---|---|
| Singleplayer | `arma3_x64.exe -autotest=<cfg>`: runs a list of missions "in special mode", logs a mission that does not end in `END1` as an `<AutoTest result="FAILED">` block and returns an error level[^startup]. Mission paths are relative to the game folder, or to the `-profiles` folder when one is given[^startup]. |
| Dedicated server | `arma3server_x64.exe -port=<p> -config=<server.cfg> -profiles=<dir> -name=<n> -mod=<mods> -autoInit`[^startup][^hrun]. |
| Game client | `arma3_x64.exe -connect=<ip> -port=<p> -password=<pw> -profiles=<dir> -name=<n> -mod=<mods>`[^startup][^hrun]. |
| Headless client | `arma3server_x64.exe -client -connect=<ip> -password=<pw>`; it is placed in a free headless-client slot automatically[^hc]. |
| Player-hosted | `arma3_x64.exe -host` starts a non-dedicated host[^startup]; how to load its mission with nobody in the lobby is not known from any source read **(unverified)**[^r14]. |

What each start option does:

| Option | Effect | Source |
|---|---|---|
| `-autotest=<cfg>` | Runs the listed missions one after another and exits; on the test machine exit code 0 when all passed, 1 otherwise, and the next mission starts about 1 s after the last ended | [^startup][^hdoc] |
| `-autoInit` | The server initialises the mission as the first client would; ignored unless `server.cfg` has `persistent = 1`; mission parameters then return only their defaults | [^startup] |
| `-filePatching` | Lets the game load unpacked data; a server accepts such clients only with `allowedFilePatching` (1 headless clients, 2 all clients) | [^startup][^servercfg] |
| `-world=empty` | No default world and no main menu intro at start, for faster loading | [^startup] |
| `-noPause`, `-noPauseAudio` | Keeps the game, and its audio, running while its window has no focus | [^startup] |
| `-profiles=<dir>`, `-name=<n>` | The profile folder (logs, caches and crash files go there) and the profile name | [^startup] |
| `-skipIntro`, `-noSplash` | Skip intros and splash screens; the wiki says the gain is likely negligible | [^startup] |
| `-showScriptErrors` | Shows script errors on screen | [^startup] |
| `-noSound` | Disables sound output | [^startup] |
| `-pid=<file>` | The dedicated server writes its process id to the file and removes it when it stops | [^startup] |
| `-preprocDefine=<NAME>` | Defines a macro, prefixed `CMD__`, for the whole game (2.14 and later) | [^startup] |
| `-limitFPS=<n>` | FPS limit of a server or headless client, default 50; since 2.20 a server with no players limits itself to 30, and the limit also works on clients | [^startup] |
| `-init=<command>` | Runs a command once in the main menu, such as `playMission`; the mission must be in the game's `Missions` folder | [^startup] |
| `-command=<name>` | A named pipe that takes a few commands (`shutdown`, `session`, `connect`); clients only, not a dedicated server | [^startup] |

The harness measured `-enableHT`, `-hugePages`, `-cpuCount`/`-exThreads`, a smaller window, `-loadMissionToMemory` and dropping `-filePatching`; none moved a start by more than 1 s[^hdoc].

A mission must place clients with no person in the lobby: `skipLobby = 1` gives a joining player the first free role[^r14], and a headless client needs a playable `HeadlessClient_F` slot, one slot per unit[^hc][^tsmoke]. A client takes the first free slot, so a check finds a machine by profile name, not by slot[^hdoc].

## 2. Many checks in one session

There are three ways to give a check a clean start, from cheapest to dearest:

1. **Reset in place.** The mission keeps running; the framework undoes what the check changed. The engine offers the pieces: the `EntityCreated` mission event handler (2.10 and later) and `EntityDeleted` (2.18) to record what a check created[^meh], and `allMissionObjects` to find every object of a type, which the wiki calls "quite performance heavy"[^amo]. No engine command resets a mission's state as a whole **(inference)**: time, weather, player position and damage, variables, event handlers and the mod's own state are each reset by their own commands, and a mod's internal state can only be reset through the mod's public functions. So a reset in place is a contract between the framework and the check: the framework resets what it can name (created entities, players, time, weather, its own handlers), and a check that changes anything else says how to undo it or asks for a fresh mission.
2. **A fresh mission in the same processes.** Singleplayer moves to the next `-autotest` mission in about 1 s[^hdoc]. A dedicated server loads another mission with `#mission <name>` or restarts the current one with `#restart`, and `#reassign` starts over with role assignment[^srvcmds]. A mission's own script can send these with `"<serverCommandPassword>" serverCommand "#..."`; `true` does not mean the command succeeded[^servercommand]. `#missions` triggers `missionEnd`, which can restart or shut down a server that has `missionsToServerRestart` or `missionsToShutdown` set[^srvcmds].
3. **A fresh session.** A new start of every process: 18 to 20 s in singleplayer, about 25 s before the last of a server and two clients is in[^hdoc].

The `-autotest` list fixes the missions at start, and the game exits after the last one[^startup][^hdoc], so a singleplayer session that takes new checks while it runs cannot be an `-autotest` list of the checks themselves **(inference)**: it is one long mission that runs checks sent to it (`12-remote-execution.md`), with `-autotest` only as the way to start it without a person. `-autotest` also ends the mission when the player dies (`EndMode = KILLED`), which the prior harness had to work around[^amiss][^f004].

## 3. Leaving, joining and mission changes

- **Join in progress.** With `-autoInit` the server starts the mission before anyone is in, so every client joins in progress[^tjip]. A joining client's `time` starts at 0 and jumps to the server's after about 2.5 s on average[^time]. Without `-autoInit` the server waits in the lobby, and on the test machine mission time stayed at 0 for about a minute after the mission started[^tjip].
- **Leaving.** A clean leave fires `PlayerDisconnected` and `HandleDisconnect` within 0.11 s, but `OnUserDisconnected` only when the process ends; a crash fires all three about 15.6 s after the process ended[^tdc]. `HandleDisconnect` fires only on the server and not for a player who controlled no unit[^meh]. The server's `DisconnectTimeout` defaults to 15 s, range 1 to 90[^servercfg], which matches the 15 s the harness saw an ended process stay listed[^hdoc].
- **Rejoining.** A client that left could not get back into a mission with `disabledAI = 1` and no respawn; with base respawn it rejoined in under 20 s[^r14][^tdc]. Objects a client created stay, owned by the server, after it leaves[^tdc].
- **Mission change with clients connected.** In about one change in two, the engine left a client in a mission it never loaded, or a headless client without a role; the harness restarts every client for each mission[^hdoc]. A client kicked while it runs comes back at once, with script errors; a machine still listed when the next mission loads keeps its slot[^hdoc]. A server that waited in the lobby went back to the first mission of its cycle once clients joined[^hdoc].
- **Client states.** `getClientStateNumber` goes from 0 `NONE` through 6 `ROLE ASSIGNED`, 8 `GAME LOADED` and 10 `BRIEFING READ` (ready to play) to 12[^cstate], and the server sees each user's state through `getUserInfo` and the `OnUserClientStateChanged` event (2.06)[^meh]. A join is complete for the framework at state 10 and once the client answers on its channel **(inference)**.
- **Server restart.** `#restartserver` shuts down and restarts the server process; `#shutdown` stops it[^srvcmds].

## 4. Several clients on one machine

- `loopback = 1` forces a LAN server that accepts local instances only, and `kickDuplicate = 0` lets several instances with one game ID join[^servercfg][^mps]. BattlEye must be off, or it closes every additional instance[^mps].
- Every instance runs on the one Steam account, so all carry one Steam UID[^tdc]. A kick blocked all of them until `kickTimeout[]` was set to 0 for every kind of kick (the default is 60 s each)[^hdoc][^servercfg].
- Headless clients need `headlessClients[]` and `localClient[]` with `127.0.0.1`; any number can run on one machine[^hc][^servercfg].
- A server uses its game port and the next four (Steam query +1, Steam +2, VON +3, BattlEye +4); a second server on one machine should sit at least 100 ports higher[^ds]. The harness uses 2402[^hrun].
- Each instance gets its own `-profiles` folder and `-name`, so each writes its own RPT[^r14][^hrun].
- Memory: the test machine has 16 GB; a server and two full game clients fit, more players are headless clients[^r14]. Per-instance memory has not been measured **(needs a spike)**.

## 5. How fast each path is

| Path | Singleplayer | Dedicated, 2 clients and 1 headless client |
|---|---|---|
| Full start to the first mission line | 18 to 20 s[^hdoc] | server 13 s, headless client 17 to 18 s, clients 24 to 26 s, started together[^hdoc] |
| Next mission, same processes | about 1 s[^hdoc] | about 2 s; fresh clients then about 25 s[^hdoc] |
| Client leave seen by the server | | clean leave 0.1 s; crash about 15.6 s[^tdc] |
| Reset in place | **needs a spike** | **needs a spike** |
| `#restart` with clients kept | | **needs a spike** |

## 6. Findings and what they force

1. **One long mission per session, with checks sent into it, is the only path that keeps a session across many checks and new checks.** `-autotest` fixes its list at start and exits after it[^startup]. Forces a session mission that the framework generates and keeps running, checks delivered to it at run time, and `-autotest` used only to start singleplayer unattended. Touches `CR-01`, `CR-03`, `CR-40`.
2. **No engine command resets a mission.** Forces a reset contract: the framework records created entities with `EntityCreated`[^meh] and deletes them, restores players, time and weather, and removes its own handlers; whatever else a check changes it undoes itself, or the check asks for a fresh mission. Touches `CR-02`, `CR-50`.
3. **A mission change under connected clients broke about one time in two.** Forces a fresh mission on a dedicated server to restart its clients, which costs about 25 s[^hdoc], and makes reset in place the default. Touches `CR-01`, `CR-02`, `CR-41`.
4. **`-autotest` ends the mission when the player dies.** Forces the framework to prevent or catch that end (for example with respawn in the session mission, or a death check that reports instead of ending) so a death is a check's event, not the session's end. Touches `CR-02`, `CR-14`[^amiss].
5. **A leave is seen at once, a crash after the disconnect timeout.** Forces the framework to wait on the server's own events and the client state, with a ceiling set from `DisconnectTimeout`, which it may lower in its `server.cfg`[^servercfg][^tdc]. Touches `CR-04`, `CR-14`, `CR-20`.
6. **Every instance shares one Steam ID and one machine.** Forces `loopback`, `kickDuplicate = 0`, `kickTimeout[]` of 0, BattlEye off, a port block per server, a profile folder per instance, and machines named by profile name. Touches `CR-01`, `CR-61`[^servercfg][^mps][^ds].
7. **Joining in progress resets `time` for a few seconds, and `-NoAutoInit` held mission time at 0 for about a minute.** Forces waits on a joining client in real time or after its state reaches 10, never on `time` alone. Touches `CR-04`, `CR-20`[^time][^tjip].
8. **The player-hosted setup has no known unattended start.** Forces it out of the MVP, which needs singleplayer and dedicated only[^prd]. Touches `CR-41`.

## Needs a spike

- **Reset in place.** Pass condition: 20 checks in one singleplayer session and 20 in one dedicated session (2 clients, 1 headless client), each creating units, vehicles and objects and changing time, weather and the player; after each reset, a probe finds no object created by an earlier check, the players at their start state, and time and weather as at mission start, on every machine; the reset takes under 2 s.
- **`#restart` and `#mission` with clients connected.** Pass condition: 20 changes each, with 2 clients and 1 headless client kept connected; every machine reaches client state 10 in the new mission and answers within 30 s, in 20 of 20. If not, the framework restarts clients on every change.
- **Lower `DisconnectTimeout`.** Pass condition: with `disconnectTimeout = 3`, a killed client's `HandleDisconnect` fires within 5 s in 10 of 10, and no healthy client is dropped over a 30-minute session.
- **Memory per instance.** Pass condition: the working set of the server, each game client and each headless client is logged over a 30-minute session with the target mods, and the most clients and headless clients that fit in 16 GB with 2 GB left free is named.
- **One long session.** Pass condition: a singleplayer and a dedicated session each stay up for 2 hours running checks every 30 s, with no rise in frame time over 20 percent and no growth in memory over 1 GB.

[^startup]: https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters (oldid 378577)
[^servercfg]: https://community.bistudio.com/wiki/Arma_3:_Server_Config_File (oldid 379176)
[^srvcmds]: https://community.bistudio.com/wiki/Multiplayer_Server_Commands (oldid 378924)
[^servercommand]: https://community.bistudio.com/wiki/serverCommand
[^hc]: https://community.bistudio.com/wiki/Arma_3:_Headless_Client (oldid 352522)
[^ds]: https://community.bistudio.com/wiki/Arma_3:_Dedicated_Server (oldid 378682)
[^mps]: https://community.bistudio.com/wiki/Multiplayer_Scripting, section on testing on one machine
[^time]: https://community.bistudio.com/wiki/time (oldid 379479)
[^meh]: https://community.bistudio.com/wiki/Arma_3:_Mission_Event_Handlers (oldid 379196)
[^amo]: https://community.bistudio.com/wiki/allMissionObjects
[^cstate]: https://community.bistudio.com/wiki/getClientStateNumber (oldid 348456)
[^t1]: research/10-prior-harness.md
[^hdoc]: arma-grappling-hook@3f4f7bd:docs/harness.md
[^hrun]: arma-grappling-hook@3f4f7bd:harness/run.ps1
[^r14]: arma-grappling-hook@3f4f7bd:research/14-test-automation.md
[^tjip]: arma-grappling-hook@3f4f7bd:research/tests/mp-jip-state/README.md
[^tdc]: arma-grappling-hook@3f4f7bd:research/tests/mp-disconnect/README.md
[^tsmoke]: arma-grappling-hook@3f4f7bd:research/tests/harness-smoke/README.md
[^amiss]: arma-grappling-hook@3f4f7bd:acceptance/docs/m00-missions.md
[^f004]: arma-grappling-hook@3f4f7bd:acceptance/findings/m00/004-a-let-go-8-m-up-now-and-then-does-more-leg-damage-than-the-game-s-own-fall-from-there.md
[^prd]: spec/client-requirements.md
