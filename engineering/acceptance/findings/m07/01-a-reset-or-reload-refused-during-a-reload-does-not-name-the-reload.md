---
type: Finding
id: m07-1
milestone: M7
title: A reset or reload refused during a reload says "a reload or a reset" instead of naming the reload
description: shakedown reset, shakedown reload and the reload tool asked while a reload applies are refused with exit 4 and session-busy, but each message says the session is busy with "a reload or a reset", not which one is in progress, as RLD-01 asks.
status: draft
raised_by: test
for: developer
checks:
  - "US-058 RLD-01 NFR-US-01 the refusal of a reset, a reload and the reload tool during a reload names the reload in progress, not a reload or a reset on a dedicated session with two clients"
  - "US-058 RLD-01 NFR-US-01 the refusal of a reset, a reload and the reload tool during a reload names the reload in progress, not a reload or a reset in singleplayer"
stories: [US-058]
generated: { by: "claude-code/tester", at: 2026-11-11T12:12:06-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-11T13:12:54-08:00, note: "The session state reloading is shared by a reload and a reset, and the refusals named only that state. The session records what it is busy with as it enters reloading (reload <id> (level <level>), or a reset), and the refusals of reset, reload, the reload tool and run name it; RLD-01, spec/contract/cli.md and docs/reloading.md say a reload is named by its id and level. Proved by m07 us_054_us_058_nfr_pf_02_reload_during_a_run_and_rerun_by_finding_in_singleplayer and _on_a_dedicated_session, which ask all three during a game-restart reload; both named checks passed through the filter, 2 of 2." }
verified:
  - { by: "claude-code/tester", at: 2026-11-11T13:28:55-08:00 }
---

# A reset or reload refused during a reload says "a reload or a reset" instead of naming the reload

## What the spec promises

`RLD-01`: "A `reload` or a `reset` asked while a run, a reload or a reset is in progress is refused, naming what is in progress and, for a run, its progress and `run stop`." `US-058` says the same in its description: "refused, saying what is in progress". `docs/reloading.md`: "A reload or a `reset` is refused while a run, another reload or a reset is in progress, exit code 4 (the tools: `session-busy`), naming what is in progress".

## What the system does

On a ready session of setup `rld7` (a dedicated server with two clients, `@skd_fixture` watched in a staged copy with a `rebuild`) and of `rld7sp` (singleplayer), the staged `fn_version.sqf` is changed and `shakedown reload --json` is started. While `status` shows the session `reloading`, three requests are made. Each is refused, as it should be, but none names what is in progress:

```
$ shakedown reset --json
Session s-20261111-200205 is busy with a reload or a reset, so it cannot be reset now. Wait for it to be ready.
(exit 4)

$ shakedown reload --json
Session s-20261111-200205 is busy with a reload or a reset. Wait for it to end; `shakedown status` shows it.
(exit 4)

reload tool, {}:
isError: session-busy: Session s-20261111-200205 is busy with a reload or a reset. Wait for it to end; `shakedown status` shows it.
```

During a run the same requests name the run, its progress and `shakedown run stop`, as promised ("Session ... is running run r-... (0 of 2 checks done, now at m07/US-058-slow), so it cannot reload now. ...").

## What it costs

Little. A mod developer who asks for a reload while another one, started from another window or by an agent, is restarting the game is told only that the session is busy with one of two things, and has to run `shakedown status` to learn that a game restart is under way and how long to wait. The refusal itself works, and nothing under the run or reload is touched.

## The check that holds it

`US-058 RLD-01 NFR-US-01 the refusal of a reset, a reload and the reload tool during a reload names the reload in progress, not a reload or a reset`, once in the `rld7` session and once in `rld7sp`. It failed in every repeat of the M7 round.
