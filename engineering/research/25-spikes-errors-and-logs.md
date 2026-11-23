---
type: Research
title: "Spikes SPK-07 and SPK-13: script errors and live RPTs"
description: "What the ScriptError mission event caught on a singleplayer p1, a dedicated server, a game client and a headless client for every kind of code against what each RPT showed, and how soon a reader beside a running game sees a diag_log line in its RPT, on the test machine in M3, with the RPT's error format and what taking the value of code that ends in an assignment does, found on the way."
status: stable
generated: { by: "claude-code/developer", at: 2026-11-06T08:30:00+01:00 }
sources:
  - { id: spikes, resource: "crates/shakedown/tests/spikes_m03.rs, spk_07_spk_13_dedicated_server_client_headless, spk_07_spk_13_singleplayer_p1 and engine_fact_the_value_of_code_ending_in_an_assignment, two runs of 2026-11-06 on the Windows test machine, Arma 3 2.22.154089 stable", read: 2026-11-06 }
---

# Spikes SPK-07 and SPK-13: script errors and live RPTs

M3 ran `SPK-07` and `SPK-13` (`spec/06-open-questions.md`) on the Windows test machine, Arma 3 2.22.154089 stable, once in a dedicated session the runner started (a server, two game clients `p1` and `p2`, a headless client `hc1`) and once in a singleplayer session, each loading the fixture `@skd_fixture` with its `skdfx_fnc_throw`. The owner was connected through Shadow PC, so the client windows rendered. Both run again after `python tools/install.py` with `cargo test -p shakedown --test spikes_m03 -- --ignored --nocapture --test-threads=1`.

## SPK-07: passed

On each machine the spike added a `ScriptError` mission event handler of its own that kept every call, then raised errors one kind of code at a time, reading after each how many the handler had caught, and compared the total with the script errors the machine's RPT showed between two marker lines, found by the rule of `INT-AR-07`[^spikes].

| Code | Error | `server` | `p1` | `p2` | `hc1` | singleplayer `p1` |
|---|---|---|---|---|---|---|
| `[] spawn skdfx_fnc_throw` (spawned, a function's file) | undefined variable | 1 | 1 | 1 | 1 | 1 |
| `[] remoteExec ["skdfx_fnc_throw", owner]` from the lead | undefined variable | 1 | 1 | 1 | 1 | 1 |
| `[] spawn { ... }` (spawned inline code) | undefined variable | 1 | 1 | 1 | 1 | 1 |
| A one-shot `EachFrame` handler (unscheduled) | type error, `"skd" + 1` | 1 | 1 | 1 | 1 | 1 |
| `isNil { ... }` (unscheduled) | type error | 1 | 1 | 1 | 1 | 1 |
| `remoteExecCall ["call", owner]` of inline code from the lead (unscheduled) | type error | 1 | 1 | 1 | 1 | 1 |
| `isNil`, `EachFrame` or `remoteExecCall` running `skdfx_fnc_throw` (unscheduled) | undefined variable | 0 | 0 | 0 | 0 | 0 |
| **Handler total / RPT errors** | | 6 / 6 | 6 / 6 | 6 / 6 | 6 / 6 | 6 / 6 |

Every error the RPTs showed was caught by the handler on the machine where it ran, on every machine kind; the handler missed none. The handler fired at once, in the same frame as the error, with:

- the text without the `Error ` prefix, `Undefined variable in expression: _skdfx_undefined` or `Generic error in expression`, which is also the text of the RPT's indented `Error <message>` line;
- the file as the RPT names it, `skdfx\addons\main\functions\fn_throw.sqf`, without a leading backslash, and the line of the error in that file (4); for inline code the file is empty and the line counts from the code's start;
- a stack in `diag_stacktrace` form, one frame per scope, each `[file, line, scope name, variables]`, outermost first: a `remoteExecCall` of inline code gave two frames, the outer without variables. The variables hold whatever the scope held, so the framework does not pass them on.

**An undefined variable in unscheduled code is no error at all**: the engine writes nothing to the RPT and raises no `ScriptError`, so there is nothing to catch; a type error in the same unscheduled code is reported both ways. `skdfx_fnc_throw` raises its error in scheduled code (`spawn`, `remoteExec`, and the code a check sends to a machine, which runs scheduled, `CTL-03`).

**Forces:** `ERR-01` stands as written, and the RPT scan of `ERR-02` stays a second source for what the handler cannot see, such as a line written to the RPT by other means; `SPK-07`'s fallback (errors attributed by time only) is not needed on any machine kind. The fixture contract says that `skdfx_fnc_throw` raises nothing when called in unscheduled code.

## SPK-13: passed

On each machine the spike sent 100 orders in a row, each writing one `diag_log` line with a number of its own, and after each a reader opened the machine's RPT anew, read it from where it ended before the order, and looked for the line every 10 ms, with a ceiling of 2 s[^spikes].

| Machine | Seen within 2 s | Median | 95th percentile | Max |
|---|---|---|---|---|
| `server` | 100 of 100 | 498 ms | 507 ms | 510 ms |
| `p1` (client) | 100 of 100 | 500 ms | 508 ms | 510 ms |
| `p2` (client) | 100 of 100 | 501 ms | 507 ms | 509 ms |
| `hc1` (headless client) | 100 of 100 | 500 ms | 508 ms | 513 ms |
| singleplayer `p1` | 100 of 100 | 501 to 507 ms | 506 to 521 ms | 509 to 524 ms |

The times run from the order's dispatch to the reader seeing the line. The engine writes its RPT in blocks about every 0.5 s, so a line reaches the file within about half a second of the `diag_log` call. No RPT held a line about a file it could not open or write, with the reader opening it 100 times beside the game.

**Forces:** `logs` and the RPT scan read a running machine's RPT (`LOG-03`, `INS-03`); `SPK-13`'s fallback is not needed. A runner that waits for a marker line in an RPT gives it a ceiling of a few seconds.

## The RPT's script error format

On 2.22 with `-debug`, a script error takes three to four timestamped lines, each `h:mm:ss` local time and one space[^spikes]:

```
 7:15:18 Error in expression <private _skdfx_sum = _skdfx_undefined + 1;
 7:15:18   Error position: <_skdfx_undefined + 1;
 7:15:18   Error Undefined variable in expression: _skdfx_undefined
 7:15:18 File skdfx\addons\main\functions\fn_throw.sqf..., line 4
```

The quoted code of the first two lines runs on over lines without a timestamp when it holds line breaks. The indented `Error <message>` line is the handler's text; the `File` line comes only for code from a file. The times are local and to the second, so the framework turns an RPT time into UTC from a marker line of its own that carries both (`LOG-04`).

## Code that ends in an assignment

An assignment returns nothing, so the value of code whose last statement is one, such as `{ skd_a = 1 }`, is nothing. Taking that value raises an error and ends the script there: `private _r = [] call { skd_a = 1 }` raises `GIAS pre stack size violation`, `_b set [0, ...]` raises `GIO pre stack size violation` and `[...] param [0]` raises `GIAR pre stack size violation`, while the same call as a statement of its own raises nothing[^spikes]. Wrapped in a `switch`, `private _r = switch (true) do { default { [] call { skd_a = 1 } } }` raises nothing and leaves `_r` nil, and code that returns a value returns it unchanged. The framework took the value of a step's code this way before M3, so a step that ended in an assignment raised that error unseen; it takes it through a `switch`.

[^spikes]: crates/shakedown/tests/spikes_m03.rs, spk_07_spk_13_dedicated_server_client_headless, spk_07_spk_13_singleplayer_p1 and engine_fact_the_value_of_code_ending_in_an_assignment, two runs of 2026-11-06 on the Windows test machine, Arma 3 2.22.154089 stable
