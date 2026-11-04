---
type: Research
title: The harness of arma-grappling-hook
description: What the PowerShell harness and acceptance suite of arma-grappling-hook do, what they measured on the test machine, where they failed, and what the framework keeps, replaces or makes unnecessary.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T21:52:33+01:00 }
sources:
  - { id: hdoc, resource: "arma-grappling-hook@3f4f7bd:docs/harness.md", read: 2026-11-04 }
  - { id: hrun, resource: "arma-grappling-hook@3f4f7bd:harness/run.ps1", read: 2026-11-04 }
  - { id: hgit, resource: "arma-grappling-hook@3f4f7bd: commit d46f4c4 (Harness: several missions per game start, stall detector, window titles, gates -Mods)", read: 2026-11-04 }
  - { id: tech, resource: "arma-grappling-hook@3f4f7bd:spec/07-tech-stack.md, section 6.7", read: 2026-11-04 }
  - { id: areadme, resource: "arma-grappling-hook@3f4f7bd:acceptance/README.md", read: 2026-11-04 }
  - { id: arun, resource: "arma-grappling-hook@3f4f7bd:acceptance/run.ps1", read: 2026-11-04 }
  - { id: suite, resource: "arma-grappling-hook@3f4f7bd:acceptance/suite/m00.psd1", read: 2026-11-04 }
  - { id: alib, resource: "arma-grappling-hook@3f4f7bd:acceptance/missions/common/acc_lib.sqf", read: 2026-11-04 }
  - { id: abase, resource: "arma-grappling-hook@3f4f7bd:acceptance/missions/common/acc_base.inc.sqf", read: 2026-11-04 }
  - { id: arope, resource: "arma-grappling-hook@3f4f7bd:acceptance/missions/common/acc_rope.inc.sqf", read: 2026-11-04 }
  - { id: acommon, resource: "arma-grappling-hook@3f4f7bd:acceptance/missions/common/initServer.sqf, initPlayerLocal.sqf, acc_watcher.sqf, description.ext", read: 2026-11-04 }
  - { id: aletgo, resource: "arma-grappling-hook@3f4f7bd:acceptance/missions/m00-let-go/mission/acc_climber.sqf", read: 2026-11-04 }
  - { id: akeys, resource: "arma-grappling-hook@3f4f7bd:acceptance/missions/m00-keys/mission/acc_climber.sqf", read: 2026-11-04 }
  - { id: afalls, resource: "arma-grappling-hook@3f4f7bd:acceptance/missions/m00-falls/mission/acc_climber.sqf", read: 2026-11-04 }
  - { id: amiss, resource: "arma-grappling-hook@3f4f7bd:acceptance/docs/m00-missions.md", read: 2026-11-04 }
  - { id: f001, resource: "arma-grappling-hook@3f4f7bd:acceptance/findings/m00/001-us-064-promises-a-15-m-let-go-kills-but-with-ace-the-game-s-own-15-m-fall-does-not-always-kill.md", read: 2026-11-04 }
  - { id: f002, resource: "arma-grappling-hook@3f4f7bd:acceptance/findings/m00/002-a-player-who-lets-go-can-stay-hanging-in-the-air-instead-of-falling.md", read: 2026-11-04 }
  - { id: f004, resource: "arma-grappling-hook@3f4f7bd:acceptance/findings/m00/004-a-let-go-8-m-up-now-and-then-does-more-leg-damage-than-the-game-s-own-fall-from-there.md", read: 2026-11-04 }
  - { id: cov, resource: "arma-grappling-hook@3f4f7bd:acceptance/coverage/m00.md", read: 2026-11-04 }
  - { id: coord, resource: "arma-grappling-hook@3f4f7bd:coordinator.md", read: 2026-11-04 }
  - { id: m01, resource: "arma-grappling-hook@3f4f7bd:docs/milestones/m01.md, section on the gates", read: 2026-11-04 }
  - { id: r14, resource: "arma-grappling-hook@3f4f7bd:research/14-test-automation.md", read: 2026-11-04 }
  - { id: tauto, resource: "arma-grappling-hook@3f4f7bd:research/tests/autotest-smoke/README.md", read: 2026-11-04 }
  - { id: tsmoke, resource: "arma-grappling-hook@3f4f7bd:research/tests/harness-smoke/README.md", read: 2026-11-04 }
  - { id: tmanual, resource: "arma-grappling-hook@3f4f7bd:research/tests/manual-smoke/README.md", read: 2026-11-04 }
  - { id: tjip, resource: "arma-grappling-hook@3f4f7bd:research/tests/mp-jip-state/README.md", read: 2026-11-04 }
  - { id: tdc, resource: "arma-grappling-hook@3f4f7bd:research/tests/mp-disconnect/README.md", read: 2026-11-04 }
  - { id: tclock, resource: "arma-grappling-hook@3f4f7bd:tests/m00-let-go-clock/README.md", read: 2026-11-04 }
  - { id: prd, resource: "spec/client-requirements.md", read: 2026-11-04 }
---

# The harness of arma-grappling-hook

The harness of `arma-grappling-hook` is the only Arma 3 test harness this project has run, and the first thing the framework replaces[^prd]. It is two PowerShell scripts: `harness/run.ps1` starts the game and runs test missions[^hrun], and `acceptance/run.ps1` turns a suite file into harness runs and reports each check[^arun]. Checks are SQF inside generated missions and report through `diag_log` lines that the harness reads from each machine's RPT[^hdoc]. The mod's tech stack makes the result lines, `START`, `KILL` and the rule that a script error fails a mission its contract between missions and the harness[^tech]. All measurements below were taken on the Windows test machine, game 2.22.154089 Stable, between 2026-10-31 and 2026-11-04[^hdoc][^cov][^tauto].

## 1. What `harness/run.ps1` does

Step by step, as the script runs[^hrun]:

1. **Checks Steam.** It refuses to start when no `steam` process runs, because without Steam the game exits at once with code 53[^hrun][^tmanual]. It does not check that a user is logged in[^hrun].
2. **Builds the mod** with `hemtt build --no-bin` unless `-NoBuild` is given, and builds any research test addon in a throwaway HEMTT project under `%TEMP%`[^hrun]. Workshop mods are loaded from fixed Workshop IDs in the Steam library folder (CBA 450814997, ACE 463939057)[^hrun].
3. **Generates each mission** in a run folder under `%TEMP%`: the folder's own files, a generated `mission.sqm` with playable riflemen `p1`..`pN`, `HeadlessClient_F` slots `hc1`..`hcN` and an empty helipad whose init field logs the mission index and the game and mod versions on every machine, and `skipLobby = 1` plus a `Header` and any `-Define` in front of `description.ext`[^hrun]. It also writes `agh_run.sqf` (the run's settings as a hashmap), `agh_done.sqf` (ends the mission) and `agh_manual.sqf` (logged scroll actions for manual runs) into every mission[^hrun][^hdoc].
4. **Singleplayer** runs every mission as one `-autotest` list with `-profiles`, `-name=p1`, `-window -noPause -noPauseAudio -skipIntro -noSplash -world=empty -showScriptErrors` and `-noSound`[^hrun]. A mission passes with `endMission "END1"` and fails with `LOSER`[^hdoc].
5. **Dedicated** copies the missions into `<Arma 3>\MPMissions`, writes a `server.cfg` with `loopback = 1`, `kickDuplicate = 0`, `BattlEye = 0`, `verifySignatures = 0`, `allowedFilePatching = 2`, `persistent = 1`, `kickTimeout[]` all 0, `headlessClients[]` and `localClient[]` of `127.0.0.1`, a per-run random `serverCommandPassword` and the missions as the server's mission cycle, and starts `arma3server_x64.exe` with `-autoInit` on port 2402[^hrun]. When the server logs `AGH_TEST|SERVER_UP` it starts the headless clients (`arma3server_x64.exe -client`) and the game clients (`arma3_x64.exe -connect=127.0.0.1`), each with its own profile folder and `-name`[^hrun].
6. **Late joiners and kills.** Clients past `-Clients` start only when the server logs `AGH_TEST|START|<name>`, which also restarts a running client; `AGH_TEST|KILL|<name>` ends a client's process, which the server sees as a crash[^hdoc][^hrun]. A restarted machine gets a fresh profile folder, so each start keeps its own RPT[^hrun].
7. **Several missions per start.** At a mission's `DONE` on a dedicated server, the harness ends every client and headless client process; the server's `agh_done.sqf` pings the clients over public variables every 0.25 s, kicks each one (`#kick`) once it has not answered for 1.5 s, logs `AGH_RUN|left`, and loads the next mission with `#mission`; the harness then starts fresh machines for it[^hrun][^hdoc].
8. **Follows the lead machine's RPT** (the game in singleplayer, the server on a dedicated run) every 500 ms for the mission markers, `START`, `KILL` and `DONE`, until the last `DONE`, the lead process exits, a mission's ceiling passes (default 600 s), the stall detector fires, or a manual run is stopped[^hrun]. It sets each window's title to its machine's name with `SetWindowText`[^hrun].
9. **The stall detector** watches the length of every machine's newest RPT and ends the run as `STALL` once no machine's log has grown for `-StallSec` (default 90 s)[^hrun][^hdoc].
10. **Stops every process it started** in a `finally` block, by process object, and deletes the copies in `MPMissions`[^hrun]. At the last `DONE` of a dedicated run it stops the clients before the server, so no machine logs the errors of a mission ending under it[^hrun].
11. **Writes `result.txt` per mission**: a header with the settings, the outcome and the seconds, a verdict line, and from each machine's RPT every `AGH_`/`AGHT_` line, every mod `AGH|` line, every `Warning Message` line and every script error, cut into missions at the engine's `Starting mission:` line before each `AGH_RUN|mission` marker[^hrun][^hdoc]. The verdict is `PASS` only when the mission ended with `DONE` (or exit code 0), logged `DONE|ok=true`, no `AGH_CHECK` line said `FAIL` and no machine logged a script error line[^hrun]. Outcomes are `DONE`, `EXIT <code>`, `TIMEOUT`, `STALL`, `SERVER EXITED`, `MISSION <i> AGAIN`, `NOT STARTED`, `STOPPED` and `CLIENTS EXITED`[^hrun][^hdoc]. The script exits 0 when every mission passed[^hrun].

Script errors are found by matching RPT lines against a fixed list of `Error ...` patterns[^hrun]. Every outcome other than `DONE` makes the verdict `FAIL`: a stall, a timeout and a crash are reported the same way as a failed check[^hrun].

## 2. What `acceptance/run.ps1` adds

- **A suite file per milestone** (`acceptance/suite/m00.psd1`): the missions with their checks, ceilings and cases, the named runs with their harness arguments and the checks each run needs, `MultiplayerMachines` (the machines each multiplayer check must log a line from), and the quick runs[^suite].
- **Filters**: `-Checks a,b`, `<check>#<case>` for one case, `<check>@<run>` for one check in one run, `-Runs`, `-Quick`, `-Full`, `-Finding <file>` (the checks in a finding's `checks:` key) and `-List`[^arun][^areadme].
- **Setups per check**: a check runs in singleplayer unless its promise needs more than one machine, and with ACE only when its promise concerns ACE; `-Full` runs everything everywhere[^areadme][^coord].
- **The filter reaches the mission** as a generated `acc_checks.sqf` holding `acc_only = [...]`, so a mission runs only the steps the named checks need and the suite's source is never changed[^arun][^alib].
- **One row per check and run**: `PASS`, `FAIL`, `MISSING` (no line) or `MISSING <machine>`, plus a verdict row per mission and the seconds of each run[^arun]. It exits 0 only when every row passed[^arun].

All of a run's missions go to one game start through `harness/run.ps1 -Mission a,b,...`[^arun].

## 3. What the checks look like in SQF

A check is SQF in a mission's `acc_climber.sqf`, which includes shared files and calls the mod's public functions directly[^aletgo][^amiss]. It logs `AGH_CHECK|<check id>|PASS|<detail>` or `...|FAIL|...`, one line per case, with `machine=` and `ace=` appended[^alib]. The detail is long: the measured values and the engine's view (`time`, `diag_tickTime`, `diag_fps`, `isGamePaused`, `isGameFocused`, `accTime`, `dialog`, the animation, ground contact)[^abase].

The helpers every check uses, which are the first draft of the check API's vocabulary[^alib][^abase][^arope]:

| Helper | What it does | API word it suggests |
|---|---|---|
| `acc_fnc_want`, `acc_fnc_wantCase` | Is this check or case in the filter | test selection, `test.only` |
| `acc_fnc_log` | Logs a check line; a client also sends it to the server, because the harness can end a client before its last lines reach the RPT | `expect` and report, with a guaranteed delivery path |
| `acc_fnc_poll` | `waitUntil` a condition or a ceiling in real seconds, then returns the condition | `waitFor(condition, timeout)` |
| `acc_fnc_settle` | Polls until a unit rests on a surface for 15 frames, ceiling 12 s | a domain wait: "settled" |
| `acc_fnc_serverHasMe` | Asks the server by public variables where it has the unit, until within 0.1 m and 3 degrees, ceiling 5 s | wait until the server agrees, run code on the server |
| `acc_fnc_heal`, `acc_fnc_damage`, `acc_fnc_hAbove` | Heal (with ACE's full heal), read damage, height above the surface below | scene and state helpers |
| `acc_fnc_drawCompare`, `acc_fnc_drawSample` | Has every machine compare its value with the server's and waits for all to answer, ceiling 6 s | read on every machine and compare |
| `EachFrame` sampling in `_letGo` | Samples position, velocity, state and `time` every frame of a step | traces of a step |
| `_standTop`, `_getOnTop`, `_descend` | Put the unit at a spot, retry up to five times, then act | scene setup with retries |
| `selectPlayer` hand-off | Hands the player to a spare unit before a fall that may kill[^afalls] | a fixture for deaths under `-autotest` |

Keys are driven by calling the CBA keybind's down and up handlers from `CBA_fnc_getKeybind` with `[display, dik, shift, ctrl, alt]`, not by sending input[^akeys][^amiss]. Server state is read through public-variable request and answer pairs (`acc_q`/`acc_a`, `acc_drawQ`/`acc_drawA`), because the mod's remote-execution whitelist is not the suite's to use[^alib]. The watchers on other machines are a separate script started from `initPlayerLocal.sqf`, coordinated with the climber by public variables such as `acc_ready_p2` and `acc_p1done`[^acommon]. A late client is started by the server logging `AGH_TEST|START|p3` once a rope is up[^acommon].

What this shows: a check needs a way to run code on a named machine and get a value back, a wait with a ceiling, a guaranteed report path from every machine, a step trace, and scene setup that waits for the server to agree. Today each of these is written by hand per mission, with public variables and `diag_log`.

## 4. What it measured

| What | Value | Date |
|---|---|---|
| Singleplayer start to the first mission line | 18 to 20 s | 2026-11-03[^hdoc] |
| Dedicated start: server / headless client / game clients | 13 / 17 to 18 / 24 to 26 s, all four started together | 2026-11-03[^hdoc] |
| Next mission in one singleplayer `-autotest` start | about 1 s | 2026-11-03[^hdoc] |
| Next mission on a dedicated server | about 2 s for the change, then about 25 s for the fresh machines to be in | 2026-11-03[^hdoc] |
| Time saved per extra mission in one start | about 20 s singleplayer, about 13 s dedicated | 2026-11-03[^hdoc] |
| Server left to shut down by itself at the end | about 18 s more, waiting for the clients it had lost | 2026-11-03[^hdoc] |
| A process that ended stays listed on the server | about 15 s | 2026-11-03[^hdoc] |
| A crash as the server sees it (`PlayerDisconnected`, `HandleDisconnect`) | 15.6 s after the process was ended | 2026-10-31[^tdc] |
| A clean leave (closing the mission display) | `PlayerDisconnected` +0.08 s, `HandleDisconnect` +0.11 s; `OnUserDisconnected` only when the process ended, 15 s later | 2026-10-31[^tdc] |
| Longest silence across all machines in a passing run | 57 s, over 110 runs; stall threshold set to 90 s | 2026-11-02 and 03[^hdoc] |
| Start options tried: `-enableHT`, `-hugePages`, `-cpuCount=4 -exThreads=7`, a smaller window, `-loadMissionToMemory`, no `-filePatching` | none moved a start by more than 1 s; none kept | 2026-11-03[^hdoc] |
| Where start time goes | about 7 s to Steam's manager, then addons, then the mission | 2026-11-03[^hdoc] |
| `-autotest` with a passing and a failing mission | exits by itself, code 1, 31 s; pass only: code 0, 24 s | 2026-10-31[^tauto] |
| Harness smoke: server, one headless client, one game client | server mission up 16 s; clients reported within 23 s; whole run 60 s | 2026-10-31[^tsmoke] |
| `-NoAutoInit` | mission time stayed at 0 for about a minute after the mission started | 2026-10-31[^tjip] |
| Acceptance suite, default run | 295 s, down from 853 s; `-Quick` 117 s; `-Full` 830 s, later 1007 s | 2026-11-03 and 04[^coord][^cov] |
| `-Full` per run | `sp-cba` 130 s, `sp-cba-ace` 138 s, `dedicated-cba` 357 s, `dedicated-cba-ace` 382 s | 2026-11-04[^cov] |
| One defect chase | 12 game starts of about 3.5 min each, 47 of a 64-minute round | 2026-11-04[^coord] |
| Engine stepping of a far AI unit | moved in about 1 frame of 10, against every frame for the player | 2026-11-04[^amiss][^f004] |

A run takes minutes because of how long its missions run, not because of the start[^coord].

## 5. Where it failed or was flaky

- **The Shadow stream and the stopped clock (M00-002).** In 2 of about 13 singleplayer runs, the game's clock `time` stood still for minutes while frames dropped from 150 to 170 to 19 a second; nothing fell, the game's own fall included[^f002]. The owner confirmed that the Shadow PC stream to the test machine was disconnected at both times[^f002]. The suite now counts a run only while the stream is connected for its whole length[^areadme]. A run with the game minimised and unfocused, stream connected, kept its clock running at about 32 frames a second[^tclock]. The harness's stall detector did not catch it, because the game went on logging[^f002][^hrun].
- **Clients kept across a mission change.** In about one change in two, the engine left a client in a mission it never loaded, or gave a headless client no role, after which it ran the mission as a server of its own; so every mission gets fresh client processes[^hdoc]. A machine still listed when the next mission loads keeps its slot, and a machine kicked while it still runs comes back, a headless client at once, with script errors[^hdoc]. A server that waited in the lobby (`-NoAutoInit`) went back to the first mission of its cycle once clients joined, so such a run takes one mission[^hdoc]. Making mission changes reliable took most of a 2 h 15 min harness round[^coord][^hgit].
- **Kicks with one Steam ID.** Every client of a run shares one Steam ID, so a kick of one kept all of them out until `kickTimeout[]` was set to 0 for every kind of kick[^hdoc]. Both disconnects in `mp-disconnect` carried the same Steam UID[^tdc].
- **ACE's random falls (M00-001).** With ACE, a 15 m fall ends healthy, incapacitated or dead at random, and from 5 m the legs read 0, 0.45, 0.5 or 0.95[^f001][^amiss]. The check compares against up to ten of the game's own falls from the same point[^amiss]. A check of game behaviour that is random needs a reference measured under the same conditions in the same run.
- **The check's own fixture made it flaky (M00-004).** To keep `-autotest` from ending the mission when the player died (`EndMode = KILLED`), the check handed the player to a spare AI 40 m away, which the engine steps about one frame in ten, and the leg damage jumped[^f004][^amiss]. As the player, 30 of 30 falls matched within 0.006[^f004].
- **A client's last log lines can be lost** when the harness ends it, so every client check line is also sent to the server and logged there[^alib][^amiss].
- **The server lags a move.** A unit moved with `setPosASL` reaches the server a moment later; a request sent in the same moment was refused, so checks wait until the server agrees[^amiss].
- **Whole-machine stops.** A `-Full` run whose log stopped while addons loaded never finished; neither the stall detector nor the ceiling ended it, because the whole machine had stopped, and it was rebooted hours later[^cov]. The machine rebooted twice more during sessions and cut rounds off[^coord].
- **Stalls with ACE.** In singleplayer with ACE, three of five game starts stalled at different steps, likely the stream, and were not chased further[^m01][^coord].
- **Environment.** Steam stops at reboots and sometimes at the end of a session; it is checked before every round[^coord]. A missing required addon stops a dedicated server at its warning with the process running, so a run waits out its ceiling[^r14]. Under `-autotest` that warning box does not show[^r14].

## 6. What the framework keeps, replaces, and makes unnecessary

| Part | Keep, replace or drop | Why |
|---|---|---|
| `-autotest` for singleplayer, `server.cfg` keys, start options, `skipLobby`, generated `mission.sqm`, `HeadlessClient_F` slots | Keep | Measured working on the test machine[^hdoc][^tsmoke][^tauto]. |
| Per-machine profile folders and `-name`, fresh profile per restart | Keep | Each start keeps its own RPT[^hrun]. |
| Versions line per machine per mission | Keep | Workshop mods update themselves[^r14]. |
| Stopping every process in `finally` | Replace | It cannot run when the runner itself dies (see `17-health.md`). |
| A game start per run, a mission per group of checks | Replace | `CR-01` asks for one session and many checks; a fresh mission per change cost about 25 s of client starts on a dedicated server[^hdoc]. |
| Results by `diag_log` lines parsed from RPTs | Replace for verdicts, keep for logs | A lost client line had to be relayed through the server[^alib]; `CR-10` and `CR-53` want values back as data. |
| Stall = no log line for 90 s | Replace | It missed a game whose clock stopped while it went on logging[^f002]; it cannot tell a stall from a crash or a stopped machine[^cov]. |
| Verdict `FAIL` for a stall, timeout or crash | Replace | `CR-31` and `G3` want those reported invalid[^prd]. |
| Hand-written public-variable request and answer pairs | Drop | A remote-execution channel per machine replaces them (`CR-10`). |
| Calling CBA keybind handlers instead of pressing keys | Keep as the default way to act | It drives the mod's real path without input[^akeys]. |
| Suite files, filters by check, case, run and finding | Keep the ideas | They kept the fix loop narrow[^coord]; `CR-40`, `CR-41`. |
| Window titles per machine | Keep | Lets a person tell windows apart in manual runs[^hdoc]. |
| Keep-alive key and "run only with the stream connected" | Replace with a health check | A person cannot watch the stream for a suite that runs unattended[^areadme]. |

## 7. Findings and what they force

1. **A game start costs 18 to 26 s per machine and no start option shortens it.** Forces sessions that stay up across checks, and a reset in place as the default between checks. Touches `CR-01`, `CR-02`, `G2`[^hdoc][^prd].
2. **A mission change with clients connected was unreliable about one time in two.** Forces the framework not to rely on changing missions under connected clients; a change that is needed restarts the clients, and the framework measures it again (see `14-sessions.md`). Touches `CR-01`, `CR-04`[^hdoc].
3. **The stall detector watched logs, and logs kept growing while the clock stood still.** Forces health on in-game signals: game time against real time, frame number, every machine answering. Touches `CR-30`, `CR-31`[^f002][^tclock].
4. **Every stall, timeout and crash was a `FAIL`.** Forces the three outcomes `pass`, `fail`, `invalid`, with the environment's faults as `invalid`. Touches `CR-31`, `G3`[^hrun].
5. **Process cleanup ran only if the runner lived, and the machine reboots and shuts down on its own.** Forces process ownership that survives a runner crash and a record of what a run started, checked at the next start. Touches `CR-05`, `CR-07`[^hrun][^coord].
6. **Results were text lines in RPTs, and a client's last lines could be lost.** Forces a result channel per machine that returns data and confirms delivery, and RPTs kept only as logs. Touches `CR-10`, `CR-24`, `CR-51`[^alib].
7. **`-autotest` ends a singleplayer mission when the player dies, and the work-around made a check flaky.** Forces the framework to own mission end in singleplayer, so a death is an event a check can assert, not the end of the session. Touches `CR-02`, `CR-14`[^f004].
8. **Random game behaviour (ACE medical) and engine stepping (far AI) decide verdicts.** Forces repeat counts and an in-run baseline under the same conditions. Touches `CR-32`, `CR-34`[^f001][^f004].
9. **All clients share one Steam ID.** Forces `kickDuplicate = 0` and `kickTimeout[]` of 0 in every generated `server.cfg`, and finding machines by profile name, not by slot or Steam ID. Touches `CR-01`, `CR-04`[^hdoc][^hrun].
10. **The checks' helpers are the vocabulary.** Wait with a ceiling, run on a machine, read on every machine and compare, wait until the server agrees, trace a step, settle a unit, heal, call a keybind handler. Forces these into the SQF check API. Touches `CR-20`, `CR-22`, `CR-50`, `CR-12`[^alib][^abase].

## Needs a spike

- **A reset in place replaces a mission change.** Pass condition: in one dedicated session with two clients and one headless client, 20 checks run one after another with a reset between them, and every check sees the same starting state as a fresh mission (same objects, time, weather and player state), with no client lost.
- **Whether the stream cut alone stops the clock.** Pass condition: with the stream cut on purpose for 5 minutes, a heartbeat logs game time against real time on each machine; the result says whether `time` stops, and the health check flags the run invalid within its threshold.

[^hdoc]: arma-grappling-hook@3f4f7bd:docs/harness.md
[^hrun]: arma-grappling-hook@3f4f7bd:harness/run.ps1
[^hgit]: arma-grappling-hook@3f4f7bd: commit d46f4c4 (Harness: several missions per game start, stall detector, window titles, gates -Mods)
[^tech]: arma-grappling-hook@3f4f7bd:spec/07-tech-stack.md, section 6.7
[^areadme]: arma-grappling-hook@3f4f7bd:acceptance/README.md
[^arun]: arma-grappling-hook@3f4f7bd:acceptance/run.ps1
[^suite]: arma-grappling-hook@3f4f7bd:acceptance/suite/m00.psd1
[^alib]: arma-grappling-hook@3f4f7bd:acceptance/missions/common/acc_lib.sqf
[^abase]: arma-grappling-hook@3f4f7bd:acceptance/missions/common/acc_base.inc.sqf
[^arope]: arma-grappling-hook@3f4f7bd:acceptance/missions/common/acc_rope.inc.sqf
[^acommon]: arma-grappling-hook@3f4f7bd:acceptance/missions/common/initServer.sqf, initPlayerLocal.sqf, acc_watcher.sqf, description.ext
[^aletgo]: arma-grappling-hook@3f4f7bd:acceptance/missions/m00-let-go/mission/acc_climber.sqf
[^akeys]: arma-grappling-hook@3f4f7bd:acceptance/missions/m00-keys/mission/acc_climber.sqf
[^afalls]: arma-grappling-hook@3f4f7bd:acceptance/missions/m00-falls/mission/acc_climber.sqf
[^amiss]: arma-grappling-hook@3f4f7bd:acceptance/docs/m00-missions.md
[^f001]: arma-grappling-hook@3f4f7bd:acceptance/findings/m00/001-us-064-promises-a-15-m-let-go-kills-but-with-ace-the-game-s-own-15-m-fall-does-not-always-kill.md
[^f002]: arma-grappling-hook@3f4f7bd:acceptance/findings/m00/002-a-player-who-lets-go-can-stay-hanging-in-the-air-instead-of-falling.md
[^f004]: arma-grappling-hook@3f4f7bd:acceptance/findings/m00/004-a-let-go-8-m-up-now-and-then-does-more-leg-damage-than-the-game-s-own-fall-from-there.md
[^cov]: arma-grappling-hook@3f4f7bd:acceptance/coverage/m00.md
[^coord]: arma-grappling-hook@3f4f7bd:coordinator.md
[^m01]: arma-grappling-hook@3f4f7bd:docs/milestones/m01.md, section on the gates
[^r14]: arma-grappling-hook@3f4f7bd:research/14-test-automation.md
[^tauto]: arma-grappling-hook@3f4f7bd:research/tests/autotest-smoke/README.md
[^tsmoke]: arma-grappling-hook@3f4f7bd:research/tests/harness-smoke/README.md
[^tmanual]: arma-grappling-hook@3f4f7bd:research/tests/manual-smoke/README.md
[^tjip]: arma-grappling-hook@3f4f7bd:research/tests/mp-jip-state/README.md
[^tdc]: arma-grappling-hook@3f4f7bd:research/tests/mp-disconnect/README.md
[^tclock]: arma-grappling-hook@3f4f7bd:tests/m00-let-go-clock/README.md
[^prd]: spec/client-requirements.md
