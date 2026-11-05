---
type: Deferred
title: "Crash dumps in a run folder are not redacted"
description: "P-09 copies each crash dump into the run folder as the engine wrote it, and a dump holds process memory that may include the Steam ID and names."
id: DEF-03
state: open
milestone: null
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# Crash dumps in a run folder are not redacted

## Summary

A crash dump (`.mdmp`, `.bidmp`) is a copy of process memory. `P-09` copies it into `dumps/<machine>/` of the run folder unchanged, so sharing a run folder can share the Steam identity the rest of the folder keeps out.

## Current behaviour

Dumps are copied unredacted. The result file contract says so, and `.shakedown/` is ignored by git (`CFG-02`).

## The gap

A person who zips a run folder to share a failure shares the account's Steam ID and names with it.

## Options

1. Keep dumps in the machine's profile folder only, and give their absolute paths in the result file.
2. Copy dumps into the run folder only when the run is asked to keep them.
3. Leave it, and say in `docs/` that `dumps/` is not to be shared.

## Recommendation

Leave it for the MVP, with the warning in `docs/`; take the first option once a run folder is shared outside the owner's machine.

## Acceptance criteria

1. A run with a crash leaves no dump in the run folder, the result file gives the dump's path in the profile folder, and `US-016` and `US-043` still pass with that path.
