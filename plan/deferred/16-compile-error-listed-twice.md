---
type: Deferred
title: "An exec of code that does not compile lists its error twice"
description: "exec of code that does not compile, such as 1 +, gives the engine's Invalid number in expression error twice in what it answers, where one is meant."
id: DEF-16
state: open
milestone: null
status: stable
---

# An exec of code that does not compile lists its error twice

## Summary

`exec` of code that does not compile exits 3, and the tool returns `compile-error`, with the engine's message (`CTL-04`, `US-014`). For `1 +`, the M4 test developers saw the same "Invalid number in expression" error listed twice. The answer is right; it says it twice.

## Current behaviour

`fn_runExec.sqf` compiles the source with `skd_compiling` set; the `ScriptError` handler in `fn_init.sqf` records each error raised meanwhile in `skd_compileErrors` and also sends each to the runner, marked `compile`. The machine answers `compile-error` with the first recorded error and its line. `exec` in `runner/control.rs` turns that into "The code does not compile on p1: Invalid number in expression (line 1). Fix the SQF and send it again.", and `errors`, `inspect` and `exec`'s own `errors` leave out every error marked `compile`. The code takes the message from one error only, so the second copy likely comes from the engine's own text or from a second error the engine raised for the same compile and that some answer does not leave out. That has not been traced, because tracing it needs the game.

## The gap

A person or an agent reading the answer sees two errors where the code has one, and may look for a second fault.

## Options

1. Trace which answer holds the second copy, and remove the duplicate there: one error for one fault.
2. Leave it: the answer names the fault and its line, and the call fails as it should.

## Recommendation

The first, when `exec` is next worked on. It is likely one line once traced.

## Acceptance criteria

1. `shakedown exec --machine p1 --code "1 +"` exits 3, and its output, with and without `--json`, holds "Invalid number in expression" once.
2. The `exec` tool with the same code returns `compile-error` whose text holds it once, and `errors` afterwards lists no error for it.
