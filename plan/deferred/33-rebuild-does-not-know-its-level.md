---
type: Deferred
title: "A mod's rebuild does not know which level of reload runs it"
description: "One rebuild command serves both levels, so a mod whose build is a PBO needs a rebuild that only copies loose files, since a game process still holds the PBOs during a mission-restart reload, and a new file then needs a hemtt build outside the reload."
id: DEF-33
state: open
milestone: null
status: draft
---

# A mod's rebuild does not know which level of reload runs it

## Summary

M14 runs a mod's `rebuild` at level `mission-restart` once the game has let go of the changed `.sqf` files, while the game processes still hold the mod's PBOs and every other file they read (`RLD-06`). The same command runs at level `game-restart`, after every game process has ended.

## Current behaviour

A rebuild that rewrites a PBO, such as `hemtt build`, fails a `mission-restart` reload. A rebuild that only copies changed loose files into the file patching tree, such as `robocopy`, serves both levels, but a new file then reaches no PBO, so file patching does not find it after a `game-restart` reload either. The developer runs `hemtt build` with the session stopped.

## The gap

A mod built with HEMTT, as arma-grappling-hook is, cannot have one rebuild that both copies a changed function at `mission-restart` and builds the PBOs at `game-restart`.

## Options

1. Pass the level to the rebuild, such as in an environment variable `SHAKEDOWN_RELOAD_LEVEL`, so a script runs `hemtt build` only at `game-restart`.
2. Two keys, `rebuild` and `rebuild_files`, one per level.

## Recommendation

Option 1, when arma-grappling-hook's use shows new files are common enough to matter. It is one variable in the contract of `mods.<name>.rebuild`.

## Acceptance criteria

- A rebuild can tell from its environment whether a `mission-restart` or a `game-restart` reload runs it, and `docs/reloading.md` shows a script that copies at one and builds at the other.
