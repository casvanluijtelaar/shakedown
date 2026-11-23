---
type: Deferred
title: "An order held through a mission restart can keep a tool call past 60 s"
description: "An exec, wait or inspect sent to a machine whose mission restarts waits for it before its own ceiling starts, so on a dedicated session, where a mission restart takes about 48 s, an MCP call can last longer than the 60 s of AGT-03."
id: DEF-20
state: open
milestone: null
status: stable
---

# An order held through a mission restart can keep a tool call past 60 s

## Summary

`P-08` holds every new order to a machine whose mission restarts or whose process is replaced until the machine is `ready` again, for at most the setup's start ceiling, and then sends it with its whole ceiling, so no order is lost across a restart (`NFR-RL-04`). A tool call of the MCP server lasts at most 60 s (`AGT-03`).

## Current behaviour

An `exec`, `wait_until`, `expect` or `inspect` call sent while a `@reset mission` check or a `game-restart` reload restarts a dedicated session waits up to about 48 s for the machine (measured in `research/27-spikes-reload.md`), then up to its ceiling of at most 50 s plus the grace. The command line has no 60 s limit and is not affected.

## The gap

An agent that calls a tool on a machine during a restart can wait longer than `AGT-03` allows, and its client may give up on the call. No M6 story sends a tool call during a restart.

## Options

1. Count the hold against the call's own ceiling in the MCP server only, and answer `session-busy` naming the restart when it would pass 60 s.
2. Refuse an order to a held machine at once from the MCP server, with `session-busy` and the advice to call `status` until the session is `ready`.
3. Leave it: a restart outside a run happens only on the agent's own `reload`, which it waits for.

## Recommendation

Option 2: an agent learns at once that the session is restarting, and the command line keeps the hold.

## Acceptance criteria

1. An `exec` tool call to `p1` sent while a `@reset mission` check restarts a dedicated session answers within 60 s.
2. An `exec` from the command line sent at the same time is answered `2` once the new `p1` is ready.
