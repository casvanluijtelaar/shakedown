---
type: Deferred
title: "Checks cannot share helper code"
description: "A check cannot load another file and runs no preprocessor, so helper code that many checks use is copied into each, and a change to it is one edit per copy that nothing keeps in step."
id: DEF-27
state: open
milestone: null
status: stable
---

# Checks cannot share helper code

## Summary

An old suite usually shares a library between its tests through `#include`. A Shakedown check is compiled without the preprocessor (`API-01` to `API-05`) and has no way to load another file, so the M10 port of arma-grappling-hook copied a 200-line helper block into eight check files. Raised by the M10 port as finding `m10-4`.

## Current behaviour

`docs/porting-a-suite.md` says each check carries its own copy of the helpers it uses, as local code values at the top of its body, kept identical across files.

## The gap

A change to a shared helper is one edit per copy, and nothing tells the author that two copies differ. Long copies also lengthen the body the game compiles before each check.

## Options

1. A library folder in the project, such as `checks/_lib/` or a `lib` key in `shakedown.toml`: every `.sqf` file in it is compiled once per session on every machine, after the mission starts and after each reload, into a variable named after the file, and `list` skips it. A compile error in it ends every check `fail` / `check-error` naming the file and line.
2. A header key, `// @uses <file>`, that prepends the named file's text to the body before it is sent, so line numbers of the body shift by a known amount the runner subtracts.
3. Leave the copies, as documented.

## Recommendation

The first option, after the MVP: it keeps the check body a plain SQF file, needs no line offsets, and loads the helpers on every machine, so code sent with `skd_fnc_on` can call them too. It is a change to the check API contract, the project file and `RLD-*` (a reload recompiles the library).

## Acceptance criteria

1. A helper defined in the library folder is callable in the body of every check and in code sent to any machine, in singleplayer and on a dedicated session.
2. A changed helper is used by the next check after `shakedown reload`, with no game restart.
3. A compile error in a library file ends the next check `fail` / `check-error` with the file and line.
4. `list` lists no library file as a check, and `docs/porting-a-suite.md` says where an old suite's shared library goes.
