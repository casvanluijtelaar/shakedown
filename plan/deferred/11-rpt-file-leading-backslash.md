---
type: Deferred
title: "The RPT scan keeps a leading backslash in an error's file"
description: "The RPT scan copies a File line's path as the RPT writes it, so a path with a leading backslash keeps it in the result file, while the docs say an error's file has none."
id: DEF-11
state: open
milestone: null
status: stable
---

# The RPT scan keeps a leading backslash in an error's file

## Summary

`docs/script-errors-and-logs.md` says an error's `file` is the file as the engine names it, without a leading backslash. The RPT scan of `ERR-02` copies the path of an error's `File <file>..., line <n>` line as written. The engine wrote no leading backslash on that line in any of M3's runs, so no result showed it.

## Current behaviour

An error the scan finds keeps the path from its `File` line unchanged. When the scan matches its errors to the handler's (`P-05` step 9), it compares the two files with any leading backslash ignored, and an error both saw is listed once, with the handler's copy.

## The gap

If the engine writes a `File` line with a leading backslash, an error that only the scan found has a `file` with one, against the docs. Any comparison of the two copies of one error that does not ignore the backslash would see two errors, and the error would be listed twice.

## Options

1. The scan drops a leading backslash from the path when it reads the `File` line, so both copies hold the same text.
2. Leave it until an RPT shows such a line.

## Recommendation

The first option, when the RPT scan is next worked on: it is one line, and it makes the scan hold what the docs say.

## Acceptance criteria

1. An RPT error whose `File` line names `\skdfx\addons\main\functions\fn_throw.sqf` appears in the result file with `file` `skdfx\addons\main\functions\fn_throw.sqf`.
2. The same error, also seen by the handler, is listed once.
