---
type: Research
title: "Spikes SPK-01, SPK-02 and SPK-03 in singleplayer"
description: "What the round trip through the extension and its callback, large orders and results, and the session token through the environment measured on singleplayer p1 on the test machine in M0, with the engine facts the walking skeleton found on the way."
status: stable
generated: { by: "claude-code/developer", at: 2026-11-05T17:30:00+01:00 }
sources:
  - { id: spikes, resource: "crates/shakedown/tests/spikes.rs, runs of 2026-11-05 on the Windows test machine (sessions s-20261105-161417, s-20261105-161619, s-20261105-161740 and one after)", read: 2026-11-05 }
  - { id: probe, resource: "An -autotest mission of the developer's scratch folder logging toJSON, fromJSON, compile, ScriptError and terminate behaviour with diag_log, 2026-11-05, Arma 3 2.22.154089 stable", read: 2026-11-05 }
  - { id: e2e, resource: "crates/shakedown/tests/e2e.rs, us_037_us_038_checks_in_one_session, 2026-11-05", read: 2026-11-05 }
  - { id: rpt, resource: "RPT files of the spike and probe sessions on the test machine, 2026-11-05", read: 2026-11-05 }
---

# Spikes SPK-01, SPK-02 and SPK-03 in singleplayer

M0 ran the singleplayer `p1` parts of `SPK-01` and `SPK-03` and the whole of `SPK-02` (`spec/06-open-questions.md`) on the Windows test machine, Arma 3 2.22.154089 stable, with the extension built on `arma-rs` 1.13.0 and the in-game agent of `addon/`. The game ran in a window with no person connected through Shadow PC. Each spike runs again with `cargo test -p shakedown --test spikes -- --ignored --nocapture` after `python tools/install.py`[^spikes].

## SPK-01, singleplayer p1: passed

The runner sent `1 + 1` as an `exec` order to `p1` 100 times in a row, three times in three sessions[^spikes].

| Session | Answered `2` | Lost | Median | 95th percentile | Max |
|---|---|---|---|---|---|
| 1 | 100 | 0 | 13.4 ms | 17.5 ms | 131.3 ms |
| 2 | 100 | 0 | 1.0 ms | 1.4 ms | 3.7 ms |
| 3 | 100 | 0 | 1.0 ms | 1.3 ms | 1.6 ms |

No RPT line names a 301 warning; the only line naming the extension is `CallExtension loaded: shakedown (...\@shakedown\shakedown_x64.dll) [0.0.1]`[^rpt]. The callback fires in a singleplayer mission and the `ExtensionCallback` handler registered in `postInit` receives it, so `p1` needs no fallback.

**Forces:** `P-03` stands as written for `p1`. The part across a mission restart runs in M6, the server, client and headless client parts in M2.

## SPK-02: the channel passed; compiling a large source is the engine's cost

| What moved | Size | Arrived | Time | Longest frame | Longest frame with nothing moving |
|---|---|---|---|---|---|
| An order carrying a string argument, fetched in parts | 1,048,578 bytes, 174,763 quotes | intact: length, quote count, first and last 12 characters | 91 ms and 922 ms (the second run counted the quotes in SQF) | 11 ms and 41 ms | 43 ms and 47 ms |
| A result string, sent in one `send` | 1,572,864 characters | intact, compared in full | 88 ms and 97 ms | 20 ms and 15 ms | as above |
| A source of 80,660 short statements, fetched and compiled | 1,048,598 bytes | value `80660` | 241 ms and 222 ms | 119 ms and 101 ms | as above |

Fetching in parts of 10,000 bytes, joining the parts, `fromJSON`, `toJSON` and `send` cost a few milliseconds at a megabyte and kept every frame under 100 ms[^spikes]. The frame over 100 ms in the third row is one `compile` of a 1 MB source, a single engine command the framework cannot split.

`compile` grows with the square of the length of one string literal in the source[^spikes]:

| Source holding one string literal of | `compile` takes |
|---|---|
| 7 KB | 0.002 s |
| 29 KB | 0.031 s |
| 115 KB | 0.45 s |
| 229 KB | 1.8 s |
| 459 KB | 7.1 s |
| 1.2 MB | over 30 s: the game logged `No alive in 10000 ms` and wrote `FROZEN` dumps, and the order met its runner ceiling[^rpt] |

`fromJSON` of the same JSON took under 2 ms at every size[^spikes].

**Forces:** the part size of `INT-AR-03` stays at 10,000 bytes, because the channel holds the frame limit; lowering it would not shorten a `compile`. `NFR-PF-05` covers what the framework moves; the time to compile a check's own body is the engine's and is written into the check API contract as a limit a test author can see: keep long data out of string literals in a check.

## SPK-03, singleplayer p1: passed

The game process started by the runner read `SHAKEDOWN_RUNNER`, `SHAKEDOWN_SESSION`, `SHAKEDOWN_MACHINE` and `SHAKEDOWN_TOKEN` from its environment: its extension's hello carried the session id, the machine name and the session token, and the runner accepted it[^spikes]. The game's command line, read through `Win32_Process`, holds no `skd_s_` token[^spikes].

**Forces:** `SEC-02` stands as written for `p1`. The other machine kinds run in M2.

## Engine facts found on the way

- **`compile` takes no comments.** Neither `//` nor `/* */` compiles: both give `Invalid number in expression` at the comment[^probe]. The runner therefore turns every comment of a check, its header included, into spaces before it sends the body, keeping line breaks, so the engine's line numbers are the file's.
- **A compile error is reported only through `ScriptError`.** `compile "1 +"` returns `{1 +}`, not nil, and raises the `ScriptError` mission event twice with the text, an empty file name, the line within the compiled text and the position[^probe]. The agent records the errors raised while it compiles code it was sent; that is the engine's compile message of `US-038`, such as `Invalid number in expression, at line 4 of the check file`[^e2e].
- **`toJSON` keeps 6 significant digits**: `123456789` becomes `1.23457e+08`, and `1048578` arrives as `1048580`[^probe][^spikes]. SQF numbers are 32-bit floats, and `toFixed` keeps their full value (`123456789.25 toFixed 3` is `123456792.000`). The agent sends the clocks of `OBS-05` as text with fixed decimals, and the runner reads them as numbers.
- **`toJSON` writes an object as `null`**, a hashmap as an object, and nil as `null`; `fromJSON` returns a hashmap for an object, and `null` arrives as nil[^probe]. The agent encodes values by `DM-13` before `toJSON`.
- **`terminate` does not stop a script until it next suspends**: a script that terminated itself ran its loop on to its end[^probe]. A check API function that fails therefore terminates the body and then waits with `waitUntil { false }`, so it never returns.
- **`productVersion`** on the test machine is `["Arma 3","Arma3",222,154089,"Stable",false,"Windows","x64",""]`, which the agent reports as `2.22.154089 stable`[^probe].
- **A singleplayer client loads an extension from a mod given by its absolute path** outside the game folder[^rpt]; only a dedicated server needs the junction of `INT-AR-06`.
- **The first starts of the game on a cold machine failed** three times in a row, each with `Error 20 reading file` on a different `enoch` PBO and `Application terminated intentionally` about 5 s after start; the fourth and every later start worked[^rpt]. Python read the same files in full without error. Nothing in the framework caused it; a session start that meets it fails with the game's exit and its RPT lines (`SES-04`).

[^spikes]: crates/shakedown/tests/spikes.rs, runs of 2026-11-05 on the Windows test machine (sessions s-20261105-161417, s-20261105-161619, s-20261105-161740 and one after)
[^probe]: An -autotest mission of the developer's scratch folder logging toJSON, fromJSON, compile, ScriptError and terminate behaviour with diag_log, 2026-11-05, Arma 3 2.22.154089 stable
[^e2e]: crates/shakedown/tests/e2e.rs, us_037_us_038_checks_in_one_session, 2026-11-05
[^rpt]: RPT files of the spike and probe sessions on the test machine, 2026-11-05
