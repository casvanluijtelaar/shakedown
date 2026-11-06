---
type: Index
title: M4 findings
description: The findings of M4 Agent tools.
---

# M4 findings

* [AGT-07 puts a long tool result in the session or run folder, where the contract and the docs put it in .shakedown/results](01-agt-07-puts-long-results-in-the-session-or-run-folder-where-the-contract-puts-them-in-shakedown-results.md) - spec/03-functional-requirements.md says a tool result over 20,000 characters is written to a file in the session or run folder; spec/contract/mcp-tools.md and docs/agent-tools.md say the project's .shakedown/results/, which is where the built server writes it.
