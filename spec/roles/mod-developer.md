---
type: Role
title: Mod developer
description: Changes a mod's code and wants to know at once whether it still does what it promised, by reloading the change into a running session and rerunning the checks a finding or story names; a person or an AI agent.
status: draft
id: mod-developer
group: person-or-agent
---

# Mod developer

**Who.** Someone who changes the mod under test (PRD section 3). Often an AI agent working in the mod's own repository, answering findings the test author raised; sometimes a person. Builds the mod with its own tools, such as HEMTT; Shakedown loads the build.

**Trying to get done.** Reload the changed code into a running game and rerun the checks a finding names, without a restart, and get the verdict in seconds.

**What they understand.** The mod's source, its build, and which files they changed. Not the framework's internals: a reload is told to them as the level it took, which file needed it and why, and how long it took; a failed reload as which step and which machine did not take the new code.
