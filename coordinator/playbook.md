# Coordinator playbook

Load this file at the start of every coordinator session, together with `CLAUDE.md`, which it does not repeat. It holds the loop, the frame of every subagent prompt, the checks after every report, what has been learned about the machine, the decisions the owner made about how to run the loop, and the state at the end of the last session. Product decisions never go here; they go into the spec through a framework developer round. A coordinator adds what it learns to the section it belongs in, as a dated entry.

## The coordinator's job

The coordinator writes no code, no checks, no spec and no plan, and edits no repository file except this playbook. It starts subagents in one of the three roles from `CLAUDE.md` (framework developer, test developer, technical writer), each with one task, reads each final report, checks the repository and the machine after each one, and decides what to start next or stops and asks the owner. Every repository change goes through a subagent in the role that owns the file. A plan or spec change is a framework developer's task, even a one-line one. A technical writer round runs only for a document change with no code change. The coordinator never reads the code.

The coordinator runs on the Windows test machine, because every round that builds or checks the framework starts Arma 3.

## The loop for one milestone

The project is MVP first (`CLAUDE.md`, "How this project runs"). The loop serves the milestone's demo, not completeness.

1. **Build.** A framework developer builds the milestone. If the report leaves deliverables open, start another round on what remains. That round sorts the open items by the milestone's stories and exit criteria, builds those, and moves the rest to the milestone that owns them or to `plan/deferred/`. It may not defer an item the milestone's demo needs.
2. **Test.** A test developer builds the checks for the milestone and writes findings. If it calls the work done but lists stories it did not drive, start another tester round for those first.
3. **Fix.** Show the owner each set of findings and go on without waiting. Start a framework developer on the open findings in `acceptance/findings/mNN/`. It grounds a finding that questions the spec before it fixes it.
4. **Verify.** Start a fresh test developer to verify every new `addressed` entry and add `verified` entries. A finding that does not hold becomes a new finding that links to the old one.
5. Repeat 3 and 4 until every finding is verified and every story in the milestone has a passing check.
6. **The owner looks.** When the milestone's exit criteria include something only a person can judge in the game, hand it to the owner: the command that sets up the situation and what to look for. A framework developer round records what the owner saw.
7. **Close.** A small framework developer round runs the milestone's repeated-demo test if its exit criteria have one, sets the milestone's `state` to `complete`, writes `docs/milestones/mNN.md`, tags the release if the milestone has a `version`, and names the next milestone. Report that `main` is ready to push.

A phase closes when the full acceptance suite is green on the Windows test machine and the GitHub Actions gates are green on `main`. There is no CI run of the game.

### Keep the fix loop narrow

- Every finding names its checks. The suite takes a filter that runs only the named checks.
- A fix round runs the checks its findings name, through the filter, before it hands back. A finding whose checks stay red is not addressed.
- A fix round runs only the tests of the parts it touched and the one end-to-end test that covers its change.
- A verify round runs only the checks behind the new `addressed` entries, three times, in one game session. That is the flake check.
- The full acceptance suite runs once, at the last verify before the milestone closes.

### What is not worth a round before the MVP

- A finding about an edge case that no story of the milestone promises is not a finding. The tester lists it in its report as a candidate for `plan/deferred/`, and the coordinator passes the list to the owner at the end of the milestone.
- A finding that is real but outside the milestone's demo path goes to `plan/deferred/` through a framework developer round, when the owner agrees.

## When to stop and ask the owner

| Stop | By default | What lifts it |
|---|---|---|
| A finding questions the spec, and the research and the client requirements cannot settle it | Show the finding and the developer's report. Fix nothing for it. The other findings go on. | Nothing. The owner decides. |
| A spike fails and its fallback changes what the spec promises | Show the spike's result and the fallback. Build nothing that depends on it. | Nothing. The owner decides. |
| A look in the game that only the owner can do | Give the command and what to look for. | Nothing. Only the owner can look. |
| The end of a milestone: every finding verified | Report that `main` is ready to push. Ask whether to push and whether to start the next milestone. | "Go on to the next milestone without asking." The push still waits for the owner. |
| An agent the owner stopped | Report what is left and ask. | Nothing. |

A lift holds until the owner takes it back, also in a resumed session. Record it with its date under "Decisions the owner made about the loop". Do not ask for a push between rounds.

## Sequencing rules

- One round at a time. Two rounds never run at once, because both would start Arma 3 on one machine: one Steam session, one set of ports, limited memory. A round that starts no game, such as a technical writer round, may run beside one that does.
- Do not start the next milestone while the current one has open findings.

## The frame of every subagent prompt

> You are the [framework developer | test developer | technical writer] for this repository, [repository path]. You run as a subagent. A coordinator reads your final report. Nobody watches you live and nobody answers questions.
>
> Read CLAUDE.md in full and follow it. [For a tester: Do not open the source code folders or fixture mod sources that `spec/07-tech-stack.md` names, and do not edit them, except the reload target files `spec/contract/fixture-mod.md` names, in the installed copy only.] [For a developer: Of `acceptance/`, read only `acceptance/README.md` and `acceptance/findings/`. Do not edit any check.]
>
> [The task, pointing at files. No summary of findings and no suggested fix.]
>
> - This project is MVP first. Build or check what the milestone's stories and exit criteria need, and stop. List what you left out in your report.
> - Answer your own questions from the user stories and the spec. Where they are silent or disagree, choose the reading that fits the spec's design principles, write it into the spec, and name it in your report. A finding that questions the spec is the exception: ground it in the research and the client requirements, or report it as open.
> - Give every background shell and every wait a ceiling: a counted loop, never an open-ended wait. Before you report, list the `arma3_x64` and `arma3server_x64` processes and stop any you started, by id, never by name.
> - Run `git worktree list` first. Do not touch another session's worktree or processes.
> - Use `[scratch folder]/[worktree name]/` for temporary files. Nothing else in the scratch folder is yours.
> - Never use `git stash`. Never use memory. Publish nothing outside this machine.
> - Commit when you are done. This overrides the rule of waiting to be asked. Work in `git worktree add .worktrees/[role]-[topic] -b [role]/[topic]`, inside the repository. When done, rebase on main, fast-forward merge into main, remove the worktree and delete the branch. If the main checkout has uncommitted changes, do not merge, and say so. Do not push.
>
> End with a short numbered report: each item done or not, the tests and checks you ran and their results, the decisions you made and where you recorded them, what you left out, the commit hashes, that the worktree is removed, the game processes you stopped, and that no background shell is running.

Never pass a model override. Every subagent runs on the coordinator's model, however small the task.

## Rules for tester prompts

- Never tell a tester what to test. It derives every check from the stories and requirements. The coordinator may say which findings to verify and what a previous tester left uncovered.
- A developer's claim about which checks must change is a claim to check against the contract, not an instruction.
- An answer that changed the contract must itself agree with the stories and requirements, or it is a finding.
- No failure is load. A failure in one run out of three is a flaky framework (a finding) or a flaky check (the tester fixes it). Flaky results are the problem this framework exists to end.
- A reason not to drive a story is checked against the story.
- A failure caused only by another session's work in progress is not a finding.
- Continue the same tester by message for follow-up rounds. Start a fresh tester for the verify round.

## Rules for developer prompts

- Point at the findings folder and the contract. Do not retell the findings or suggest fixes.
- Every fix comes with a test that proves it.
- A round that changes a rule searches every live document for it first.
- Seen is not fixed. A round that reports a defect in the milestone's path that it did not fix goes back to fix it. A defect outside the path goes to `plan/deferred/`.
- A report that claims a path works because a test covers a helper is sent back for a test that drives the real path in the game.

## Checks after every report

Run these in PowerShell before reporting to the owner or starting the next agent:

```powershell
$R = '<repository path>'
git -C $R log --oneline -3
git -C $R status --short
git -C $R status -sb | Select-Object -First 1      # commits ahead of origin
git -C $R worktree list
Get-Process arma3_x64, arma3server_x64 -ErrorAction SilentlyContinue | Select-Object Id, ProcessName, StartTime
Get-PSDrive C | Select-Object Used, Free
python $R/tools/okf_check.py
```

Look for a worktree left behind, game or server processes from an interrupted run, and run folders piling up under `%TEMP%`. If a subagent was denied a command, do not run it for the subagent. Tell the owner and offer it.

## Machine hygiene

- Steam must be running and logged in before any run; without it `arma3_x64.exe` exits at once.
- The machine is a cloud Windows VM. A game client window renders only while the owner is connected through Shadow PC. A dedicated server and headless clients need no window.
- Check free disk on `C:` before a full acceptance run.
- Leave alone what is not this session's: another project's game processes, and other sessions' worktrees.

## Handling interruptions

- A resumed session is not proof the old one ended. Before resuming an agent, list the running agents and check for an older `claude` process whose shells work in that agent's worktree.
- When the owner stops a subagent, inspect its worktree and the game processes, report what is left and ask. Do not restart or clean up on your own. When the owner resumes it, wait for its next report.
- A message to a subagent that is about to finish can arrive too late. Check the report for the item and send it again if it is missing.
- When the owner writes findings by hand, a framework developer round copies them word for word into the milestone's findings folder.

## Decisions the owner made about the loop

- 2026-11-04: MVP first. The loop serves each milestone's demo. Edge cases go to `plan/deferred/`.
- 2026-11-04: Local first. No CI runs the game. A phase closes on a green full acceptance run on the Windows test machine.
- 2026-11-04: Three roles: framework developer, test developer, technical writer. There is no frontend developer.
- 2026-11-04: Fresh worktree per round.
- 2026-11-05: The repeated demo of M0 exit criterion 2 runs 5 times, not 10, and only when M0 closes. Other rounds drive the demo once.
- 2026-11-05: M0 closes without the 5-run repeated demo. The owner chose to skip it for this close only; the developer's earlier 5-run and 10-run results stand.
- 2026-11-05: Every worktree lives inside the repository at `.worktrees/<role>-<topic>`, never beside it. `.worktrees/` is ignored by git.
- 2026-11-06: The owner told the coordinator to commit and push M1 and its tag itself once nothing needed a decision. That covered that one push only.
- 2026-11-06: The owner told the coordinator to commit and push M2 and its tag `0.0.3`. That covered that one push only.
- 2026-11-06: The owner dropped the firewall prompts for `shakedown.exe` seen during the M2 build: no finding, nothing deferred.
- 2026-11-06: The owner told the coordinator to run M3 from Build through Close, and to commit and push `main` and the tag `0.0.4` itself once nothing needed a decision. That covered that one push only. For M3 no subagent writes the Steam ID or the Steam name anywhere; a report that holds either is not passed on, and the owner is told it happened.
- 2026-11-06: The owner leaves the Windows Firewall prompts as they are and answers Cancel; no deferred item to make the framework listen on loopback only.
- 2026-11-06: `US-062` (a wait's condition that raises a script error ends the wait at once, `OBS-01`) stays in M4, and the owner wants it guaranteed there: before M4 closes, the coordinator confirms that `acceptance/coverage/m04.md` shows `US-062` passing.
- 2026-11-06: The owner told the coordinator to run M4 from Build through Close, and to commit and push `main` and the tag `0.0.5` itself once nothing needs a decision. That covers that one push only. The shakedown MCP server is registered only through the project's `.mcp.json`; no subagent adds it to a user or global Claude Code configuration or starts a Claude Code session of its own; the tester drives the server over its protocol. `US-046`'s third criterion runs in a test developer session on the Mac that the owner starts from the coordinator's prompt. Exit criteria 2 and 5 are the owner's together, in a fresh Claude Code session.
- 2026-11-06: For `US-003`'s second criterion (no `steam` process), the owner let a test developer stop the `steam` process by its id for that one check; the owner starts Steam and logs in again afterwards. The tester stops Steam only as the last step of its round, after every other game run.
- 2026-11-04: The owner pushes. The coordinator never pushes unless told to push for that one push.
- 2026-11-09: The owner told the coordinator to run M5 from Build through Close, and to commit and push `main` and the tag `0.0.6` itself once nothing needs a decision. That covers that one push only.
- 2026-11-09: `SPK-06` runs first in the M5 build round, before any work that depends on it. If it fails and its fallback (a per-user scheduled task, `INT-WIN-02` edited) changes what the spec promises beyond that, the coordinator stops and asks the owner.
- 2026-11-09: For M5 the shakedown MCP server is registered only through the project's `.mcp.json`; no subagent adds it to a user or global Claude Code configuration.
- 2026-11-09: For M5 only, for `SPK-06` and for `US-007`'s checks, a subagent may start a headless Claude Code session (`claude -p`) in `e2e/grappling-hook` whose only job is to be the MCP client that starts the session or finds it. It must end before the subagent reports. This overrides, for M5, the M4 rule against subagents starting Claude Code sessions.
- 2026-11-09: In M5, kills for `STP-03`, `US-035` and `NFR-RL-03` (`taskkill /F` of the runner) go by the process id of a runner the subagent started, never by name.
- 2026-11-09: M5 exit criterion 5, the demo, is the owner's, in a fresh interactive Claude Code session. The coordinator gives the commands, what to look for and the `PATH` step.
- 2026-11-09: As in M3 and M4, in M5 no subagent writes the Steam ID or the Steam name anywhere; a report that holds either is not passed on, and the owner is told. The identity search covers `SEC-04`'s `audit.jsonl` and `runner.log` as well.
- 2026-11-09: The owner told the coordinator to run M6 from Build through Close, and to commit and push `main` and the tag `0.0.7` itself once nothing needs a decision. That covers that one push only.
- 2026-11-09: `SPK-08`, and `SPK-01`'s part across a mission restart, run first in the M6 build round, before any work that depends on them. If either fails and its fallback changes what the spec promises beyond what `m06.md` names, the coordinator stops and asks the owner.
- 2026-11-09: `NFR-PF-02`'s target is set from `SPK-08`'s measured time over 10 reloads. The developer writes the number and how it was measured into the spec. The coordinator does not ask the owner for the number.
- 2026-11-09: M1 took `SPK-05`'s first fallback, not its second, so M6 exit criterion 3 does not apply. The close round says so in `docs/milestones/m06.md`.
- 2026-11-09: In M6 no subagent edits, builds or commits in `..\arma-grappling-hook`. Every check and test of a changed file works on the fixtures' installed copy, as `FIX-03` allows. The grappling-hook part of the demo is the owner's.
- 2026-11-09: The M5 leave to start headless Claude Code sessions does not carry over to M6. Testers drive the reload tool over the MCP protocol, as in M4. The shakedown MCP server is registered only through the project's `.mcp.json`.
- 2026-11-09: M6 exit criterion 5, the demo, is the owner's. When M6 is ready, the coordinator gives the commands to start the dedicated session with two clients on arma-grappling-hook, which function to change and how to rebuild it, the reload command, and what the report and the rerun of the M2 check should show.
- 2026-11-09: As in M3 to M5, in M6 no subagent writes the Steam ID or the Steam name anywhere; a report that holds either is not passed on, and the owner is told.
- 2026-11-09: The owner dropped M5's tester candidate that the contract gives no shape for a client request after the hello: no finding, nothing deferred. `US-053`'s proof by the closed connection, the marker and `audit.jsonl` stands.
- 2026-11-09: `SPK-08` failed in the M6 build round: Arma 3 holds a loose mod file it has read, shared for reading only, so it cannot be written, and a running process keeps reading the old content across a mission restart until it lets go of the file, at a time no wait or flush the research round tried controls (`research/27-spikes-reload.md`). The owner chose a bounded research round, before any spec change, to look for a way around the hold. The owner decides the fallback after that round reports.
- 2026-11-09: After the `SPK-08` research round the owner chose its fallback (a): every `.sqf` change is reloaded by a game restart in one step (stop the processes, change the file, start them), and `RLD-02`, `P-08`, `US-054`, `US-055`, `US-057`, `FIX-03`, the fixture contract and the M6 demo are edited to it. `NFR-PF-02` is set from the measured time of that reload over 10 reloads. Finding what makes the game release a file goes to `plan/deferred/`.
- 2026-11-09: The owner told the coordinator that when M6 closes it commits and pushes `main` and the tag `0.0.7` (as already allowed) and goes straight on into M7 without asking. The M7 push still waits for the owner.
- 2026-11-09: The owner is away for the rest of M6 and M7 and answers no questions. M6 runs through test, fix and verify; its demo (exit criterion 5) waits for the owner, so M6 stays `in-progress` and M7 starts once M6 has no open finding. M6 closes, is tagged `0.0.7` and is pushed only after the owner has run the demo. The M7 push waits for the owner.
- 2026-11-09: The same holds for M8: if the coordinator is still running when M7 has no open finding, M8 starts and runs the same way, its demo and push waiting for the owner.
- 2026-11-09: The owner told the coordinator to run every remaining milestone the same way while away, one after another: each runs through build, test, fix and verify and the next starts once it has no open finding; each demo, close, tag and push waits for the owner.
- 2026-11-09: The owner's demos of M6 to M10 all wait until the end, after M10's verify. Each milestone still runs its own test, fix and verify rounds before the next starts. Each close, tag and push waits for its demo.
- 2026-11-09: While the owner is away, where the loop would stop for the owner (a failed spike whose fallback changes the spec, a finding that questions the spec), the coordinator takes the developer's recommendation, records it here as taken in the owner's absence, and goes on. The owner may reverse it.
- 2026-11-12: Taken in the owner's absence (M9 build): `SPK-16` failed for the headless client only (it stayed up about 61 s after its dumps). Instead of m09.md's fallback (harness-written dumps), the developer kept the real fault and the engine's dumps and added a 5 s watchdog in the fixture extension that ends a process still running with `0xC0000005` (FIX-01, `skdfx_fnc_crash`); rerun 5 of 5 on every kind. Also taken: a heartbeat missing for 10 s is `lost` / `no-answer` (HLT-02, P-07), and a stopped clock is dated from the first heartbeat that shows it (DM-06). The owner may reverse any of them.
- 2026-11-13: The owner told the coordinator to push `main` with the milestones finished so far (M6 to M8 verified, M9 in its verify round), knowing their demos and reviews are still open. That push is of `main` only: no tag is made or pushed until each milestone closes after its demo. Later pushes still wait for the owner.
- 2026-11-13: For M10 only, the owner lifted the rule against working in `..\arma-grappling-hook`: one subagent ports the M00 checks there, in a worktree `.worktrees/shakedown-m10` on branch `shakedown/m10`, and commits only on that branch. It never touches that repository's `main`, never merges, never pushes, builds only inside its worktree, and leaves that repository's other worktrees alone. The owner reviews and merges. Every other rule about that repository stands.
- 2026-11-13: The owner reviewed the decisions taken in their absence from M6 to M10 (the `rebuild` key, `NFR-PF-02`, `US-057` criterion 2, PRD goal G2 left as it is, the M7 refusal rules, the M8 run-stop decisions and the `dedicated-2` setup, the M9 crash watchdog, the 10 s heartbeat loss, the clock and pause rules, the `INS-01` step rule, `DEF-25`, the M10 cleanup machine argument and the object lookup rule) and kept every one. G2's 30 s promise stays, with `DEF-21` holding the gap.
- 2026-11-13: On the owner's word the coordinator fast-forwarded arma-grappling-hook's `main` to `shakedown/m10` (`88e531c`, the ported M00 checks), not pushed. The worktree and branch stay while the M10 tester runs the ported suite from them. The merged `shakedown.toml` still points `mods.agh.path` at that worktree's build by an absolute path.
- 2026-11-13: The owner decided the tester candidates of M6 to M8: defer a client that hangs in its start waiting the full 180 s, and `session.json` giving `reason: null` after a start that failed with `Error 20` (both with no milestone); drop the rest (an empty `rpt_tail` for a client killed just after launch, the unnormalised reload path, the 46 s against 82 s restart timings, a refusal during a reset, the run-only refusal in `docs/dedicated-sessions.md`, and the five untested RUN-05 behaviours).
- 2026-11-13: The owner asked for a proper root `README.md` for mod authors (what Shakedown is and a quickstart) and a hand-written GitHub wiki that covers every feature in detail. The wiki is its own repository (`shakedown.wiki`); the GitHub repository is private with its wiki disabled, so the wiki is drafted in a local git repository beside the checkout and nothing is pushed until the owner enables the wiki and says so.
- 2026-11-13: On the owner's word the coordinator pushed Shakedown's `main` (`e7352f3`) and arma-grappling-hook's `main` (`34472bf`), no tags. The wiki stays local: the repository is private with its wiki disabled.
- 2026-11-09: No research round runs longer than about one hour of game runs. The coordinator sets that bound in every research prompt.

## Knowledge about the tooling

- 2026-11-05: On the Windows test machine `python3` is the Microsoft Store alias stub and does not run. Use `python` (3.14), as in `python tools/okf_check.py`.
- 2026-11-05: Baseline of the Windows test machine before M0: Steam running, no `arma3_x64` or `arma3server_x64` process, about 200 GB free on `C:`, about 400 folders under `%TEMP%` that are not this project's. `cargo`, `hemtt` (WinGet) and `gh` (logged in as `casvanluijtelaar`) are on the path. The bundle check passes with 114 concepts and lists 86 requirements that no story covers yet.
- 2026-11-06: After the machine had idled overnight, the first game start died at 40 s with `Error 20 reading file 'addons\sounds_f.pbo'`, a base-game file. The file was intact and read end to end at once afterwards. A start that fails reading a base-game PBO is the machine, not the framework: start again before calling it a finding.
- 2026-11-06: Baseline before M2: 16 GB of RAM with 10.9 GB free and Steam running, 18.9 GB commit limit with 11.1 GB free (the page file adds under 3 GB), 186 GB free on `C:`, no `arma3_x64` or `arma3server_x64` process, about 416 folders under `%TEMP%`. A dedicated server, two game clients and a headless client at once may come close to that. Check free memory as well as disk before every round that starts a dedicated session, and after it, to see that the memory came back.
- 2026-11-06: A dedicated server, two game clients and a headless client together peaked at 12.3 GB of memory in use on the machine, 6.8 GB of it in the four game processes. That fits in 16 GB with room to spare. After the round, 10.8 GB was free again.
- 2026-11-06: Steam stopped by itself partway through the M2 tester round, and every later start was refused. Before every round that starts the game, check that a `steam` process is running. If it is not, ask the owner to start Steam and log in; never start it for a subagent.
- 2026-11-06: The M2 build round left Windows Firewall inbound-allow rules for `shakedown.exe` in the main checkout and in a worktree, and for a cargo test binary, after the owner clicked Allow on the prompts. Windows asks only when a program listens on an address other than loopback.
- 2026-11-06: Every round that builds in a fresh worktree makes Windows Firewall prompt again for `shakedown.exe` and for cargo test binaries such as `spikes_m03-<hash>`, because each build path is a new program. Cancel blocks only other hosts; the runs on this machine go over loopback and pass.
- 2026-11-06: To check for the Steam identity without printing it, read the Steam ID and the account's names from `loginusers.vdf` and count hits only: in `git grep` at `HEAD`, in commit messages, and in the folders a run leaves. The coordinator kept such a script in its scratch folder during M3; it prints counts, never values. On this machine `loginusers.vdf` has no `MostRecent` key.
- 2026-11-09: A writer stamped an `addressed` entry with a made-up time later than its commit and the verify, which kept the finding open. Tell every subagent that writes an `addressed` or `verified` entry to take the time from the clock (`date`). The coordinator had the writer correct that one stamp to its commit time, though `acceptance/findings/format.md` says entries are never rewritten.
- 2026-11-06: The Mac pulls from `origin`, so a Mac round needs `main` pushed first. The owner had the coordinator push `main` without the tag before the M4 Mac round.
- 2026-11-09: The owner's demo in Claude Code needs `shakedown` on `PATH`; it is not on the user `PATH`. Prepend `C:\Users\Shadow\Documents\shakedown\target\shakedown` in that PowerShell window before `claude`. The demo session's `shakedown mcp` stays up until that Claude Code session ends.
- 2026-11-06: A tester's full acceptance run takes about 15 minutes with about 21 session starts; checks share sessions.
- 2026-11-09: PowerShell scripts do not run here (execution policy), and the auto-mode classifier refuses `-ExecutionPolicy Bypass`. The coordinator's count-only identity scan is a Python script that reads the Steam path from the registry with `winreg` and `loginusers.vdf`, and is run as `python -I steam_scan.py <extra folders>`. Since M5 the folders to search are `%LOCALAPPDATA%\Shakedown` (`runner.log`, `record.json`), the `.shakedown` folders under `e2e/` and `acceptance/` (`audit.jsonl`, `session.json`) and the rounds' scratch folders.
- 2026-11-09: Agent shells on this machine run in a job that forbids breakaway, so a runner they start goes through WMI (`SES-08`); `runner.log` names the way each start went. A runner started with a console got killed by a stray console event until the M5 build round gave the WMI start no console.
- 2026-11-09: Commands handed to the owner must run as pasted. A `<placeholder>` in a PowerShell line is a parse error; read the value in the command instead, as `(shakedown status --json | ConvertFrom-Json).runner_pid`.
- 2026-11-09: A round that ends with no finding has no verify round, so the coordinator asks the test round itself for the one full acceptance run before close. A fix that lands after that run needs the full run again at its verify.
- 2026-11-09: Baseline before M6: Steam running, no `arma3_x64` or `arma3server_x64` process, 10.8 GB of 16 GB free, 20.3 GB commit free, 172.5 GB free on `C:`, about 851 entries under `%TEMP%`, twice the M2 count.
- 2026-11-09: Steam stopped by itself again between the M6 build round and the research round after it, as in M2. That round did its web part and stopped before any game run.
- 2026-11-10: In the M6 verify round Steam stopped by itself three times and the coordinator's Claude Code process ended twice. Once Steam went during the US-057 reload, in which the check ends `p2` by its id; one Claude Code end came 0.7 s after US-035's `taskkill /F` of the runner by the id `status` gave. No link is proven. A full run that loses Steam partway is not a result: it fails every later session start with `precondition-failed` or exit code 53.
- 2026-11-11: The owner explained the Steam and Claude Code stops: the Shadow PC VM shuts itself down when the work idles or the owner's Mac sleeps, and Steam and the coordinator end with it. Shadow's documented rules: the VM shuts down 10 minutes after the stream closes, and after 1 to 2 hours with no user input even while the stream is open; sessions are capped at 4, 6 or 8 hours by plan; only Shadow PC Pro's "Always On" option keeps it running. Simulated input breaks Shadow's terms and `CLAUDE.md`. After every resume, check for `steam` before any round.
- 2026-11-11: `Error 20 reading file` on a base-game PBO happened again in the M6 full run, once on a server; and a client hung in its start with a silent RPT. Both were the machine: the same checks passed when run again.
- 2026-11-05: The Windows test machine has no global git identity. The repository's local config sets `user.name casvanluijtelaar` and `user.email cas.luijtelaar@umain.com`, as on the Mac; worktrees share it.

## State at the end of the last session

- 2026-11-13: M9 Invalid, not failed is built, tested and verified, `state: in-progress`, waiting only for the owner's demo before its close and the tag `0.0.10`. All eight stories pass; the full run of `ba0ae21` passed 271 of 277, five lost to `Error 20` and passed on rerun, the sixth m09-3. Exit criterion 2 (`NFR-RL-01`) held. Findings: `m09-1` (inspect of a client named no step) fixed in `b78ded3`; `m09-2` (a resumed runner lost every machine, an M5 regression from the heartbeat gap) fixed in `b78ded3`; `m09-3` (the server logs `Object not found` for a killed player's body about once in 40 kills) deferred as `DEF-25`, and the DEF-09 check leaves out only that body by net id. Also deferred: `DEF-24` (the order and check ceilings count a paused runner). For the owner to review: the decisions taken in their absence above, the 2 s pause rule of `HLT-02`, the `INS-01` step rule, and deferring m09-3. The Shadow stream cut of `SPK-15` and `US-059` was not driven. `main` was pushed at `1e97e92` on the owner's word. Next: M10, which needs the owner (its port is in arma-grappling-hook).
- 2026-11-12: M8 Walk away is built and tested with no finding, `state: in-progress`, waiting only for the owner's demo before its close and the tag `0.0.9`. `US-033`, `US-034`, `US-036` and `US-063` passed 48 of 48 over three runs; the full run at `202174d` (framework `f6b094f`) passed 243 of 243. For the owner to review, the developer's decisions: exit 2 for a stopped run, how `run stop` is parsed, Ctrl+C during a `run --setup` start, a stop during the mission restart of a `@reset mission` check, the ceiling read at run start, `session stop` during a run with `P-10` reordered, `run_stop`'s semantics, and the `dedicated-2` setup in `e2e/grappling-hook` that makes the M8 demo exit 1. Tester candidates: Ctrl+C while `run --setup` is still starting, a second Ctrl+C, a mission restart finishing before the stop, `session stop` during a run giving `session-stopped`, the stopped check's body actually ending in the game, and `session.json` with `reason: null` after an `Error 20` start. Next: M9, built while the owner is away.
- 2026-11-11: M7 Reload inside the mission is built, tested and verified, `state: in-progress`, waiting only for the owner's demo before its close and the tag `0.0.8`. `US-054` (rerun by a finding's tag after a game-restart reload) and `US-058` (a reload or reset during a run or a reload is refused, naming it) pass; `SPK-09`, level A, `RLD-05` and `DM-08` wait on `DEF-21`. Finding `m07-1` (the refusal during a reload did not name it) is fixed in `b2d48df` and verified; the full run at `b2d48df` passed 227 of 227. For the owner to review: a run counts as active until its result file is written, so a broken session's reload is refused until then; `RLD-01` and `cli.md` say a refusal names a reload by id and level. Candidates: a refusal during a reset is not driven (a reset takes under 1 s); `docs/dedicated-sessions.md` names only the run refusal. Next: M8, built while the owner is away.
- 2026-11-11: M6 Reload by restart is built, tested and verified, `state: in-progress`, waiting only for the owner's demo (exit criterion 5) before its close, the tag `0.0.7` and the push. `SPK-08` failed; the owner chose a game restart for every `.sqf` change, with a per-mod `rebuild` key (`RLD-06`); `SPK-01` passed across a mission restart; `NFR-PF-02` is under 90 s plus the mod's rebuild, measured 81.8 s over 10 reloads (the tester measured 46–48 s with a headless client). Finding `m06-1` (a reset failed when an enemy killed `p2` as it was put back) is fixed in `749d8d6` and verified. The last full run, at `d310b44`, passed 200 of 207; the 7 failures were an `Error 20` base-game PBO read and a client that hung in its start, plus one check the tester fixed, and they passed 8 of 8 when run again. Deferred: `DEF-20`, `DEF-21`, `DEF-22`. For the owner at the end: the tester's candidates (an empty `rpt_tail` for a client killed just after launch, a reload path returned unnormalised, the gap between 46 s and 82 s game-restart timings, a hung client start waiting the full 180 s), and the developer's spec decisions (the `rebuild` key, `US-057` criterion 2, the M7 edits, PRD goal G2 untouched). Next: M7, built while the owner is away.
- 2026-11-09: M5 A session that outlives its client is complete and tagged `0.0.6` on `0d6fabd`, pushed with `main` by the coordinator on the owner's go. `SPK-06` passed with no fallback: the client starts the runner through a breakaway relay, or through WMI where breakaway is refused; `INT-WIN-02` was edited in place and no promise changed. The build round found and fixed a WMI-started runner ended by a console event, and deferred `DEF-18` (auditing `wait`, `expect` and `inspect`) and `DEF-19` (one runner start over 10 s right after a kill, cause unknown). The developer edited `US-035` criterion 2 to say `status` shows `not-running` and starts no runner, as `P-01` and the contract say. The test round found `m05-1` (`runner.json` readable by Administrators, against `SEC-02`), fixed with a protected access list for the current user alone and verified; the last full acceptance run passed 191 of 191 at `fbf9b52`; `NFR-RL-03` held 5 of 5 in every round. The owner ran the demo in fresh Claude Code sessions and said everything went according to spec. No Steam ID or Steam name reached any commit, file, `runner.log`, `audit.jsonl` or report. Open for the owner: whether to defer or drop the tester's candidate that the contract gives no shape for a client request after the hello, so `US-053` is proven by the closed connection, a marker and `audit.jsonl`. Next is M6 Reload by restart (`plan/milestones/m06.md`, `0.0.7`); it starts only when the owner says so.
- 2026-11-09: M4 Agent tools is complete and tagged `0.0.5` on `f5a5db5`, pushed with `main` by the coordinator on the owner's go. The test developer found no framework defect; the full acceptance suite passed 171 of 171 at `db00a23`; `US-003`'s Steam criterion passed 3 of 3 with Steam stopped by the tester on the owner's leave; `US-046`'s third criterion passed 3 of 3 on the Mac; `US-062` passes in `acceptance/coverage/m04.md`, as the owner asked to have confirmed. Findings `m04-1` and `m04-2` were document gaps, answered by the writer and verified. The owner ran the demo in a fresh Claude Code session in `e2e/grappling-hook` and said everything worked as expected. The owner deferred `DEF-14` (death camera after reset), `DEF-15` (a refused start leaves a runner), `DEF-16` (a compile error listed twice) and `DEF-17` (a long result with no project), with no milestone; `DEF-13` (a cancelled `session_start` or `reset` keeps running) was deferred by the build round. The owner dropped the wording of the Steam refusal. No Steam ID or Steam name reached any commit, file or report. Next is M5 A session that outlives its client (`0.0.6`); it starts only when the owner says so.
- 2026-11-06: M3 Script errors and logs is complete and tagged `0.0.4` on `791fad2`, pushed with `main` by the coordinator on the owner's go. `SPK-07` and `SPK-13` passed with no fallback; no promise changed. The test developer round wrote no findings; the full acceptance suite passed 117 of 117 at `8d496a0`; exit criterion 2 held over the three acceptance runs; the owner ran the demo and said it looked good. `DEF-09` is fixed at its source (each machine deletes only its own entities). The owner's decisions on the tester's candidates: `DEF-11` (an RPT `File` path with a leading backslash, no milestone) and `DEF-12` (search what a failed start prints and keeps for the Steam identity, planned into M4 beside `DEF-10`); the wait-condition item is `US-062`, already in M4, which must be confirmed covered before M4 closes. No Steam ID or Steam name reached any commit, file or report. Next is M4 Agent tools (`plan/milestones/m04.md`, `0.0.5`); it starts only when the owner says so.
- 2026-11-06: M2 Dedicated sessions is complete and tagged `0.0.3` on `9008564`, not pushed; `main` is 8 commits ahead of `origin`. SPK-01, SPK-03 and the dedicated half of SPK-04 passed with no fallback; the contract did not change. Findings m02-1 (fixed) and m02-2 (a mistake in the check: it did not read a background `session start`'s pipes) are verified; the full acceptance suite passed 266 of 266 at `ecd9cf8`; the repeated demo passed 10 of 10 at the close; the owner ran the demo. The owner's decisions: `skd_fnc_create` should wait until every machine sees the object (`DEF-08`, no milestone, documented now); the Type_189 "Object not found" line after a reset is fixed at its source in M3, which must not grow code only to hide it (`DEF-09`); `status` keeps a failed start and its reason (`DEF-10`, M4); the firewall prompts for `shakedown.exe` are dropped. The framework's own M1 demo checks in `e2e/` use `player` and end `check-error` in a dedicated setup; the acceptance suite's M1 demo checks pass there; exit criterion 4 was accepted on that reading. For arma-grappling-hook, not Shakedown: the mod does not give a respawned unit its actions again. Next is M3 Script errors and logs (`plan/milestones/m03.md`); it starts only when the owner says so.
- 2026-11-06: M1 A grappling-hook check in singleplayer is complete, tagged `0.0.2` and pushed. SPK-04 (singleplayer) and SPK-12 passed, SPK-05 took its first fallback (a `Killed` handler), SPK-10 failed with no promise changed; the contract did not change. Findings m01-1, m01-2 and m01-3 (the owner's: a check id given n times runs n times, in the order given) are verified; the full acceptance suite passed 65 of 65; the owner ran the demo. The demo loads arma-grappling-hook from its `hemtt build` output in `..\arma-grappling-hook\.hemttout\build`, not a `hemtt dev` build. The owner dropped the candidate about CBA's unlocalised keybind group names in the unknown-mod message. A test comment at `crates/shakedown/tests/m01.rs:598` still says the `#2` marker ends the line; the next developer round in that file corrects it. Next is M2 Dedicated sessions (`plan/milestones/m02.md`); it starts only when the owner says so.
- 2026-11-05: M0 Walking skeleton is complete and tagged `0.0.1` (`a5b52f7`). Every exit criterion is met: the gates passed on GitHub in run 42080299853, the owner ran the demo, and the owner skipped the repeated demo at this close. The test developer round wrote no findings. Next is M1 "A grappling-hook check in singleplayer" (`plan/milestones/m01.md`); it starts only when the owner says so. The owner decided the tester's M0 deferral candidates: a `@check` name without a story or requirement ID is accepted (fix in M1 as a finding), a UTF-8 BOM hides the header (a deferred item planned into M1), `@setups` with an unknown setup is not reported by `list` (a deferred item for M2), and the wrong-argument message of the check API may lack a next step (the first M1 tester judges it against `NFR-US-01`; the coordinator names it as left uncovered by the M0 tester). Dropped: `//@check` without a space is accepted, a killed runner leaves a stale `runner.json` that the next command recovers from, and `session stop` with no session exits 4 as the contract says.
