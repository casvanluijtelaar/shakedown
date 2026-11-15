---
type: Index
title: Mod developer stories
description: The user stories of the mod developer role, one User Story concept per file.
---

# Mod developer stories

* [US-054 Reload a changed function and rerun the checks a finding names](US-054-reload-changed-function.md) - A mod developer changes one function, reloads it into the running session in one step, and reruns the checks a finding names, with the verdict in about a game start.
* [US-055 Reload a change to a final function by a game restart](US-055-reload-final-function.md) - A change to a function that is final, such as a CBA `PREP` function, is reloaded in one step, the processes stopped, the change made and the processes started, and the reload says why it needed a game restart.
* [US-056 A change that needs a rebuild and a game restart](US-056-change-needs-rebuild.md) - A new or deleted file, or a config change, is reloaded by a game restart, and the reload says that the mod must be rebuilt first.
* [US-057 A reload that does not complete](US-057-reload-fails.md) - When a reload does not reach every machine, it fails naming the step and the machine, the session is broken, and the checks after it are invalid rather than run on mixed code.
* [US-058 A reload asked while a run is active](US-058-reload-during-run.md) - A reload or a reset asked while a run, a reload or a reset is in progress is refused, saying what is in progress, and the run goes on untouched.
* [US-081 A changed function reaches a running session without a game restart](US-081-reload-without-game-restart.md) - A mod developer changes one function of a mod built for file patching and reloads it, and every machine of a running session runs the new code after a mission restart instead of a game restart, within 30 s of the save in singleplayer.
* [US-082 The game still holds the changed file](US-082-game-holds-changed-file.md) - When a machine still holds a changed file after the reload has made it read the framework's release files, the reload applies a game restart instead and reports why, so no check runs on code the game did not take.
