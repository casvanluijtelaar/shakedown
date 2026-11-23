---
type: Index
title: M4 findings
description: The findings of M4 Agent tools.
---

# M4 findings

* [AGT-07 puts a long tool result in the session or run folder, where the contract and the docs put it in .shakedown/results](01-agt-07-puts-long-results-in-the-session-or-run-folder-where-the-contract-puts-them-in-shakedown-results.md) - spec/03-functional-requirements.md says a tool result over 20,000 characters is written to a file in the session or run folder; spec/contract/mcp-tools.md and docs/agent-tools.md say the project's .shakedown/results/, which is where the built server writes it.
* [docs/install.md gives no way to build Shakedown on macOS, and names only list and version as working without a game](02-docs-install-gives-no-way-to-build-shakedown-on-macos-and-names-only-list-and-version-without-a-game.md) - NFR-PT-02 promises list, status, guide and the MCP server on macOS with no game, but docs/install.md says only list and version work with no game, and its one install path stops at hemtt build on a Mac before it assembles an install folder.
