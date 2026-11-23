---
type: Research
title: Health, stalls and clean stops
description: How the framework tells a healthy session from a stalled one, what the game does without focus or a visible window, how every process a run started is stopped even when the runner crashes, how the environment is checked before a run, and how a crash is told from a stop.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T21:52:33+01:00 }
sources:
  - { id: startup, resource: "https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters (oldid 378577)", read: 2026-11-04 }
  - { id: servercfg, resource: "https://community.bistudio.com/wiki/Arma_3:_Server_Config_File (oldid 379176)", read: 2026-11-04 }
  - { id: time, resource: "https://community.bistudio.com/wiki/time (oldid 379479)", read: 2026-11-04 }
  - { id: tick, resource: "https://community.bistudio.com/wiki/diag_tickTime (oldid 379413)", read: 2026-11-04 }
  - { id: fps, resource: "https://community.bistudio.com/wiki/diag_fps (oldid 379331)", read: 2026-11-04 }
  - { id: focus, resource: "https://community.bistudio.com/wiki/isGameFocused (oldid 309152)", read: 2026-11-04 }
  - { id: userinfo, resource: "https://community.bistudio.com/wiki/getUserInfo (oldid 378191)", read: 2026-11-04 }
  - { id: cstate, resource: "https://community.bistudio.com/wiki/getClientStateNumber (oldid 348456)", read: 2026-11-04 }
  - { id: crash, resource: "https://community.bistudio.com/wiki/Crash_Files", read: 2026-11-04 }
  - { id: pipe, resource: "https://community.bistudio.com/wiki/Arma_3:_Named_Pipe (oldid 378557)", read: 2026-11-04 }
  - { id: meta, resource: "https://community.bistudio.com/wiki/Arma_3:_meta.cpp (oldid 373435)", read: 2026-11-04 }
  - { id: jobs, resource: "https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects (ms.date 2025-07-14)", read: 2026-11-04 }
  - { id: joblimit, resource: "https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information", read: 2026-11-04 }
  - { id: jobassign, resource: "https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject", read: 2026-11-04 }
  - { id: hung, resource: "https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-ishungappwindow", read: 2026-11-04 }
  - { id: steamreg, resource: "https://gamedev.stackexchange.com/questions/167176 (answer 167219, read through the Stack Exchange API)", read: 2026-11-04 }
  - { id: shadow, resource: "https://support.shadow.tech/hc/en-us/articles/32731841405073-Shadow-PC-and-the-Automatic-Shutdown (updated 2026-07-23)", read: 2026-11-04 }
  - { id: t1, resource: "research/10-prior-harness.md", read: 2026-11-04 }
  - { id: hdoc, resource: "arma-grappling-hook@3f4f7bd:docs/harness.md", read: 2026-11-04 }
  - { id: hrun, resource: "arma-grappling-hook@3f4f7bd:harness/run.ps1", read: 2026-11-04 }
  - { id: f002, resource: "arma-grappling-hook@3f4f7bd:acceptance/findings/m00/002-a-player-who-lets-go-can-stay-hanging-in-the-air-instead-of-falling.md", read: 2026-11-04 }
  - { id: areadme, resource: "arma-grappling-hook@3f4f7bd:acceptance/README.md", read: 2026-11-04 }
  - { id: tclock, resource: "arma-grappling-hook@3f4f7bd:tests/m00-let-go-clock/README.md", read: 2026-11-04 }
  - { id: cov, resource: "arma-grappling-hook@3f4f7bd:acceptance/coverage/m00.md", read: 2026-11-04 }
  - { id: coord, resource: "arma-grappling-hook@3f4f7bd:coordinator.md", read: 2026-11-04 }
  - { id: m01, resource: "arma-grappling-hook@3f4f7bd:docs/milestones/m01.md, section on the gates", read: 2026-11-04 }
  - { id: r14, resource: "arma-grappling-hook@3f4f7bd:research/14-test-automation.md", read: 2026-11-04 }
  - { id: tmanual, resource: "arma-grappling-hook@3f4f7bd:research/tests/manual-smoke/README.md", read: 2026-11-04 }
  - { id: prd, resource: "spec/client-requirements.md", read: 2026-11-04 }
---

# Health, stalls and clean stops

A verdict can be trusted only when the run can tell the mod's failure from the environment's (`G3`)[^prd]. This topic covers how a session's health is watched (`CR-30`), how an environment fault becomes `invalid` rather than `fail` (`CR-31`), how every process is stopped even when the runner dies (`CR-05`), and how the environment is checked before a run (`CR-06`). Engine commands are checked on the Bohemia Interactive Community Wiki for game 2.22, stable branch; none below needs the profiling or development branch. The prior harness's findings come from `10-prior-harness.md`[^t1].

## 1. A healthy session and a stalled one

What the prior harness learned: its stall detector ended a run once no machine's RPT had grown for 90 s[^hdoc]. It missed the worst stall it met. While the Shadow stream to the test machine was cut, the game's clock `time` stood still for minutes while frames fell from 150 to 170 to 19 a second, and the game kept logging[^f002]. Every fall stopped, the game's own included, and the checks failed instead of being marked invalid[^f002]. The suite now counts a run only with the stream connected for its whole length[^areadme]. In singleplayer with ACE, three of five game starts stalled at different steps, each ended by the 90 s detector[^m01]. A `-Full` run also hung while addons loaded and was ended by nothing, because the whole machine had stopped[^cov].

The signals the engine offers, per machine:

| Signal | What it gives | Source |
|---|---|---|
| `time` | Seconds since the mission started; stops when the game is paused; differs per client; a joining client starts at 0 and jumps to the real value after about 2.5 s | [^time] |
| `diag_tickTime` | Real seconds since the game started; keeps running while paused | [^tick] |
| `diag_fps` | Average frame rate over the last 16 frames, loading screens included | [^fps] |
| `isGameFocused` | Whether the game window has focus | [^focus] |
| `getUserInfo` on the server | Per user: owner, client state, whether headless, and `[ping, bandwidth, desync]` | [^userinfo] |
| `getClientStateNumber` | 0 `NONE` to 12; 10 `BRIEFING READ` is ready to play | [^cstate] |
| Process and window, from outside | Whether the process lives; `IsHungAppWindow` reports a window that has not called `PeekMessage` for 5 s | [^hung] |

A health check built on these **(inference)**:

- **The clock moves.** Each machine sends a heartbeat with `time`, `diag_tickTime`, `diag_frameNo` and `diag_fps`. The prior harness's clock watch called a second with `time` advancing less than a fifth of real time a stop[^tclock]. A session is unhealthy when game time on a machine runs far slower than real time while the game is not paused by the framework.
- **The frame rate is sane.** A floor, compared with the machine's own baseline. Since 2.20 a server with no players limits itself to 30 frames a second[^startup], so the server's floor cannot be the clients'.
- **Every machine answers.** Each heartbeat arrives within a ceiling, and on the server each user's client state stays at the ready state with a ping and desync below a ceiling[^userinfo][^cstate].
- **The processes live and the windows answer.** The runner watches each process, and each client window with `IsHungAppWindow`[^hung].
- **The log grows** stays as a last signal, for a machine whose script channel is dead.

A failed health check during a check makes that check `invalid`, with the reason and each machine's last heartbeat, never `fail`.

## 2. Focus, minimised and not shown

- `-noPause` keeps the game running when its window has no focus, and `-noPauseAudio` keeps its audio[^startup]. Without them a game that loses focus pauses, and `time` stops when the game is paused[^startup][^time].
- The FPS is limited automatically when a windowed game is unfocused[^startup]. With `-noPause`, a minimised and unfocused game on the test machine kept its clock running at about 32 frames a second[^tclock].
- With no visible desktop (the Shadow stream cut), the game went on at about 19 frames a second with its clock stopped[^f002]. Why the clock stops then is not known **(unverified)**.
- The test machine renders a window only while the owner is connected through Shadow PC[^r14][^tmanual]. Shadow PC shuts down by itself: online, when the stream is open but no input arrives for a while, and offline, when the stream is closed or disconnected; it "relies on input from the user's peripherals to stay active", even while a program runs[^shadow]. The test machine rebooted several times during sessions, ending rounds without warning[^coord]; that this was Shadow's shutdown is **(unverified)**. The prior coordinator ran a keep-alive key every 60 s, which reached the game as a key event[^tclock].
- A dedicated server and headless clients need no window[^r14], so a multiplayer session whose clients render is as exposed as singleplayer, and one of only a server and headless clients is not **(inference)**.

## 3. Stopping every process

The prior harness stopped every process it started in a `finally` block, by process object, and removed the mission copies it had put into the game's `MPMissions` folder[^hrun]. That cannot run when the runner itself is killed, and a reboot left `%TEMP%` empty, so run folders do not outlive the machine[^r14].

Windows job objects close that gap:

- A job object manages a group of processes as a unit; terminating all of them is one of its operations[^jobs].
- Child processes created with `CreateProcess` by a process in a job belong to the job by default, unless the job allows breakaway; children created through `Win32_Process.Create` do not[^jobs].
- With `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, closing the last handle to the job terminates every process in it, and in nested jobs every process of its child jobs too[^jobs][^joblimit]. A handle closes when its owner process dies **(inference)**, so a runner that holds the only handle takes every game process with it when it crashes.
- Nested jobs work from Windows 8 and Windows Server 2012, so the runner can itself run inside another job, such as a terminal's[^jobs][^jobassign].
- A process assigned after it started may already have started children outside the job; `CREATE_SUSPENDED` creates a process that runs only once resumed, and is required when the job has `JOB_OBJECT_SECURITY_ONLY_TOKEN`[^jobassign]. Starting each game process suspended, assigning it, then resuming it leaves no gap **(inference)**.
- `JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION` suppresses the system's fault error box for processes in the job[^joblimit], so a crashed game does not wait on a dialog **(needs a spike)**.

What a job object does not cover: the Steam client, which starts the game through the launcher when Steam is not running, outside any job[^tmanual]; files and folders a run wrote, such as mission copies in `MPMissions`[^hrun]; and ports. So each run also writes a record of what it started and wrote (process ids with start times, folders, ports) where the next run finds it, and the next run clears what is left before it starts (`CR-05`, scenario 6 of the PRD)[^prd] **(inference)**. Process ids are matched with their start time, since an id can be reused **(inference)**. The dedicated server's `-pid` file, written at start and removed when the server stops, tells a live server from one that ended[^startup].

## 4. Checking the environment before a run

| Check | How | Source |
|---|---|---|
| Steam runs | a `steam` process; without it `arma3_x64.exe` exits at once with code 53 | [^tmanual][^hrun] |
| Steam is logged in | `HKCU\Software\Valve\Steam\ActiveProcess`, value `ActiveUser`: the SteamID3 of the logged-in user; 0 when nobody is, as the prior coordinator polled it after starting Steam | [^steamreg][^coord] |
| The game is installed | `arma3_x64.exe` and `arma3server_x64.exe` in the Steam library folder | [^r14] |
| Each mod is present and which version | the mod folder's `meta.cpp`: `publishedid` and `timestamp` of the last Workshop update, written when a mod is published | [^meta] |
| Versions as the game loaded them | each machine logs `productVersion` and each mod's `CfgPatches` `versionStr` at mission start | [^hrun] |
| BattlEye off | `BattlEye = 0` in the generated `server.cfg`; BattlEye closes additional instances | [^servercfg][^r14] |
| A required addon is missing | a dedicated server stops at its warning with the process running, and the run waits out its ceiling; under `-autotest` the box does not show | [^r14] |
| Free disk | the prior coordinator checked `C:` before a whole-suite run | [^coord] |

Workshop mods update themselves, so a version can change between two runs without anyone choosing it[^r14]; the check compares what is on disk with what a run asks for, and the result file records what each machine loaded.

## 5. A crash or a stop

- Arma 3 writes `arma3.rpt`, `arma3.bidmp` and `arma3.mdmp` to `%LOCALAPPDATA%\Arma 3`, or to the `-profiles` folder when one is given[^crash].
- The game's freeze check writes up to 4 dumps per game run, 2 per distinct freeze, unless `-noFreezeCheck` is given[^startup]. A dump for a freeze is evidence for a stalled session as well as for a crash.
- `-autotest` exits 0 when every mission passed and 1 when one failed; a game started without Steam exits with code 53[^tmanual][^hdoc].
- A client killed or crashed is seen by the server about 15.6 s later, as a disconnect[^r14].
- The named pipe of `-command=<name>` accepts `shutdown`, on clients only[^pipe], which gives the runner a clean stop for a client besides killing it.

So a stop is a process that ended after the framework asked it to; a crash is a process that ended unasked, with or without a new dump in its profile folder; a freeze is a dump while the process lives **(inference)**. The exit code a crash returns is not documented **(needs a spike)**.

## 6. Findings and what they force

1. **Log silence missed a game whose clock stopped.** Forces a heartbeat per machine with game time, real time, frame number and frame rate, judged against thresholds, plus process and window checks from outside. Touches `CR-30`, `CR-27`[^f002][^hdoc].
2. **A health failure must not read as a check failure.** Forces every check that overlaps an unhealthy interval to be `invalid`, with the reason and each machine's last heartbeat in the result. Touches `CR-31`, `CR-51`, `G3`[^f002][^prd].
3. **A runner that dies leaves the game running, and the machine reboots without warning.** Forces every game process into a job object with kill-on-close, started suspended, and a run record the next run clears before it starts. Touches `CR-05`, `CR-07`[^jobs][^jobassign][^coord].
4. **The test machine's window, and maybe the machine, depend on the Shadow stream and its input.** Forces a health check that flags a stopped clock on a rendering client, sessions of only a server and headless clients where a check needs no rendering client, and a stated limit that rendering checks need the owner's stream. Touches `CR-30`, `CR-41`, `CR-25`[^shadow][^r14].
5. **The environment fails in a few known ways: Steam not running or not logged in, a mod missing or updated, a missing addon that hangs a server.** Forces a pre-run check that names what is missing before any game starts, and a server start that watches for its warning instead of waiting out a ceiling. Touches `CR-06`, `CR-31`[^tmanual][^steamreg][^meta][^r14].
6. **Crashes and freezes leave dumps in the profile folder.** Forces one profile folder per process start and a result that names any dump a process left. Touches `CR-24`, `CR-31`[^crash][^startup].
7. **`-noPause` is required on every rendering client.** Without it a client that loses focus pauses and its clock stops. Forces `-noPause` and `-noPauseAudio` into every client start. Touches `CR-01`, `CR-30`[^startup][^time].

## Needs a spike

- **Health thresholds.** Pass condition: over 50 passing checks in singleplayer and dedicated, record each machine's ratio of game time to real time, frame rate and heartbeat gap; set thresholds that none of the 50 crosses; then a stream cut, a frozen client (process suspended) and a killed client are each flagged `invalid` within 15 s, in 5 of 5.
- **Kill-on-close.** Pass condition: the runner starts a dedicated session (server, 2 clients, 1 headless client) inside a kill-on-close job and is itself killed with `taskkill /F`; within 10 s no `arma3_x64` or `arma3server_x64` process of that run is left, in 5 of 5, and the next run clears its mission copies.
- **Crash exit code and dumps.** Pass condition: a client is crashed on purpose (an extension that faults, or an access violation through a debug path) and the exit code, any `.mdmp` or `.bidmp` in its profile folder, and the RPT's last lines are recorded; a crash is told from a stop by the runner in 5 of 5.
- **A fault dialog.** Pass condition: with `JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION` set, a crashed game process ends without a dialog waiting on the desktop.
- **Shadow without the stream.** Pass condition: with the stream closed and no input, a session of a server and headless clients runs 60 minutes and its clock moves; the time until Shadow shuts the machine down, if it does, is recorded.

[^startup]: https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters (oldid 378577)
[^servercfg]: https://community.bistudio.com/wiki/Arma_3:_Server_Config_File (oldid 379176)
[^time]: https://community.bistudio.com/wiki/time (oldid 379479)
[^tick]: https://community.bistudio.com/wiki/diag_tickTime (oldid 379413)
[^fps]: https://community.bistudio.com/wiki/diag_fps (oldid 379331)
[^focus]: https://community.bistudio.com/wiki/isGameFocused (oldid 309152)
[^userinfo]: https://community.bistudio.com/wiki/getUserInfo (oldid 378191)
[^cstate]: https://community.bistudio.com/wiki/getClientStateNumber (oldid 348456)
[^crash]: https://community.bistudio.com/wiki/Crash_Files
[^pipe]: https://community.bistudio.com/wiki/Arma_3:_Named_Pipe (oldid 378557)
[^meta]: https://community.bistudio.com/wiki/Arma_3:_meta.cpp (oldid 373435)
[^jobs]: https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects (ms.date 2025-07-14)
[^joblimit]: https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information
[^jobassign]: https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject
[^hung]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-ishungappwindow
[^steamreg]: https://gamedev.stackexchange.com/questions/167176 (answer 167219, read through the Stack Exchange API)
[^shadow]: https://support.shadow.tech/hc/en-us/articles/32731841405073-Shadow-PC-and-the-Automatic-Shutdown (updated 2026-07-23)
[^t1]: research/10-prior-harness.md
[^hdoc]: arma-grappling-hook@3f4f7bd:docs/harness.md
[^hrun]: arma-grappling-hook@3f4f7bd:harness/run.ps1
[^f002]: arma-grappling-hook@3f4f7bd:acceptance/findings/m00/002-a-player-who-lets-go-can-stay-hanging-in-the-air-instead-of-falling.md
[^areadme]: arma-grappling-hook@3f4f7bd:acceptance/README.md
[^tclock]: arma-grappling-hook@3f4f7bd:tests/m00-let-go-clock/README.md
[^cov]: arma-grappling-hook@3f4f7bd:acceptance/coverage/m00.md
[^coord]: arma-grappling-hook@3f4f7bd:coordinator.md
[^m01]: arma-grappling-hook@3f4f7bd:docs/milestones/m01.md, section on the gates
[^r14]: arma-grappling-hook@3f4f7bd:research/14-test-automation.md
[^tmanual]: arma-grappling-hook@3f4f7bd:research/tests/manual-smoke/README.md
[^prd]: spec/client-requirements.md
