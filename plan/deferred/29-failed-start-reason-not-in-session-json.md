---
type: Deferred
title: "session.json of a failed start holds no reason"
description: "A start that failed, such as one the game ended with Error 20 reading file on a base-game PBO, writes reason null into session.json, so once status no longer holds it nothing says why the start failed."
id: DEF-29
state: open
milestone: null
status: stable
---

# session.json of a failed start holds no reason

## Summary

Every session folder gets a `session.json` when the session ends (`docs/sessions-and-the-runner.md`). For a start that failed it holds `state: failed` and `reason: null`. The reason of `SES-04`, the machine, the cause and the last 20 RPT lines, lives only in the runner's memory and in the output of `session start`. Testers met this after a start that failed because the game could not read a base-game file (`Error 20 reading file`).

## Current behaviour

When a start fails, `session::start` (`runner/session.rs`) calls `fail`, which sets the session `failed` with `reason` `{"message", "machine", "rpt_tail"}`, then `stop` (`P-10`). `stop` writes the end with `record::write_end(&self.folder, &self.id, end.as_str(), None)`: the reason argument is always `None`, so `session.json` reads `reason: null` for every failed start, whatever the cause. `write_end` gets a reason only from the recovery of `P-12`, which writes `runner-died`.

The runner keeps the failed view for `status` (`INS-02`, the `failed` state of the session status machine in `spec/01-domain-model.md`) until the next `session start` or `session stop`. That copy lives in the runner process only. A runner with no session and no client exits after 60 s (`P-01` step 4), and `status` then shows no runner and no session. From then on the session folder is the only record, and it says the start failed but not why.

`Error 20 reading file` on a base-game PBO is a fault of the machine, not of the framework: the game logs it while loading and ends with "Application terminated intentionally" about 5 s after it starts, and a second start works (research `22-spikes-singleplayer-channel.md`, `docs/milestones/m01.md`). The runner sees the process exit and fails the start through the exit path of `P-02` step 8, and the RPT tail in the reason holds the lines that name the file. This is the case where `reason: null` costs the most, since only those lines tell a broken game file from a broken mod.

## The gap

A person or an AI agent who comes back after the runner has exited, or who reads the session folder rather than `status`, cannot tell why a start failed: a missing Steam login, a crashed client, a ceiling and a game that could not read its own files all leave the same `session.json`. They have to find the right machine's RPT in the profile folders and read it themselves, which the reason of `SES-04` exists to spare them.

## Options

1. `stop` writes the session's `reason` into `session.json` when it holds one: the message, the machine and the RPT tail, redacted (`LOG-02`). A session ended by `session stop` from `ready` keeps `reason: null`, and the recovery keeps `runner-died`.
2. Write only the message and the machine, and leave the RPT lines in the profile folders.
3. `status` reads the newest session folder's `session.json` when the runner holds no session. This builds on the first or second option, since it needs a reason in the file to show.

## Recommendation

The first. It is a small change at the one place that writes the file, it keeps the same reason `session start` printed, and it keeps the reason of a `broken` session that is then stopped as well. Leave `status` as `INS-02` has it: the third option changes what `status` means and no story asks for it.

## Acceptance criteria

1. After a start that fails because a process exited (the case of `US-004` criterion 1), the session folder's `session.json` holds `state: failed` and a `reason` with the same `message`, `machine` and `rpt_tail` that `session start` printed, redacted.
2. After a start that fails at the start ceiling, `session.json` holds that reason likewise.
3. A session ended with `session stop` from `ready` still writes `reason: null`, and one the recovery ended still writes `runner-died`.
4. `docs/sessions-and-the-runner.md` and `SES-04` say that `session.json` keeps the reason of a failed start.
