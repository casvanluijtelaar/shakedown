---
type: User Story
title: "A change that needs a rebuild and a game restart"
description: "A new or deleted file, or a config change, is reloaded by a game restart, and the reload says that the mod must be rebuilt first."
status: stable
id: US-056
role: mod-developer
kind: mistake
requirements: [RLD-02, RLD-06, NFR-US-01, FIX-03]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-056 A change that needs a rebuild and a game restart

As a mod developer, I want to be told when my change needs a rebuild and a game restart, and why, so that I do not wait on a reload that cannot take my change.

## Acceptance criteria

1. `shakedown reload <a .sqf path under the watched source that is not in the build>` reports level `game-restart` for that file with a reason that says the mod must be rebuilt before this reload, because file patching finds only files in the built PBO.
2. A reload of a `config.cpp` under the watched source reports level `game-restart` with the reason that config is read at game start, and every machine comes back `ready` with a start number one higher.

## Next to

- [US-054](US-054-reload-changed-function.md) (feature): reload a changed function.
