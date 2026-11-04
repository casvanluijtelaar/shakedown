---
type: Role
title: Test author
description: Writes acceptance checks in SQF for a mod from its user stories, without reading the mod's source, runs them and reads the verdicts; a person or an AI agent.
status: stable
id: test-author
group: person-or-agent
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# Test author

**Who.** Someone who writes the acceptance checks of a mod from its user stories (PRD section 3). Often an AI agent working as the test developer of a mod's repository, through the MCP tools; sometimes a person, through the command line. Either way the test author does not read the mod's source and works only from the check API, the command line, the MCP tools and the result file.

**Trying to get done.** Set up a situation in the game, make players do things, wait for an outcome and assert it, on every machine that matters, in singleplayer and on a dedicated server with several clients.

**What they understand.** SQF, the mod's user stories and its public functions, actions and keybinds, and the words of [../00-overview.md](../00-overview.md). Not the framework's internals: a failure is told to them as which check, which step, which machine, what was expected and what was seen, and whether the mod or the environment is to blame.

**What they need to trust.** A `fail` means the mod did not do what the check asserts. An `invalid` means the environment stopped the check and nothing is known about the mod. A broken check is told as the author's to fix.
