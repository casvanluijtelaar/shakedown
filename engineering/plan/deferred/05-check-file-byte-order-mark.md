---
type: Deferred
title: "A check file saved with a byte order mark has no header"
description: "A check file that starts with a UTF-8 byte order mark is reported as having no @check line, because its first header line does not read as one."
id: DEF-05
state: done
milestone: M1
status: stable
---

# A check file saved with a byte order mark has no header

## Summary

The check API contract says a check file "starts with a header: one or more lines of the form `// @key value`" (`API-01`). Editors on Windows, Notepad and some PowerShell commands among them, save UTF-8 with a byte order mark before the first character. A check saved that way is otherwise a correct check.

## Current behaviour

`list` gives such a check the error "the header has no @check line", and `run` ends it `fail` / `check-error` with that message, though the file's first line is a `// @check` line.

## The gap

The test author is told the file has no `@check` line while looking at one. `US-038` promises a message that says what to fix, and nothing in this one points at the byte order mark, which no editor shows.

## Options

1. Skip a UTF-8 byte order mark at the start of a check file before reading the header, and say in `spec/contract/check-api.md` that one is allowed.
2. Refuse the file with a `check-error` that names the byte order mark and says to save the file as UTF-8 without one.

## Recommendation

The first option. The mark carries no meaning in a UTF-8 file, the author did not choose it, and refusing it only adds a step to every check written on Windows.

## Acceptance criteria

1. A check file that starts with a UTF-8 byte order mark and is otherwise a correct check is listed by `list` with its name and an `error` of `null`, and runs to the same outcome as the same file without the mark.
2. `spec/contract/check-api.md` says a check file may start with a UTF-8 byte order mark.
