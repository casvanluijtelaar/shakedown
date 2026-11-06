---
type: Finding
id: m04-1
milestone: M4
title: AGT-07 puts a long tool result in the session or run folder, where the contract and the docs put it in .shakedown/results
description: spec/03-functional-requirements.md says a tool result over 20,000 characters is written to a file in the session or run folder; spec/contract/mcp-tools.md and docs/agent-tools.md say the project's .shakedown/results/, which is where the built server writes it.
raised_by: test
for: writer
checks: []
stories: [US-048]
generated: { by: "claude-code/tester", at: 2026-11-06T11:02:42-08:00 }
addressed: []
verified: []
---

# AGT-07 puts a long tool result in the session or run folder, where the contract and the docs put it in .shakedown/results

## What the spec promises

`AGT-07` in `spec/03-functional-requirements.md`: "Tool results stay under 20,000 characters; anything longer is written to a file in the session or run folder and the result gives its path."

`spec/contract/mcp-tools.md`, rules for every tool: "A result over 20,000 characters is written to a file in the project's `.shakedown/results/` and the result is `{"result_path": path}` instead". `docs/agent-tools.md` says the same.

The two say different folders for the same file. A session and a run each have their own folder under `.shakedown/`; `.shakedown/results/` is neither.

## What the system does

In a singleplayer session, the `exec` tool with `{"machine": "p1", "code": "private _a = []; for '_i' from 1 to 3000 do {_a pushBack 'xxxxxxxx'}; _a joinString ''"}` returns `{"result_path": "<project>\.shakedown\results\exec-<time>-<pid>.json"}`, and `list_checks` of a project of about 170 checks returns a `result_path` in the same folder. The built server follows the contract and the docs.

## What it costs

A reader of the requirements looks for a long result in the session or run folder and does not find it; a test author who cleans up a session folder expects its long results to go with it, and they stay in `.shakedown/results/`. No behaviour is wrong today; the requirement and the contract disagree, and whichever is changed next decides which one the framework follows.

## The check that holds it

None: it is a disagreement between two documents. The check `US-048 AGT-07 logs with last 5000 and an exec value over 20,000 characters return a path to a file holding the whole result` asserts the contract's folder, `.shakedown/results/`.
