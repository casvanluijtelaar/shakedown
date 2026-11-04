# Coordinator playbook

Load this file at the start of every coordinator session, together with `CLAUDE.md`, which it does not repeat. It holds the loop, the frame of every subagent prompt, the checks after every report, the decisions the owner made about how to run the loop, and the state at the end of the last session. Product decisions never go here; they go into the spec through a backend round. A coordinator adds what it learns to the section it belongs in, as a dated entry.

## The coordinator's job

The coordinator writes no code, no tests and no spec, and edits no repository file except this playbook. It starts subagents in one of the four roles from `CLAUDE.md` (backend developer, test developer, frontend developer, technical writer), each with one task, reads each final report, checks the repository and the machine after each one, and decides what to start next or stops and asks the owner. Every repository change goes through a subagent in the role that owns the file. A plan or spec change is a backend developer's task, even a one-line one. The coordinator never reads the code.

## The loop for one milestone

1. **Build.** A backend developer builds the milestone. If the report leaves deliverables open, start another backend round on what remains. That round sorts the open items by the plan's requirement IDs, builds this milestone's items and moves the rest to the milestone that owns them. It may not defer an item because it is hard.
2. **Test.** A test developer builds the checks for the milestone and writes findings. If it calls the work done but lists stories it did not drive, start another tester round for those first.
3. **Fix.** Show the owner each set of findings and go on without waiting. Start a backend developer on the open findings in `acceptance/findings/mNN/`. A finding about a client app goes to that app's frontend developer. The backend grounds a finding that questions the spec before it fixes it.
4. **Verify.** Start a fresh test developer to verify every new `addressed` entry and add `verified` entries. A finding that does not hold becomes a new finding that links to the old one.
5. Repeat 3 and 4 until every finding is verified and every story in the milestone has a passing check.
6. **Frontend.** When there is a client app: a frontend developer builds the UI for the milestone's stories against the contract.
7. **Close.** A small backend round sets the milestone's `state` to `complete`, tags the release if the milestone has a `version`, and names the next milestone. Report that `main` is ready to push.

### Keep the fix loop narrow

- Every finding names its checks. The suite takes a filter that runs only the named checks.
- A backend fix round runs the checks its findings name, through the filter, before it hands back. A finding whose checks stay red is not addressed.
- A backend round runs only the tests of the packages it touched and the one end-to-end test that covers its change.
- A verify round runs only the checks behind the new `addressed` entries, three times. That is the flake check.
- The full acceptance suite runs once when the tester's first round for the milestone is done, and once at the last verify before the milestone closes.
- A round that fixes a CI failure runs only the failing tests by full name, once, plus the unit test that proves the fix.

## When to stop and ask the owner

| Stop | By default | What lifts it |
|---|---|---|
| A finding questions the spec, and the research and the client requirements cannot settle it | Show the finding and the backend's report. Fix nothing for it. The other findings go on. | Nothing. The owner decides. |
| The end of a milestone: every finding verified | Report that `main` is ready to push. Ask whether to push and whether to start the next milestone. | "Go on to the next milestone without asking." The push still waits for the owner. |
| An agent the owner stopped, or a step only a person can do | Report what is left and ask. | Nothing. |

A lift holds until the owner takes it back, also in a resumed session. Record it with its date under "Decisions the owner made about the loop". Do not ask for a push between rounds.

## Sequencing rules

- One backend developer at a time. A frontend round may run beside it when the two touch separate folders.
- Do not start the next milestone while the current one has open findings.
- A milestone with screens starts with the backend, which walks every screen and action against the contract and builds what the client lacks.
- When a finding can change the contract the frontend reads, the backend answers it before the next frontend round.

## The frame of every subagent prompt

> You are the [role] for /Users/casvanluijtelaar/Projects/arma-automation-tests. You run as a subagent. A coordinator reads your final report. Nobody watches you live and nobody answers questions.
>
> Read CLAUDE.md in full and follow it. Do not open [folders]. Do not edit [folders].
>
> [The task, pointing at files. No summary of findings and no suggested fix.]
>
> - Answer your own questions from the user stories and the spec. Where they are silent or disagree, choose the reading that fits the spec's design principles, write it into the spec, and name it in your report. A finding that questions the spec is the exception: ground it in the research and the client requirements, or report it as open.
> - Give every background shell a ceiling: a counted loop, never an open-ended wait. Stop your processes by pid before you report.
> - Run `git worktree list` first. Do not touch another session's worktree or containers.
> - Use `[scratch folder]/[worktree name]/` for temporary files. Nothing else in the scratch folder is yours.
> - Never use `git stash`. Never use memory. Publish nothing outside this machine.
> - Commit when you are done. This overrides the rule of waiting to be asked. Work in `git worktree add ../arma-automation-tests-[role]-[topic] -b [role]/[topic]`. When done, rebase on main, fast-forward merge into main, remove the worktree and delete the branch. If the main checkout has uncommitted changes, do not merge, and say so. Do not push.
>
> End with a short numbered report: each item done or not, the tests you ran and their results, the decisions you made and where you recorded them, the commit hashes, that the worktree is removed, what you tore down, and that no background shell is running.

Never pass a model override. Every subagent runs on the coordinator's model, however small the task.

## Rules for tester prompts

- Never tell a tester what to test. It derives every check from the stories and requirements. The coordinator may say which findings to verify and what a previous tester left uncovered.
- A developer's or frontend's claim about which checks must change is a claim to check against the contract, not an instruction.
- An answer that changed the contract must itself agree with the stories and requirements, or it is a finding.
- No failure is load. A failure in one run out of three is a flaky product (a finding) or a flaky check (the tester fixes it).
- A reason not to drive a story is checked against the story. An attacker brings their own client.
- A failure caused only by another session's work in progress is not a finding. Say so.
- Continue the same tester by message for follow-up rounds. Start a fresh tester for the verify round.
- A tester reads a client app's README for test ids, never any source.

## Rules for developer prompts

- Point at the findings folder and the contract. Do not retell the findings or suggest fixes.
- The backend reads only `acceptance/README.md` and `acceptance/findings/`, and writes only `addressed` entries and findings for a frontend.
- Every fix comes with a test that proves it, ideally one confirmed to fail without the fix.
- A round that changes a rule searches every live document for it first.
- A round that rewrites messages or adds a field runs every end-to-end test that asserts on that text or field.
- Seen is not fixed. A round that reports a defect it did not fix goes back to fix it.

## Checks after every report

```bash
R=/Users/casvanluijtelaar/Projects/arma-automation-tests
git -C $R log --oneline -3
git -C $R status --short
git -C $R status -sb | head -1        # commits ahead of origin
git -C $R worktree list
docker ps --format '{{.Names}}'
ps -Ao pid,etime,command | grep -E "until \[|<server binary>|<test runner>" | grep -v grep
python3 $R/tools/okf_check.py
```

Look for a worktree left behind, containers from an interrupted run and shells that still loop. If a subagent was denied a command, do not run it for the subagent. Tell the owner and offer it as a `!` command.

## Machine hygiene

- Check `df -h` and `docker system df` before a full acceptance run.
- Remove an interrupted run's containers by its name prefix, with `-v`. Never prune named volumes; they hold other projects' data.
- macOS has no `timeout` command. Bound a wait with a counted loop.
- Leave alone what is not this session's: other projects' containers, a dev server from the main checkout, other sessions' worktrees.

## Handling interruptions

- A resumed session is not proof the old one ended. Before resuming an agent, list the running agents and check `ps` for an older session working in that agent's worktree.
- When the owner stops a subagent, inspect the worktree and containers, report what is left and ask. Do not restart or clean up on your own.
- A message to a subagent that is about to finish can arrive too late. Check the report for the item and send it again if it is missing.
- When the owner writes findings by hand in their own worktree, a backend round copies them word for word into the milestone's findings folder.

## Decisions the owner made about the loop

- 2026-11-04: Fresh worktree per round. Revisit if the stack's cold build turns out slow or its cache is keyed on the source path.
- 2026-11-04: The owner pushes. The coordinator never pushes unless told to push and monitor CI for that one push.

## State at the end of the last session

- 2026-11-04: Repository set up. No research, spec or plan yet. Next is step 1, research, which needs `spec/client-requirements.md` filled in by the owner.
