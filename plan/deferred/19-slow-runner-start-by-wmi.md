---
type: Deferred
title: "A runner started by WMI once took more than 10 s to write runner.json"
description: "In one of 25 rounds a runner that WMI had created wrote runner.json more than 10 s later, past the 10 s a client waits for it, right after the test had killed the previous runner and its game; it did not recur in 15 rounds with timings."
id: DEF-19
state: open
milestone: null
status: stable
---

# A runner started by WMI once took more than 10 s to write runner.json

## Summary

`us_035_nfr_rl_03_a_runner_started_by_wmi_ends_only_when_killed` in `crates/shakedown/tests/m05.rs` starts a runner by WMI each round, as a client does when its job forbids breakaway (`INT-WIN-02`). In its first run with the WMI start fixed (no console), the tenth round's runner wrote `runner.json` more than 10 s after `Win32_Process.Create` had returned. A client waits 10 s for that file (`P-01` step 2), so a client would have failed with "The runner did not start within 10 s". The round began seconds after the test had killed the previous runner with `taskkill /F` and its three game processes had ended.

## Current behaviour

The runner, once started, worked: it ran its recovery and served. Nothing was logged between its creation and its `started` line. In a second run of 15 rounds each runner wrote `runner.json` 0.34 to 0.45 s after the request, and 15 WMI starts of a short process on an idle machine took 0.31 to 0.32 s. The Windows Application, System and WMI-Activity logs show nothing in the window.

## The gap

Where the 10 s went is not known: in WMI creating the process, in the process loading (an antivirus scan of `shakedown.exe` under the load of three game processes ending is one candidate), or in the runner before its first log line (its recovery ends recorded processes and deletes the mission copy). It may not recur, and when it does, a client started by WMI fails its command.

## Options

1. Measure first: the test prints, each round, when WMI was asked, how long it took, the runner process's creation time (`started_at` in `runner.json`) and when `runner.json` appeared, and the runner's `started` line in `runner.log` gives its process's creation time. A recurrence then shows which part was slow.
2. Raise the client's 10 s ceiling for a start by WMI.
3. Have the runner write `runner.json` before its recovery, with the recovery done before it answers a hello.

## Recommendation

The first, which is in place: run the WMI test when the machine is next used for long runs, and decide between the second and the third from where the time went.

## Acceptance criteria

1. A recurrence is recorded with the times above, and the slow part is named.
2. A client started by WMI right after a runner and its game were killed finds `runner.json` within its ceiling in 50 of 50 rounds.
