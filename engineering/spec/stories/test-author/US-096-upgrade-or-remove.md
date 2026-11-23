---
type: User Story
title: "Upgrade or remove an installed Shakedown"
description: "A test author moves an installed Shakedown to a newer release, or removes it, with one command of the route that installed it."
status: draft
id: US-096
role: test-author
kind: feature
requirements: [DST-02, DST-03]
---

# US-096 Upgrade or remove an installed Shakedown

As a test author, I want to move to a newer Shakedown, or take it off my machine, with one command, so that keeping the framework current costs me nothing.

## Acceptance criteria

1. With an older release installed by winget, `winget upgrade` of the package installs the newer one, and `shakedown version` in a new shell prints it.
2. With an older release installed by the one-line script, running the script again installs the latest, `shakedown version` prints it, and the user's `PATH` names the install folder once.
3. `winget uninstall` of the package removes `shakedown.exe` and `@shakedown\`, and `shakedown` is no longer found in a new shell.

## Next to

- [US-098](US-098-upgrade-while-running.md) (mistake): an upgrade while Shakedown runs.
