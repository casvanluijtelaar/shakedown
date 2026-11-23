---
type: Research
title: Best of breed
description: For each capability the framework needs, which tool or idea it takes, which it rejects and why, with the license that decides whether code can be reused.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T23:30:00+01:00 }
sources:
  - { id: harness, resource: "research/10-prior-harness.md", read: 2026-11-04 }
  - { id: tools, resource: "research/11-arma-tools.md", read: 2026-11-04 }
  - { id: remote, resource: "research/12-remote-execution.md", read: 2026-11-04 }
  - { id: reload, resource: "research/13-hot-reload.md", read: 2026-11-04 }
  - { id: sessions, resource: "research/14-sessions.md", read: 2026-11-04 }
  - { id: input, resource: "research/15-player-input.md", read: 2026-11-04 }
  - { id: observe, resource: "research/16-observation.md", read: 2026-11-04 }
  - { id: health, resource: "research/17-health.md", read: 2026-11-04 }
  - { id: testtools, resource: "research/18-test-tools.md", read: 2026-11-04 }
  - { id: agent, resource: "research/19-agent-tools.md", read: 2026-11-04 }
  - { id: legal, resource: "research/20-licenses-and-rules.md", read: 2026-11-04 }
  - { id: pain, resource: "research/21-mod-developer-pain.md", read: 2026-11-04 }
---

# Best of breed

No existing tool can be the base of the framework. The Arma tools closest to it are days to weeks old, and the two nearest, Neefay's harness and headswe's MCP server, have no license file, so they give ideas and no code[^tools]. No public source shows an in-game acceptance runner for Arma mods[^pain]. The framework is built from documented engine interfaces only: SQF, `callExtension` with its callback, startup parameters and server configuration. Hooking or scanning the engine is out, by the EULA and Steam's terms[^legal].

Each row below names the capability, what the framework takes and from where, what it rejects, and why. The topic file behind each row holds the sources and the spikes.

## The shape these choices force

Three parts, plus thin front ends[^remote][^agent][^testtools]:

1. **A framework addon** loaded on every machine: a thin native extension and an SQF agent. The extension holds a localhost connection to the runner. The agent receives orders through the `ExtensionCallback` event, runs them, and returns results through `callExtension`, split into chunks.
2. **A runner**: a long-lived process on the test machine that owns the game processes in a Windows job object, the per-run token, the ceilings, the health watchdog and the invalid verdict. It survives the agent session that started it.
3. **The check API in SQF**: steps, waits, assertions and fixtures, run by the agent inside the game. It owns the steps and the assertions, the way Unreal's Gauntlet splits an outer runner from an in-game controller.
4. **A command line and an MCP server**, both thin clients of the runner, with the same JSON results.

## Capabilities

| Capability | CR | Take | From | Reject | Why |
|---|---|---|---|---|---|
| Channel into the game | CR-10, CR-27 | A native extension with `callExtension` and `ExtensionCallback`, a background thread on localhost, results in chunks under the 10240-byte limit | Engine docs[^remote]; the shape of headswe's TCP line protocol and chunking, as an idea[^tools] | Intercept (patched per game build); ArmaDebugEngine (engine hooks, a license that forbids adaptation); the named pipe and `serverCommand` (no SQF); the debug console (a UI for a person) | The only documented two-way channel that carries any SQF and returns data on every machine type, Linux servers included[^remote] |
| Writing the extension | CR-10 | Rust with `arma-rs`, or C/C++ with the engine's header | `arma-rs` 1.13.0[^remote] | Logic inside the extension | The extension cannot be reloaded on the stable branch (`freeExtension` is dev only), so it stays thin and the logic lives in the runner and in SQF[^remote]. `arma-rs` is MIT on crates.io and GPL-2.0 in its repository's LICENSE file[^legal]; the owner chose `arma-rs` |
| Reaching a chosen machine | CR-10, CR-22 | A connection per machine to the runner; `remoteExec` with correlation ids for steps that cross machines inside a check | Engine docs[^remote]; Neefay's server hub, as an idea[^tools] | A result that does not say which machines it reached | A result names the machines reached and answered, so a missing machine reads invalid, not failed[^tools] |
| Sessions | CR-01, CR-02 | One long mission per session that takes checks while it runs; `-autotest` only to start singleplayer unattended | The prior harness's measurements[^harness][^sessions] | A mission per check; mission changes with connected clients | A start costs 18 to 26 s and no start option helps; mission changes under connected clients broke about one time in two[^harness] |
| Reset between checks | CR-02 | Track what a check creates with the `EntityCreated` event and delete it; restore players, time and weather; turn off vehicle randomisation | Engine docs[^sessions][^input] | An engine reset command (none exists) | Needs a spike: 20 checks per session, each reset under 2 s[^sessions] |
| Several clients on one machine | CR-01, CR-41 | `loopback`, `kickDuplicate = 0`, kick timeouts at 0, a port block per server, a profile folder and a neutral `-name` per process, every process started from the installed binaries | The prior harness[^harness][^sessions]; the EULA reading[^legal] | Copying the game executable | The EULA grants one copy; dedicated servers are allowed[^legal] |
| Hot reload | CR-03 | Three levels: in mission (checks, non-final functions), mission restart (CBA `PREP` with caching off), game restart (new files, config, the extension). The runner picks the lowest level a change needs and reports it | CBA, ACE and HEMTT practice[^reload] | Touching the mod's source to enable recompiling | A HEMTT dev build with file patching is the base; checks are always sent fresh and compiled with plain `compile`[^reload] |
| Building a scene | CR-11 | Scene steps run on the unit's owner, direction before position; time and weather on the server | Engine docs[^input] | | `setDir` works only where the unit is local[^input] |
| Using a mod's actions | CR-12 | `addAction` entries through `actionIDs`/`actionParams` and `action ["User", ...]` on the acting client; ACE interactions through the steps ACE's own key-up code takes, pinned to an ACE version | Engine and ACE docs[^input] | | ACE has no public way to perform an action by path[^input] |
| Pressing a mod's keys | CR-12 | Calling CBA keybind handlers, at two levels: the public handlers (as the prior harness does) and CBA's key dispatch | CBA source[^input]; the prior harness[^harness] | | Works on every machine with no window focus |
| Real key input | CR-15 | Nothing for the MVP | | `SendInput` (Zihuange's approach) | `SendInput` goes to the foreground window and breaks the rule against taking over the desktop. Posted window messages are a spike, with in-process injection as its fallback[^input][^tools] |
| Waits | CR-20 | Two ceilings: one inside the game on real time (`diag_tickTime`), one in the runner on the wall clock. A wait that ends on the runner's ceiling is invalid | Playwright's auto-waiting and polling, as the model[^testtools]; engine docs[^observe] | CBA's `waitUntilAndExecute` timeout alone | It counts mission time, which stops when the game pauses[^observe] |
| Timestamps and events | CR-21, CR-42 | Every record carries `systemTimeUTC`, `diag_tickTime`, game time and frame number; a recorder per machine, cleared between checks | Engine docs[^observe] | Game time alone | `time` differs per machine and jumps for a late joiner[^observe][^sessions] |
| Script errors | CR-23 | The `ScriptError` mission event (since 2.14), stamped with the current check and step id on each machine; the RPT scan as backup; `-debug` on | Engine docs[^observe]; Neefay's request ids, as an idea[^tools] | `-showScriptErrors` as a source | It only shows errors on screen[^observe] |
| Logs | CR-24 | A profile folder per machine per run, kept with the run, with Steam IDs and names redacted | Engine docs[^observe]; privacy reading[^legal] | The RPT as the result channel | Its last lines can be lost; results travel through the extension[^harness] |
| Health | CR-30 | A heartbeat per machine (game time against real time, frame rate), process and window checks from outside, a separate frame-rate floor for the server | Engine docs and the prior harness's findings[^health] | Log silence as the only signal | The logs grew while the clock stood still when the Shadow stream was cut[^harness][^health] |
| Outcomes | CR-31, CR-32 | Pass, fail and invalid, plus a flaky report on repeats | Playwright's flaky outcome[^testtools] | A plain fail for a stall, crash or timeout | The prior harness reported them all as fail[^harness] |
| Clean stops | CR-05 | Every game process started suspended, put in a job object that kills on close, then resumed; a run record so the next run clears what was left | Windows process docs[^health] | Starting the game as a child of the MCP server | Claude Code ends an MCP server's process tree when it stops; Neefay avoids this by starting the game outside it[^agent][^tools] |
| Environment check | CR-06 | Steam running and logged in, each mod present with its version, BattlEye off, crash dumps to tell a crash from a stop | Health research[^health] | | |
| Filters and setups | CR-40, CR-41 | Story and finding ids as tags; named setups like Playwright's projects; a check that must fail like `test.fail()` | Playwright[^testtools]; the prior harness's `-Checks`, `-Finding` and cases[^harness] | | |
| Check API | CR-50 | SQF functions for check, step, on a machine, wait until, expect, expect on every machine, expect an error, fixture, act and key; script errors fail a check unless it expects them | Playwright, Unity's LogAssert, CBA's `TEST_` macros, ArmaTesqf's wording (MIT), the prior harness's helpers[^testtools][^tools][^harness] | | Candidate names only; the spec decides |
| Result file | CR-51 | One JSON file per run with per-step records, evidence kept on failure | Maestro's per-step output, Playwright's trace on failure[^testtools] | | |
| Agent tools | CR-53, CR-60 | An MCP server over stdio from the project's `.mcp.json`; short calls, suite runs as a handle to poll, a failed check returned as a result and not an MCP error, structured output; loopback and a per-run token to the game | The MCP spec, revision 2026-07-28[^agent]; the Arma MCP servers' common core, as ideas[^agent][^tools] | HTTP transport for the MVP | Stdio limits access to the launching client and needs no OAuth[^agent] |
| Screenshots | CR-25 | The `screenshot` command; an in-process window capture as the fallback | Engine docs; Zihuange's capture (MIT)[^observe] | A Direct3D hook | Hooking the renderer runs into the EULA[^legal]. `screenshot` may fail silently with `-profiles`: needs a spike[^observe] |
| Performance | CR-26 | `diag_fps`, `diag_codePerformance` with the debug console on | Engine docs[^observe] | | `diag_codePerformance` runs one cycle in multiplayer without the debug console[^observe] |

## Profiling and development branch

Nothing the framework needs requires them. Stable 2.22 covers every recommendation above[^remote][^observe][^health]. Recorded and not used:

- `freeExtension`, which would let the extension reload: development branch.
- `diag_mergeConfigFile`, which would reload config: development branch, and removed from multiplayer.
- `diag_captureFrame` and the server's profiling commands: profiling branch.
- `RVExtensionLoadFile` and the `ParachuteLanded` event: 2.24, development branch for now.

## Code that may be reused

MIT, with the notice kept and the source recorded: a3-bridge, Zihuange/arma3-mcp, a3ref, Intercept, Pythia, ArmaTesqf, and CBA's `keybinding` and `settings` addons[^legal]. CBA and ACE3 are GPL: the framework loads and calls them and copies nothing. HEMTT is GPL-2.0: the framework runs it as a tool. No code is copied from tools with no license file or a license that forbids reuse: Neefay, headswe, arma-ws, ArmaDebugEngine, Arma-Remote-Code-Executor (GPL-3.0)[^legal]. Their ideas are read and implemented in the framework's own code, which the owner allows (`spec/client-requirements.md`, section 7).

[^harness]: research/10-prior-harness.md
[^tools]: research/11-arma-tools.md
[^remote]: research/12-remote-execution.md
[^reload]: research/13-hot-reload.md
[^sessions]: research/14-sessions.md
[^input]: research/15-player-input.md
[^observe]: research/16-observation.md
[^health]: research/17-health.md
[^testtools]: research/18-test-tools.md
[^agent]: research/19-agent-tools.md
[^legal]: research/20-licenses-and-rules.md
[^pain]: research/21-mod-developer-pain.md
