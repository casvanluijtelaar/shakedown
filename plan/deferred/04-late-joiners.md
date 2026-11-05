---
type: Deferred
title: "Late joiners for the grappling-hook port"
description: "arma-grappling-hook's dedicated M00 checks include a client that joins late, and joining late is MCH-01, a should, so those cases cannot be ported in M10."
id: DEF-04
state: open
milestone: null
---

# Late joiners for the grappling-hook port

## Summary

Two of arma-grappling-hook's M00 checks on a dedicated server, `US-009-1-freed` and `MP-01-drawn`, run with a third client that joins after the others. A Shakedown session starts every machine of its setup together; joining late is `MCH-01` (`CR-04`), a `should`.

## Current behaviour

Every machine of a session starts with it. No check can make a machine join, leave or rejoin.

## The gap

M10 ports those two checks with two clients joined at start; their late-joiner cases stay in arma-grappling-hook's PowerShell suite.

## Options

1. Plan `MCH-01` as the first milestone of phase 2, with its stories written then.
2. Add only a late client to a setup, started by a check, ahead of the rest of `MCH-01`.

## Recommendation

The first option: `MCH-01` names joining late, leaving and rejoining together, and arma-grappling-hook needs all three for disconnects and respawns.

## Acceptance criteria

1. The late-joiner cases of `US-009-1-freed` and `MP-01-drawn` run as Shakedown checks on a dedicated server and reach the verdicts arma-grappling-hook's PowerShell suite reaches on the same build.
