---
type: Index
title: Deferred work
description: One Deferred concept per item, file NN-slug.md.
---

# Deferred work

* [A lost runner.json starts a second runner that ends the live session](01-runner-file-lost.md) - When runner.json is deleted or unreadable while its runner holds a session, the next client starts a second runner, whose recovery ends the first session's game processes.
* [A Steam name that is also a common word is redacted everywhere](02-steam-name-common-word.md) - LOG-02 replaces the Steam account and persona names as whole words, so a name that is also a machine name or a common word would be replaced wherever that word appears.
* [Crash dumps in a run folder are not redacted](03-crash-dump-identity.md) - P-09 copies each crash dump into the run folder as the engine wrote it, and a dump holds process memory that may include the Steam ID and names.
* [Late joiners for the grappling-hook port](04-late-joiners.md) - arma-grappling-hook's dedicated M00 checks include a client that joins late, and joining late is MCH-01, a should, so those cases cannot be ported in M10.
