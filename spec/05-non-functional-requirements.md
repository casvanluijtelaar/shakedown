---
type: Specification
title: Non-functional requirements
description: The qualities Shakedown must have - speed, reliability, security, privacy, portability, legal limits and maintainability - each measurable and traced to the PRD's goals and requirements.
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# Non-functional requirements

Each requirement has an ID of the form `NFR-<area>-nn` and says how it is measured. Measurements are taken on the Windows test machine. A number marked "until `SPK-nn`" is the owner's accepted proposal, kept until that spike measures it and the number is set from the measurement.

## Performance

- **NFR-PF-01** (must) With a session `ready`, the time from editing a check to that check's outcome is under 30 s, for a check whose own body takes under 10 s, in singleplayer and on a dedicated session with two clients. In singleplayer, the time from saving a changed `.sqf` of a watched mod to the outcome of such a check, through a reload of level `mission-restart`, is under 30 s too (G2; conflict C2).
- **NFR-PF-02** (must) A `game-restart` reload of a changed mod file, from the reload order to every machine `ready` on the new code, takes under 90 s on a dedicated session with two clients on the Windows test machine, plus the time of the mod's own `rebuild`, and every reload result reports its time (`RLD-03`). The target is set from `SPK-08`'s measurement in M6: 10 reloads of a changed CBA `PREP` function of `@skd_fixture_cba` on a dedicated server with two clients, loading CBA_A3 3.19.0 and both fixtures on Arma 3 2.22, each with a `rebuild` that copies the one file (about 0.2 s), took 80.3 to 82.7 s, 81.8 s on average, by the reload result's `duration_s`; a session start of the same setup in that run took 82.4 s, the server's start (about 32 s) and then the clients' (about 47 s) (research `27-spikes-reload.md`). A `mission-restart` reload of a changed `.sqf`, from the reload order to every machine `ready` on the new code, takes under 20 s in singleplayer and under 90 s on a dedicated session with two clients on the Windows test machine, plus the time of the mod's own `rebuild`; the first such reload on a game folder also makes the release files once (`RLD-07`). These targets are set from M14's measurement: with CBA_A3 3.19.0 and both fixtures on Arma 3 2.22 and a `rebuild` that copies the one changed file, singleplayer reloads of a changed CBA `PREP` function took 7.9 to 9.7 s by `duration_s`, and from saving the file to a short check's outcome 10.2 to 12.3 s; on a dedicated session with two clients ten reloads took 50.7 to 52.0 s, 51.3 s on average, against a session start of 46.5 s in the same run, since the mission restart starts new clients (research `31-spike-reload-release.md`, `DEF-32`). G2's 30 s applies to a changed check, and in singleplayer to a changed `.sqf` (`NFR-PF-01`) (G2; conflict C2).
- **NFR-PF-03** (must) An order round trip (`P-03`) of a trivial expression to any machine takes under 1 s at the 95th percentile, over 100 trips per machine kind (`CR-10`; `SPK-01` measured under 45 ms on every machine kind, research `22-spikes-singleplayer-channel.md` and `24-spikes-dedicated.md`).
- **NFR-PF-04** (must) A reset in place takes under 2 s on a dedicated session with two clients and one headless client (`CR-02`; `SPK-04` measured at most 1.5 s over 63 resets, research `24-spikes-dedicated.md`).
- **NFR-PF-05** (must) The framework adds no frame over 100 ms to a game process while it moves orders and results, including a 1 MB order and a 1 MB result. Compiling the SQF an order carries is the engine's own work in one command and is outside this limit: a 1 MB source of short statements compiles in about 0.1 to 0.12 s, and a long string literal compiles in time that grows with the square of its length (research `22-spikes-singleplayer-channel.md`) (`CR-10`; `SPK-02`).
- **NFR-PF-06** (must) The default wait ceiling is 10 s, the default check timeout 120 s, the runner's grace 10 s, the start ceiling 180 s and the reset ceiling 15 s (`CR-20`; Q3; `SPK-14`, research `29-spike-default-ceilings.md`).

## Reliability

- **NFR-RL-01** (must) No environment failure is reported as `fail`. Over the MVP's acceptance runs, every check ended by a lost machine, a broken session, a failed reset or reload, or a stopped run is `invalid` (G3, `CR-31`).
- **NFR-RL-02** (must) A singleplayer session and a dedicated session with two clients each run 20 consecutive checks with a reset between each, with every check seeing the baseline state on every machine and no machine lost (`CR-01`, `CR-02`; `SPK-04`).
- **NFR-RL-03** (must) After the runner is killed with `taskkill /F`, no game process of its session is left within 10 s, in 5 of 5 tries, and the next runner start clears what the record lists (`CR-05`; `SPK-06`).
- **NFR-RL-04** (must) No order and no result is lost or delivered twice across a mission restart (`CR-03`; `SPK-01`).
- **NFR-RL-05** (must) Every script error the RPT shows from mission start on reaches the result, from the handler or from the RPT scan (`CR-23`; `SPK-07`).

## Security

- **NFR-SC-01** (must) Every socket the framework opens to listen, in `shakedown.exe`, in the extension and in the framework's own tests and helpers, binds `127.0.0.1` only, and no part of the framework binds another address even for a moment, to probe a port: a port is checked free by reading the system's socket tables, not by binding it. Windows Firewall asks about a program only when it listens on an address other than loopback, so `shakedown.exe` and the test binaries raise no firewall prompt. Verified with `netstat` during a session and with the system's socket tables. The game ports of a dedicated session are opened by the game's own `arma3server_x64.exe` and `arma3_x64.exe` and are the game's, outside this rule (`CR-60`).
- **NFR-SC-02** (must) An order sent to the runner or an extension without the right token, or from a process the runner did not start, runs no code and is logged (`CR-60`).
- **NFR-SC-03** (must) No token appears on any process's command line, in the result file, in a log the framework writes, or in tool output, verified by searching each of them for the token pattern of `SEC-06` during and after a session (`CR-60`).

## Privacy

- **NFR-PV-01** (must) No Steam ID appears in the result file, the run folder's logs, tool output or a stopped session's RPTs (`CR-24`, `CR-51`; gap H8).
- **NFR-PV-02** (must) Every process's profile name is its machine name, so no person's name is written by the framework's own processes, and no Steam name, neither the account name nor the persona name of the logged-in account, appears in the result file, the run folder's logs, tool output or a stopped session's RPTs (`LOG-02`), verified by searching each of them for both names during and after a session (`CR-24`, `CR-51`; gap H8).
- **NFR-PV-03** (must) Session and run folders are ignored by git (`CFG-02`) (gap H8).

## Usability

- **NFR-US-01** (must) Every error message, command line output and tool error names what went wrong and the next thing to do, in one or two sentences, without a stack trace of the framework (`CR-53`).
- **NFR-US-02** (must) A test author writes a check from the check API contract and the docs alone, without reading the framework's source (`CR-50`).
- **NFR-US-03** (must) The command line's `--json` output and the MCP tools' structured content are the same JSON for the same request (`CR-53`).

## Portability

- **NFR-PT-01** (must) The runner, command line, MCP server and extension build for 64-bit Windows. The extension and every part that needs no Windows API also build for 64-bit Linux, so a Linux dedicated server can come later without a rewrite (gap H10).
- **NFR-PT-02** (must) `list`, `status`, `guide` and the MCP server's start work on macOS and Linux with no game, so an AI agent on the Mac can read checks and the contract (`CR-53`).

## Legal

- **NFR-LG-01** (must) The framework uses only documented engine interfaces: SQF, `callExtension` and its callback, start options and server configuration. It never hooks, patches or scans the game's memory or files (PRD section 7; research `20-licenses-and-rules.md`).
- **NFR-LG-02** (must) Every game process runs from the one installed copy of the game, with BattlEye off, on the local machine only (PRD section 7).
- **NFR-LG-03** (must) No code is copied from CBA, ACE3, HEMTT or any project without a license compatible with GPL-2.0-or-later. Code taken under such a license records its source and license in the file it lands in (PRD section 7).
- **NFR-LG-04** (should) The framework is licensed GPL-2.0-or-later: the repository's `LICENSE` holds the GPL version 2 text with the grant of later versions, and every crate of the workspace declares `license = "GPL-2.0-or-later"`. Every release zip carries `LICENSE` and `THIRD-PARTY-LICENSES.txt`, which holds the licence text, copyright notice and any `NOTICE` file of every package compiled into `shakedown.exe` or `shakedown_x64.dll`. A dependency is added only under a licence that lets the binary it lands in be distributed under GPLv2 or GPLv3; `shakedown.exe` holds `rmcp`, Apache-2.0 only, and so is distributed under GPLv3 (PRD section 7, `CR-70`; research `34-release-and-licence.md`).

## Maintainability

- **NFR-MT-01** (must) The extension holds no test logic: every behaviour that can change lives in the runner or in SQF, so a framework change needs no game restart unless it changes the extension's calls (`CR-03`).
- **NFR-MT-02** (must) Every test and check that proves a requirement or story carries its ID in its name (CLAUDE.md).
- **NFR-MT-03** (must) The framework's own tests that do not start the game run on GitHub Actions on every push, with the bundle check (`07-tech-stack.md`).
