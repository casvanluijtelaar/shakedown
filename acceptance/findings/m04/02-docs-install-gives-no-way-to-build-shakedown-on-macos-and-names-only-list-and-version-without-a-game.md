---
type: Finding
id: m04-2
milestone: M4
title: docs/install.md gives no way to build Shakedown on macOS, and names only list and version as working without a game
description: NFR-PT-02 promises list, status, guide and the MCP server on macOS with no game, but docs/install.md says only list and version work with no game, and its one install path stops at hemtt build on a Mac before it assembles an install folder.
raised_by: test
for: writer
checks: []
stories: [US-046]
generated: { by: "claude-code/tester", at: 2026-11-06T21:35:09+01:00 }
addressed: []
verified: []
---

# docs/install.md gives no way to build Shakedown on macOS, and names only list and version as working without a game

## What the spec promises

`NFR-PT-02` (must): "`list`, `status`, `guide` and the MCP server's start work on macOS and Linux with no game, so an AI agent on the Mac can read checks and the contract (`CR-53`)." `US-046` criterion 3: "On the Mac, with no game installed, `shakedown mcp` starts and its `status` tool answers `runner` `not-running`; `list_checks` and `guide` answer too." `docs/agent-tools.md` says the server needs `shakedown` on `PATH` and points to [Installing Shakedown](../../../docs/install.md) for the install folder.

## What the documents say

`docs/install.md`, under "What the machine needs": "`list` and `version` work on any system with no game. Starting a session needs Windows." It names neither `status`, `guide` nor `mcp`. Its only way to build is `python tools/install.py`, which needs HEMTT; it lists Windows as what the machine needs and says nothing of a Mac.

On the owner's Mac, with Rust 1.99.1 and no HEMTT, `python3 tools/install.py <folder>` built `shakedown` in release mode with cargo, then failed at `hemtt build` with `FileNotFoundError: ... 'hemtt'` and assembled no install folder. The binary it left in `target/release/shakedown` does everything `NFR-PT-02` promises (the Mac round of `US-046` passed 3 of 3 with it), but no document says to use it, or how to build only what a Mac needs.

## What it costs

A test author or their AI agent on a Mac, which `NFR-PT-02` and `CR-53` serve, follows `docs/install.md` and gets no install folder, and reads that only `list` and `version` work there. They have to guess at `target/release/` or install HEMTT to build an addon they cannot use. The framework works; the document does not lead to it.

## The check that holds it

None: a document gap. The Mac round's check, `US-046 AGT-05 NFR-PT-02 on the Mac with no game shakedown mcp starts from .mcp.json and answers status runner not-running, list_checks and guide, and starts nothing else`, shows the built binary does what the documents should say.
