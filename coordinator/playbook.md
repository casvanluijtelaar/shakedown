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
- 2026-11-04: The owner pushes. The coordinator never pushes unless told to push for that one push.

## Knowledge about the tooling

- 2026-11-05: On the Windows test machine `python3` is the Microsoft Store alias stub and does not run. Use `python` (3.14), as in `python tools/okf_check.py`.
- 2026-11-05: Baseline of the Windows test machine before M0: Steam running, no `arma3_x64` or `arma3server_x64` process, about 200 GB free on `C:`, about 400 folders under `%TEMP%` that are not this project's. `cargo`, `hemtt` (WinGet) and `gh` (logged in as `casvanluijtelaar`) are on the path. The bundle check passes with 114 concepts and lists 86 requirements that no story covers yet.
- 2026-11-06: After the machine had idled overnight, the first game start died at 40 s with `Error 20 reading file 'addons\sounds_f.pbo'`, a base-game file. The file was intact and read end to end at once afterwards. A start that fails reading a base-game PBO is the machine, not the framework: start again before calling it a finding.
- 2026-11-06: Baseline before M2: 16 GB of RAM with 10.9 GB free and Steam running, 18.9 GB commit limit with 11.1 GB free (the page file adds under 3 GB), 186 GB free on `C:`, no `arma3_x64` or `arma3server_x64` process, about 416 folders under `%TEMP%`. A dedicated server, two game clients and a headless client at once may come close to that. Check free memory as well as disk before every round that starts a dedicated session, and after it, to see that the memory came back.
- 2026-11-06: A dedicated server, two game clients and a headless client together peaked at 12.3 GB of memory in use on the machine, 6.8 GB of it in the four game processes. That fits in 16 GB with room to spare. After the round, 10.8 GB was free again.
- 2026-11-06: Steam stopped by itself partway through the M2 tester round, and every later start was refused. Before every round that starts the game, check that a `steam` process is running. If it is not, ask the owner to start Steam and log in; never start it for a subagent.
- 2026-11-06: The M2 build round left Windows Firewall inbound-allow rules for `shakedown.exe` in the main checkout and in a worktree, and for a cargo test binary, after the owner clicked Allow on the prompts. Windows asks only when a program listens on an address other than loopback.
- 2026-11-05: The Windows test machine has no global git identity. The repository's local config sets `user.name casvanluijtelaar` and `user.email cas.luijtelaar@umain.com`, as on the Mac; worktrees share it.

## State at the end of the last session

- 2026-11-06: M2 Dedicated sessions is complete and tagged `0.0.3` on `9008564`, not pushed; `main` is 8 commits ahead of `origin`. SPK-01, SPK-03 and the dedicated half of SPK-04 passed with no fallback; the contract did not change. Findings m02-1 (fixed) and m02-2 (a mistake in the check: it did not read a background `session start`'s pipes) are verified; the full acceptance suite passed 266 of 266 at `ecd9cf8`; the repeated demo passed 10 of 10 at the close; the owner ran the demo. The owner's decisions: `skd_fnc_create` should wait until every machine sees the object (`DEF-08`, no milestone, documented now); the Type_189 "Object not found" line after a reset is fixed at its source in M3, which must not grow code only to hide it (`DEF-09`); `status` keeps a failed start and its reason (`DEF-10`, M4); the firewall prompts for `shakedown.exe` are dropped. The framework's own M1 demo checks in `e2e/` use `player` and end `check-error` in a dedicated setup; the acceptance suite's M1 demo checks pass there; exit criterion 4 was accepted on that reading. For arma-grappling-hook, not Shakedown: the mod does not give a respawned unit its actions again. Next is M3 Script errors and logs (`plan/milestones/m03.md`); it starts only when the owner says so.
- 2026-11-06: M1 A grappling-hook check in singleplayer is complete, tagged `0.0.2` and pushed. SPK-04 (singleplayer) and SPK-12 passed, SPK-05 took its first fallback (a `Killed` handler), SPK-10 failed with no promise changed; the contract did not change. Findings m01-1, m01-2 and m01-3 (the owner's: a check id given n times runs n times, in the order given) are verified; the full acceptance suite passed 65 of 65; the owner ran the demo. The demo loads arma-grappling-hook from its `hemtt build` output in `..\arma-grappling-hook\.hemttout\build`, not a `hemtt dev` build. The owner dropped the candidate about CBA's unlocalised keybind group names in the unknown-mod message. A test comment at `crates/shakedown/tests/m01.rs:598` still says the `#2` marker ends the line; the next developer round in that file corrects it. Next is M2 Dedicated sessions (`plan/milestones/m02.md`); it starts only when the owner says so.
- 2026-11-05: M0 Walking skeleton is complete and tagged `0.0.1` (`a5b52f7`). Every exit criterion is met: the gates passed on GitHub in run 42080299853, the owner ran the demo, and the owner skipped the repeated demo at this close. The test developer round wrote no findings. Next is M1 "A grappling-hook check in singleplayer" (`plan/milestones/m01.md`); it starts only when the owner says so. The owner decided the tester's M0 deferral candidates: a `@check` name without a story or requirement ID is accepted (fix in M1 as a finding), a UTF-8 BOM hides the header (a deferred item planned into M1), `@setups` with an unknown setup is not reported by `list` (a deferred item for M2), and the wrong-argument message of the check API may lack a next step (the first M1 tester judges it against `NFR-US-01`; the coordinator names it as left uncovered by the M0 tester). Dropped: `//@check` without a space is accepted, a killed runner leaves a stale `runner.json` that the next command recovers from, and `session stop` with no session exits 4 as the contract says.
