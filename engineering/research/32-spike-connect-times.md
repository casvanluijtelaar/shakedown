---
type: Research
title: "Spike SPK-20: the time from launch to connected"
description: "How long each kind of game process takes from its launch to its extension's connect, over 20 singleplayer starts and 20 dedicated starts with CBA, ACE and the fixture on a Windows machine in M16, why a connect ceiling of twice the slowest does not fit under 90 s, and a singleplayer start that hung in the engine's graphics start and held the machine for the whole start ceiling."
status: stable
generated: { by: "claude-code/developer", at: 2026-11-19T21:42:48Z }
sources:
  - { id: script, resource: "A scratch script of the M16 developer round that ran shakedown session start and session stop in a loop against a project with CBA_A3, ACE3 and @skd_fixture, and read each machine's 'started process' and 'extension connected' lines from runner.log", read: 2026-11-19 }
  - { id: rpt, resource: "The RPT of each singleplayer start of that series, in its session folder", read: 2026-11-19 }
  - { id: def28, resource: "engineering/plan/deferred/28-hung-client-waits-the-start-ceiling.md", read: 2026-11-19 }
---

# Spike SPK-20: the time from launch to connected

M16 ran `SPK-20` (`engineering/spec/06-open-questions.md`) on a Windows machine before building the connect ceiling of `SES-10`. Its pass condition: over 20 starts per kind of machine, cold and warm, the slowest launch-to-`connected` time of each kind is known, and a connect ceiling of twice it is under 90 s. It failed on the singleplayer client, whose slowest healthy start connected 90.3 s after its launch. `SES-10` keeps the reason in `session.json` and has no connect ceiling; a hung client fails the start at the start ceiling of `SES-04`, and `DEF-28` stays open[^def28].

## How it was measured

A script[^script] ran `shakedown session start` and `shakedown session stop` in a loop, with the framework built for M16 (0.1.6), against a project loading CBA_A3 and ACE3 from the Workshop and `@skd_fixture`: the heaviest set of mods the framework's first user starts (`sp-cba-ace` in arma-grappling-hook). 20 singleplayer starts ran first, then 20 dedicated starts of a server, two game clients and a headless client. For each machine, the time is from the runner's `started process` line to its `extension connected` line in `runner.log`. The extension connects on the in-game agent's first call, in the session mission's `postInit`, so `connected` comes once the game has started, loaded the mission and run its `postInit`, a few seconds before `ready`.

The first singleplayer start ran after no game had run on the machine for a while, the coldest start the machine gives without a reboot; the dedicated starts all came after the singleplayer ones, so none of them is cold.

## Results

| Kind | Starts | Connected | Fastest | Median | Slowest | Twice the slowest |
|---|---|---|---|---|---|---|
| Singleplayer client | 20 | 19 | 31.4 s | 33.0 s | 90.3 s | 180.5 s |
| Dedicated server | 20 | 20 | 24.1 s | 24.7 s | 28.1 s | 56.2 s |
| Dedicated game client | 40 | 40 | 35.4 s | 36.6 s | 39.4 s | 78.8 s |
| Headless client | 20 | 20 | 29.3 s | 30.1 s | 35.0 s | 70.1 s |

A dedicated session was `ready` 62.5 to 68.9 s after `session start` began; a game client is launched only once the server is ready, so its time counts from then.

The singleplayer client fails the condition, and fails it even without its slowest start: the cold first start connected after 52.6 s, so twice it is 105 s. The three kinds of a dedicated session pass on their own, but the condition is for each kind, and these were warm starts only.

## Where a singleplayer start spends its time

The RPTs of the singleplayer starts[^rpt] show two phases. From launch to the line `DX11 - Initializing DX11 engine.` the game reads its own and the mods' addons: 11 s on a warm start, 29 s on the cold first start, 42 s on the 90 s start. From there to the line `Mission file:` it starts its graphics and loads the session mission: 16 s warm, 17 s cold, 40 s on the slow start. Both phases vary with the machine's file cache and its load, not with the framework.

## A start that hung

The second singleplayer start of the series never connected. Its RPT ends 30 s after the launch with `DX11 - Detecting best GPU adapter.` and nothing after it: the engine hung choosing its graphics adapter. The start failed at the start ceiling of 180 s, held the machine and its memory until then, and the next start worked. It is the hang `DEF-28` describes, met for the first time in a measured series, and the kind a connect ceiling would catch; with a healthy start taking up to 90 s to connect, a ceiling that never fails a healthy start cannot be much shorter than the start ceiling itself.

## What the spec does with it

- `SES-10` keeps only the reason of a failed start in `session.json`; it has no connect ceiling.
- `SES-04` names, at the start ceiling, the machine that got least far, one still `launched` before one `connected`, so the hung machine is the one the reason and its RPT tail name.
- `US-087`'s first criterion is the start ceiling.
- `DEF-28` stays open, with a fourth option: connecting the extension from a `preStart` function, at the end of the engine's start, so a connect ceiling would cover only that phase. It is measured again before any ceiling is set.

[^script]: The M16 developer round's measurement script, run 2026-11-19 from 21:03 to 21:42 UTC.
[^rpt]: The RPTs of sessions `s-20261119-210355` (cold), `s-20261119-210517` (slowest), `s-20261119-210650` (hung) and `s-20261119-211138` (warm).
[^def28]: `DEF-28`, a client that hangs in its start holds `session start` for the whole start ceiling.
