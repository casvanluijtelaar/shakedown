---
type: Deferred
title: "Where a long tool result goes with no project is not specified"
description: "With no project, the MCP server writes a tool result over 20,000 characters to a shakedown-results folder in the system temp folder, and neither AGT-07 nor the MCP contract says so."
id: DEF-17
state: open
milestone: null
status: stable
---

# Where a long tool result goes with no project is not specified

## Summary

`AGT-07`, `spec/contract/mcp-tools.md` and `docs/agent-tools.md` say a tool result over 20,000 characters is written to the project's `.shakedown/results/` and the result gives its path. They do not say where it goes when `shakedown mcp` runs with no project, as it may: `status` and `guide` need none (`AGT-05`).

## Current behaviour

`fit` in `crates/shakedown/src/mcp/mod.rs` writes the result to `.shakedown/results/` of the project when the server found one, and otherwise to `shakedown-results` in the system temp folder, as `{tool}-{stamp}-{pid}.json`, and returns `{"result_path": path}`. When the file cannot be written, it returns the whole result, over the limit. With no project, only a tool that answers without one can pass the limit, such as `exec`, `inspect` or `logs` against a session another client started.

## The gap

An agent outside a project gets a path in a folder no document names, and the contract does not promise it. A cleanup of the project's `.shakedown/` misses those files, and the temp folder keeps them.

## Options

1. Write into the contract and the docs what the server does: with no project, the system temp folder's `shakedown-results`.
2. Put them in the runner's folder, beside `runner.log`, which every install has.
3. Refuse to answer over the limit with no project.

## Recommendation

The first, as a writer's edit with the next change to the MCP contract: what the server does serves the agent, and only the documents miss it.

## Acceptance criteria

1. `AGT-07`, `spec/contract/mcp-tools.md` and `docs/agent-tools.md` say where a long result goes with no project.
2. `shakedown mcp` started outside any project, asked for a result over 20,000 characters, returns `result_path` in the folder they name, and the file holds the whole result.
