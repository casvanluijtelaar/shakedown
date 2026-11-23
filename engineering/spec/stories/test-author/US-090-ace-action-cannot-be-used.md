---
type: User Story
title: "The ACE action cannot be used"
description: "When the ACE action a check asks for is missing, its condition is false or ACE is not loaded, the check says which, with the actions that exist on that path."
status: draft
id: US-090
role: test-author
kind: error
requirements: [ACT-04, DM-03, DM-04]
---

# US-090 The ACE action cannot be used

As a test author, I want a failed ACE action to say why, so that I can tell a wrong path from an action the mod did not offer.

## Acceptance criteria

1. An ACE action path whose last name does not exist ends the check `fail` / `assertion` at the ceiling, listing the action names that exist at the path's parent.
2. An action whose condition is false ends `fail` / `assertion` at the ceiling saying so.
3. On a machine without ACE the check ends `fail` / `check-error` saying ACE is not loaded.

## Next to

- [US-089](US-089-use-ace-interaction.md) (feature): use a mod's ACE interaction action.
