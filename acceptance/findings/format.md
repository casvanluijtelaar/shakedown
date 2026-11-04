---
type: Documentation
title: Findings format
description: How a finding is written, numbered, answered and verified.
status: stable
---

# Findings format

A finding is one OKF concept of type `Finding`, in its own file in `acceptance/findings/mNN/`. It reports a place where the running system, or a document, breaks a promise the spec makes.

## File and number

- The file name is a number and the problem stated as a sentence: `04-the-board-hides-orders-with-no-date.md`.
- Numbers count up within the milestone folder and are never reused. A new finding takes the number after the highest one in the folder.
- The `id` is `mNN-n`, such as `m02-4`, and never changes.
- The milestone folder's `index.md` lists every finding, one line each. The line for the folder in `acceptance/findings/index.md` ends in `/`.

## Frontmatter

```yaml
---
type: Finding
id: m02-4
milestone: M2
title: The board hides every order that has no planned date
description: The unassigned column filters on plannedDate, so an order nobody has planned never appears.
raised_by: test        # test, backend, frontend or writer
for: backend           # the role that answers it: backend, frontend or writer
checks: ["US-206 an unplanned order shows in the unassigned column"]
stories: [US-206]
generated: { by: "claude-code/tester", at: 2026-09-01T14:20:00+02:00 }
addressed: []
verified: []
---
```

`checks` names the acceptance checks that prove the finding, by name, so a fix round can run only those. A gap in a document has an empty `checks` list.

## Body

1. **What the spec promises**, quoted, with the story or requirement ID. A finding that cannot quote the promise is not a finding yet.
2. **What the system does**, shown as the request and the answer. Enough to reproduce it without the check.
3. **What it costs**, in the terms of the person the story is written for. This decides whether it is fixed now or goes into `plan/deferred/`.
4. **The check that holds it**, by name.

A finding names no cause and points at no file in the source. Its author has not read the source.

## What is not a finding

- Behaviour the spec is silent about. There the checks assert only that the answer is a domain or validation error, not an internal error.
- A promise a later milestone owns.
- A mistake in the check itself. The tester fixes it.
- A failing test in the developer's own suite. It is fixed where it fails.
- A failure caused only by another session's work in progress.

## Answering and verifying

Entries are added and never rewritten or removed. A finding is never deleted.

- The role that answers a finding adds `{ by, at, note }` to `addressed`: an OKF actor, a time, and one line on what answered it and which test proves it. For a document gap, the note names each document changed.
- A tester verifies every finding, whoever raised and answered it. When the checks hold, or the documents now say what the finding asked for, it adds `{ by, at }` to `verified`.

```yaml
addressed:
  - { by: "claude-code/backend", at: 2026-09-02T09:00:00+02:00, note: "The unassigned column lists every order with no technician, dated or not. The planning board integration test proves it." }
verified:
  - { by: "claude-code/tester", at: 2026-09-02T11:30:00+02:00 }
```

The two lists give the state:

- **Open** while `addressed` is empty.
- **Answered** when the last `addressed` entry is newer than the last `verified` entry.
- **Closed** when the last `verified` entry is newer than the last `addressed` entry.

When an answer does not hold, the tester writes a new finding that links to the old one, and sets `status: deprecated` on the old one.
