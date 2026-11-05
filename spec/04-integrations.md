---
type: Specification
title: Integrations
description: How Shakedown works with Arma 3, CBA, ACE3, HEMTT, Steam, Windows and Claude Code - the versions it is specified against, the interfaces it uses and the configuration it generates.
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# Integrations

Each integration rule has an ID of the form `INT-<system>-nn`. Versions were looked up on 2026-11-04; the tech stack ([07-tech-stack.md](07-tech-stack.md)) lists them with their sources. A version is looked up again at every dependency change and never recalled.

## Arma 3

- **INT-AR-01** Shakedown targets Arma 3's current stable (main) branch: game update 2.22 (2026-08-11) with its hotfix (2026-08-26) and its mini-update (2026-09-30). It needs at least 2.18, for `toJSON` and `fromJSON`; it uses `ScriptError` (2.14), `EntityCreated` (2.10), `ExtensionCallback`, `systemTimeUTC` and `diag_stacktrace`. It uses nothing from the development or profiling branch (`freeExtension`, `diag_mergeConfigFile`, `diag_captureFrame`). Each machine reports its `productVersion` at `ready`, and the result file records it (`CR-01`, `CR-51`).

- **INT-AR-02** Every process is started with these options. `<mods>` is the setup's mod folders followed by the framework's addon, separated by `;`. `<profile>` is `<session folder>\machines\<machine>-<start number>` (`CR-01`, `CR-24`; gap H8):

  | Process | Executable | Options |
  |---|---|---|
  | Singleplayer `p1` | `arma3_x64.exe` | `-autotest=<profile>\autotest.cfg -profiles=<profile> -name=p1 -mod=<mods> -filePatching -debug -window -noPause -noPauseAudio -noSound -skipIntro -noSplash -world=empty` |
  | Dedicated `server` | `arma3server_x64.exe` | `-port=<base port> -config=<session folder>\server.cfg -profiles=<profile> -name=server -mod=<mods> -filePatching -debug -autoInit -pid=<profile>\server.pid` |
  | Headless client `hcN` | `arma3server_x64.exe` | `-client -connect=127.0.0.1 -port=<base port> -password=<session password> -profiles=<profile> -name=hcN -mod=<mods> -filePatching -debug` |
  | Game client `pN` | `arma3_x64.exe` | `-connect=127.0.0.1 -port=<base port> -password=<session password> -profiles=<profile> -name=pN -mod=<mods> -filePatching -debug -window -noPause -noPauseAudio -noSound -skipIntro -noSplash -world=empty` |

  No run uses `-noLogs`, `-showScriptErrors` or a copied executable. Each game client's window title is set to its machine name once its extension connects, so a person can tell the windows apart.

- **INT-AR-03** The extension `shakedown_x64.dll` is built with `arma-rs`, sets `RVFeature_ContextNoDefaultCall` (it never needs the caller's Steam ID), and offers exactly these calls (`CR-10`; research `12-remote-execution.md`):

  | Call | Does | Returns |
  |---|---|---|
  | `"shakedown" callExtension ["fetch", [orderId, part]]` | Returns one part of a queued order's JSON. | `[partText, isLast]`, each part at most 10000 bytes. |
  | `"shakedown" callExtension ["send", [json]]` | Queues a message for the runner. | `"ok"`, or `"no-runner"` when the connection is down. |
  | `"shakedown" callExtension ["status", []]` | Reports the connection. | `["connected" or "connecting" or "refused", sessionId, machine]`. |
  | Callback `("shakedown", "order", orderId)` | Tells the agent an order is waiting. | - |

  It does no work inside a call except queueing and returns within 1 ms. It retries a callback the engine refused, as `arma-rs` does. Its version string is the framework version. It holds no test logic, because it cannot be replaced while the game runs on the stable branch.

- **INT-AR-04** The runner generates the session mission `shakedown_<session id>.<world>` (`CR-01`, `CR-02`, `CR-03`):
  - `mission.sqm`: for each `pN` a playable rifleman (`B_Soldier_F`, side WEST) in a group of its own, at `[spawn x + 10 × (N - 1), spawn y]` on the ground, direction 0; for each `hcN` a playable `HeadlessClient_F`; a marker `respawn_west` at the spawn point. `spawn` is a setup key, default `[100, 100]`.
  - `description.ext`: `skipLobby = 1`; `respawn = 3`; `respawnDelay = 0`; `respawnOnStart = -1`; `disabledAI = 1`; `allowFunctionsRecompile = 1`; `briefing = 0`; `debriefing = 0`; a `Header` with `gameType = "Coop"`, `minPlayers = 1`, `maxPlayers` = clients plus headless clients; and `class CfgSettings { class CBA { class Caching { compile = 0; xeh = 0; functions = 0; }; }; };`, so CBA functions recompile at every mission start without an extra addon.
  - The mission holds no scripts: the in-game agent comes from the framework's addon (`postInit`).
  - Singleplayer: the mission goes under `p1`'s profile folder in `missions\`, and `autotest.cfg` lists it 100 times (`class TestMissions` with entries `m001` to `m100`), so ending the mission loads it again (a level-B reload in about 1 s). When the list runs out the game exits and the session goes `broken`.
  - Dedicated: the mission is copied to `<game>\MPMissions\` and named in `server.cfg`.

- **INT-AR-05** For a dedicated setup the runner writes `server.cfg` with (`CR-01`, `CR-61`; research `14-sessions.md`):
  `hostname = "shakedown <session id>"`; `password` and `serverCommandPassword`, each random per session; `loopback = 1`; `kickDuplicate = 0`; `BattlEye = 0`; `verifySignatures = 0`; `allowedFilePatching = 2`; `persistent = 1`; `kickTimeout[] = {{0, 0}, {1, 0}, {2, 0}, {3, 0}}`; `headlessClients[] = {"127.0.0.1"}`; `localClient[] = {"127.0.0.1"}`; `maxPlayers` = clients plus headless clients; and `class Missions { class session { template = "shakedown_<session id>.<world>"; difficulty = "Regular"; }; };`.

- **INT-AR-06** The framework's addon `@shakedown` (its PBO and `shakedown_x64.dll`) is reached through a directory junction `<game>\@shakedown` to the installed `@shakedown` folder, because a dedicated server loads extensions only from inside its own folder tree. The session start creates the junction when it is missing and leaves it in place; it is installation, not run state (`CR-01`).

- **INT-AR-07** An RPT line is a script error when, after its timestamp, it starts with `Error ` or contains `Error in expression`. The lines that follow it starting with `Error position:` or `File ` belong to the same error and give its file and line (`CR-23`).

- **INT-AR-08** A machine's RPT, `.mdmp` and `.bidmp` files are written to its `-profiles` folder. A dedicated server started with `-pid` writes its process id to that file and removes it when it stops (`CR-24`, `CR-05`).

- **INT-AR-09** On a dedicated session the server's agent reads each user's client state with `getUserInfo`; a client is ready to play at state 10 (`BRIEFING READ`). A joining client's `time` reads 0 for about 2.5 s, so no framework wait uses `time` for a ceiling (`CR-01`, `CR-20`).

## CBA

- **INT-CBA-01** Shakedown is specified against CBA_A3 3.19.0. The framework does not depend on CBA (`API-05`). When CBA is loaded it uses only `CBA_fnc_getKeybind`, which is public, to press a keybind (`ACT-02`) (`CR-12`).
- **INT-CBA-02** The session mission turns CBA's function cache off through `CfgSettings` (`INT-AR-04`), so a CBA mod's `PREP` functions take a changed file at the next mission start. A level-A reload never assumes a CBA function can be recompiled in place; it asks whether the function is final (`RLD-02`) (`CR-03`; conflict C1).

## ACE3

- **INT-ACE-01** Shakedown is specified against ACE3 3.21.2. The framework does not depend on ACE (`CR-12`).
- **INT-ACE-02** When ACE medical is loaded (`isClass (configFile >> "CfgPatches" >> "ace_medical")`), the reset calls `[unit] call ace_medical_treatment_fnc_fullHealLocal` on the machine where the unit is local. That function is marked `Public: No` by ACE, so the framework names the ACE version it was checked against in its docs, and a reset that finds the function missing fails with the ACE version the machine loaded (`CR-02`).
- **INT-ACE-03** ACE interaction menu actions are a `should` (`ACT-04`). Nothing in the MVP calls ACE's interaction internals (gap H7, Q6).

## HEMTT

- **INT-HEM-01** Shakedown is specified against HEMTT 1.23.1. The framework does not run HEMTT for a mod under test: a watched mod is loaded from a development build file patching can read, such as `hemtt dev` output linked into the game folder under the mod's main prefix, as `hemtt launch` makes it. The project file names the mod's folder and its `source` and `prefix` (`CR-03`; `RLD-06`).
- **INT-HEM-02** The framework's own addon is built with HEMTT, as a development tool of this repository (`07-tech-stack.md`).

## Steam

- **INT-STM-01** Every game process needs the Steam client running and logged in to an account that owns Arma 3; without Steam the game exits at once with code 53. The session start checks that a process `steam.exe` of the current user runs (`SES-03`); a game exit with code 53 during a start is reported as "Steam is not running or not logged in" (`CR-01`).
- **INT-STM-02** Every game process of a session runs on the one Steam account, so they share one Steam ID. That is why `kickDuplicate = 0` and every `kickTimeout` is 0, and why machines are told apart by profile name (`CR-01`; research `10-prior-harness.md`).
- **INT-STM-03** Workshop mods update themselves. The framework records the version each machine loaded in the result file; it does not pin them (`CR-51`).
- **INT-STM-04** At every session start the runner reads the names of the logged-in Steam account, for redaction (`LOG-02`): the Steam folder from the value `SteamPath` under `HKCU\Software\Valve\Steam`, and from that folder's `config\loginusers.vdf` the `AccountName` and `PersonaName` of the entry whose `MostRecent` is `1`. It keeps them in memory and writes them nowhere (`CR-24`; gap H8).

## Windows

- **INT-WIN-01** Shakedown runs on 64-bit Windows 10 or 11. Game processes are owned through a job object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` and `JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION`, each started with `CREATE_SUSPENDED`, assigned, then resumed (`STP-01`) (`CR-05`).
- **INT-WIN-02** A client starts the runner with `CREATE_BREAKAWAY_FROM_JOB` and `DETACHED_PROCESS` so it leaves the client's job and console. If the client's job forbids breakaway, the client starts the runner through WMI (`Win32_Process.Create`), which creates it outside the job. Spike `SPK-06` confirms which path holds under Claude Code (`SES-08`) (`CR-01`, `CR-05`).
- **INT-WIN-03** The runner's own files are `%LOCALAPPDATA%\Shakedown\runner.json` (port, token, process id and start time, readable by the current user only, in the form [contract/cli.md](contract/cli.md) gives), `%LOCALAPPDATA%\Shakedown\record.json` (`STP-03`) and `%LOCALAPPDATA%\Shakedown\runner.log` (`CR-05`, `CR-60`).
- **INT-WIN-04** No part of the framework sends input to the desktop, moves the mouse or takes the foreground (PRD section 7).

## Claude Code and MCP

- **INT-CC-01** The project's `.mcp.json` declares the server as `{"mcpServers": {"shakedown": {"command": "shakedown", "args": ["mcp"], "timeout": 70000}}}`. The owner approves it once per project. The 70 s timeout sits above the 60 s every tool call stays within (`AGT-03`) (`CR-53`).
- **INT-CC-02** Claude Code ends a stdio server's whole process tree on Windows when its session stops, by a job object or by ending each process (`SES-08`); the runner's detached start (`INT-WIN-02`) keeps the game session alive across either (`CR-01`; gap H6).
- **INT-CC-03** Claude Code cuts tool output at 25,000 tokens by default; tool results stay under 20,000 characters and point to files for the rest (`AGT-07`) (`CR-53`).
- **INT-MCP-01** The MCP server speaks protocol revision 2026-07-28 and negotiates the earlier, initialization-based revisions with clients that still use them, through the official Rust SDK `rmcp` (`CR-53`).
