---
type: User Story
title: "An AI agent uses every capability as tools, with the same results as the command line"
description: "An AI agent drives every capability of the command line through the `shakedown` MCP server, and gets the same JSON a person gets with `--json`."
status: stable
id: US-048
role: test-author
kind: feature
requirements: [AGT-01, AGT-02, AGT-06, AGT-07, NFR-US-03]
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-04T23:46:25+01:00 }]
---

# US-048 An AI agent uses every capability as tools, with the same results as the command line

As a test author, I want to use every capability of the framework as tools from my AI agent's session, with the same answers as the command line, so that an agent tests a mod without parsing logs.

## Acceptance criteria

1. With the project's `.mcp.json`, Claude Code lists the tools of the MCP contract, each with an `outputSchema`, and each returns `structuredContent` with the same JSON as one text block.
2. For the same request, `status`, `list_checks`, `exec`, `inspect` and `run_result` return the same JSON as `shakedown <command> --json`; a run with a failing check is a successful `run_result` call whose outcome is `fail`.
3. `guide` returns under 8,000 characters on starting a session, running checks, reading a result and writing a check; a result that would pass 20,000 characters, such as `logs` with `last` 5000, returns a path to a file holding it instead.

## Next to

- [US-049](US-049-agent-fixable-error.md) (mistake): the agent makes a mistake it can fix.
