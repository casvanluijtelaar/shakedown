---
type: Index
title: M1 findings
description: The findings of M1 A grappling-hook check in singleplayer.
---

# M1 findings

* [A check whose name does not start with a story or requirement ID is accepted](01-a-check-name-without-an-id-is-accepted.md) - list and run take a check whose @check name starts with a plain word, though API-01 requires the name to start with the ID the check proves.
* [The check-timeout message names no next step](02-the-check-timeout-message-names-no-next-step.md) - A check that runs past its @timeout ends with the message "The check ran past its timeout of 5 s while the game still answered.", which says what went wrong but not what the author does next, as NFR-US-01 requires.
