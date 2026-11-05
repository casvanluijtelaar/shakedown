---
type: Deferred
title: "A Steam name that is also a common word is redacted everywhere"
description: "LOG-02 replaces the Steam account and persona names as whole words, so a name that is also a machine name or a common word would be replaced wherever that word appears."
id: DEF-02
state: open
milestone: null
status: stable
verified: [{ by: "human:casvanluijtelaar", at: 2026-11-05T09:26:08+01:00 }]
---

# A Steam name that is also a common word is redacted everywhere

## Summary

`LOG-02` replaces the logged-in account's account name and persona name with `<steamname>`, as whole words and case-sensitive. A persona name such as `server`, `p1` or `Error` is also a word the framework and the game write, so the redaction would replace it in places that hold no Steam name.

## Current behaviour

Every whole-word match of either name is replaced, in the result file, the run folder's logs, command and tool output and stopped sessions' RPTs. The RPT scan of `ERR-02` reads the lines before they are redacted.

## The gap

A result or a log can lose a machine name or a word of an error message, and a reader cannot tell which `<steamname>` was a name.

## Options

1. Refuse a session start whose account has a name shorter than four characters, equal to a machine or group name, or on a short list of words the game writes, and say to change the persona name.
2. Replace the names only in the lines the game writes them in, such as connection and player-list lines.
3. Leave it until an account shows it.

## Recommendation

Leave it until an account on a test machine shows it; then the first option, because it keeps the redaction simple and tells the user exactly what to change.

## Acceptance criteria

1. With a persona name `server`, a session start is refused, naming the persona name and saying to change it.
2. With an ordinary persona name, nothing changes from `US-064`.
