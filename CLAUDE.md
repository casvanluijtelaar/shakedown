# Arma automation framework

This repository is built with agentic engineering: the specs, user stories and contract are the source, and models write all of the code. The method is the Eidra agentic engineering guide (`git.aurora.eidra.com/Eidra/agentic-engineering-guide`). This file is the map: which document answers which question, and the working rules. It does not restate the specs, and it holds no status. The state of each milestone lives in its concept in `plan/milestones/`.

## Map

Start at the `index.md` of a bundle and open only what the task needs.

| Bundle | Index | Needed when |
|---|---|---|
| Research | [research/index.md](research/index.md) | Grounding a spec change or settling a finding that questions the spec. Never as a source of what to build directly. |
| Specification | [spec/index.md](spec/index.md) | Every task. What the system must do. |
| Plan | [plan/index.md](plan/index.md) | Starting or closing a milestone, deferring work. |
| Documentation | [docs/index.md](docs/index.md) | How the built system works. The contract for the tester and frontends. |
| Acceptance docs | [acceptance/docs/index.md](acceptance/docs/index.md) | Test developer only: how the harness works. |
| Findings | [acceptance/findings/index.md](acceptance/findings/index.md) | Writing, answering or verifying a finding. The format is in [acceptance/findings/format.md](acceptance/findings/format.md). |
| Coverage | [acceptance/coverage/index.md](acceptance/coverage/index.md) | Closing a milestone. |

Documents, by task:

- **`spec/client-requirements.md`** is the client's own requirements, kept as given. Needed for research, for writing the spec, and for grounding any spec change. It wins over the research on goals and scope.
- **`spec/00-overview.md`** to **`spec/06-open-questions.md`** (written in step 2): overview and scope, domain model with status machines, processes, functional requirements, integrations, non-functional requirements, open questions.
- **`spec/07-tech-stack.md`** (written in step 2): what the system is built from and why. Needed before any code, dependency or CI change. It names the source code folders and the contract file.
- **`spec/roles/`** and **`spec/stories/<role>/`**: who the system serves and what each story promises. Needed for every check, every API or UI change and every error path.
- **`plan/implementation-plan.md`** (written in step 3): phases, milestone order and the rules behind the order. **`plan/milestones/mNN.md`**: one milestone's goal, demo, stories and exit criteria.
- **`coordinator/playbook.md`**: coordinator sessions only. No role reads it.
- **`tools/okf_check.py`**: the bundle check. Run `python3 tools/okf_check.py` after adding, moving or editing any concept. It also prints open findings and uncovered requirements and stories.

Source code folders: none yet. `spec/07-tech-stack.md` names them, and this line is updated to list them when they are created.

## Which role you are working in

Every session takes one of four roles. Only the backend developer and the technical writer share folders, and the technical writer's entry says how. If the request does not make it clear which role, ask before touching anything. A research or spec session in steps 1 to 3 works as the backend developer.

**Backend developer.** Owns the source code folders, `research/`, `spec/`, `plan/`, `docs/` and every unit, integration and end-to-end test in the code. Of the acceptance suite it reads two things and no others: `acceptance/README.md` to know how to run it, and `acceptance/findings/` to see what it reported. It runs the checks a finding names to close it. Running a check is not reading it: the suite's source stays unread and no check is edited. In `acceptance/findings/` it writes only `addressed` entries and findings for a frontend developer, with `raised_by: backend`. A failing acceptance check is fixed in the code, never in the check.

**Test developer.** Owns `acceptance/`, which includes `acceptance/findings/` and `acceptance/coverage/`. Works from the contract: `docs/`, the contract file `spec/07-tech-stack.md` names, the requirements and the user stories in `spec/`, and the milestones in `plan/`. It does not open the source code folders and does not edit them. It may start and drive the built system, because running it is not reading it. A defect goes in `acceptance/findings/` for the backend developer. Fixing it is not the test developer's to do.

**Frontend developer.** Owns one client app folder, named in `spec/07-tech-stack.md` when there is one. Works from the contract: `docs/`, the contract file, the client types generated from it and the user stories. It does not open the backend source folders and does not edit them. It may start and drive the built system. A missing or wrong operation goes in `acceptance/findings/` as a finding with `raised_by: frontend`, for the backend developer. In `acceptance/` it writes only its own findings and the `addressed` entries on findings raised for it.

**Technical writer.** Works on `spec/`, `docs/` and the root `README.md`. Writes no code and runs no tests or checks. It does not edit the source code folders, client app folders or `acceptance/`, except to write findings and `addressed` entries on document gaps. It may read the source to confirm what a document says about the built system. The backend developer also edits `spec/` and `docs/` when its work changes what they say. The technical writer edits them when no code changes: wording, structure, a gap or a disagreement between two documents, and the spec of work not yet built. An edit that changes what the built system must do goes in `acceptance/findings/` for the backend developer.

The split is what makes the acceptance suite worth running. A check written from the source asserts what the code already does, so it passes by construction and proves nothing. A tester who fixes the defect has spent the independent reading of the contract that made the defect visible, and nobody is left holding it.

## How documents are written

Every markdown file in `research/`, `spec/`, `plan/`, `docs/`, `acceptance/docs/`, `acceptance/findings/` and `acceptance/coverage/` is an OKF v0.2 concept. It starts with frontmatter that has a `type`, a `title` and a one-sentence `description`, plus the keys of its type:

- `Research`: `sources`, each entry `{ id, resource, read }`, with the `id` as the label of the footnotes it backs.
- `Specification`: none.
- `Role`: `id`, `group`.
- `User Story`: `id` (`US-nnn`), `role`, `kind` (`feature`, `error`, `mistake`, `system fail` or `malicious intent`), `requirements`.
- `Plan`: none.
- `Milestone`: `id` (`MN`), `phase`, `order`, `state` (`planned`, `in-progress`, `complete`), `stories`, `requirements`, `version`.
- `Deferred`: `id`, `state` (`open`, `planned`, `done`), `milestone`.
- `Documentation`: none.
- `Milestone Report`: `milestone`.
- `Finding`: `id` (`mNN-n`), `milestone`, `raised_by`, `for`, `checks`, `stories`, `addressed` (`{ by, at, note }`), `verified` (`{ by, at }`).
- `Coverage`: `milestone`, `run` (`{ at, sha }`), `results` (`{ story, state, checks }`).

Every folder has an `index.md` with one line per concept: `* [Title](file.md) - description`. The line for a subfolder links to the folder and ends in `/`. The index at a bundle root carries `okf_version: "0.2"`. Whoever adds, renames or removes a file updates the index in the same commit.

Every `by` value is an OKF actor: `human:casvanluijtelaar` for the owner, `claude-code/<role>` for an agent, with the role `research`, `backend`, `tester`, `frontend`, `writer` or `coordinator`, and `process:<id>` for an automated process such as `process:ci`.

`status` is only `draft`, `stable` or `deprecated`. The progress of work goes in `state`. Review shows in `verified`. Never write a `log.md`.

**Requirement IDs.** Every requirement, rule and process has a stable ID of the form `AREA-nn` (such as `R-01`, `NFR-PF-02`, `B3`). An ID is defined where it is written in bold, `**R-01**`, at the start of a list item, heading or table row in a `spec/*.md` file. IDs are never reused. Commits, test names, check names, findings and comments use the ID instead of restating the requirement. A test or check that proves a story carries the story ID in its name.

At least 40 percent of the stories in each milestone are not `feature` stories, and all four failure kinds are covered.

## Working rules

- **The spec is edited in place.** When work reveals a gap, change the original file so it reads as if it had always said that. No addendum files, no "updated" or "new in M2" markers, no revision notes. Git holds the history.
- **Write for the current state.** Never word an edit as a change from what came before. Do not write "previously", "no longer", "changed from", "used to be", "now". State what holds. A comment explains what the code does and why it has that shape, never the bug it fixed.
- **A changed rule is searched first.** Before changing a rule, search every live document (every concept not `deprecated`) for it, and change every place.
- **Never use agent memory.** A rule about how to work here goes into this file. What to build goes into `spec/`, in what order into `plan/`, how the built system works into `docs/`. If a fact is worth remembering, it is worth committing.
- **No TODO markers in code.** Deferred work is a `Deferred` concept in `plan/deferred/` with the sections summary, current behaviour, the gap, options, recommendation, acceptance criteria.
- **Nothing is left running.** A test run stops every container, server and process it started, passed or failed, by pid, never by name pattern. If something has to stay up, say so and give the command that stops it. Use unusual ports for this project's servers. Bound every wait with a counted loop; macOS has no `timeout`.
- **Look up versions instead of recalling them.** Every dependency version is looked up in its registry at the time of the change.
- **Run only the tests the turn touched.** Before a turn ends, run the tests of the packages you changed and `python3 tools/okf_check.py` if you touched a document. CI runs everything on every push. The acceptance suite does not run in CI.
- **Never sleep a fixed time in a test.** Poll for the outcome with a ceiling of a few seconds.
- **Committing and pushing are the owner's call.** Wait to be asked, unless a coordinator's prompt says otherwise. Never push unless told to.
- **Git: trunk based.** All work lands on `main`. When sessions run in parallel, each works in its own worktree, `../arma-automation-tests-<role>-<topic>` on branch `<role>/<topic>`, rebases on `main` and fast-forward merges. Never use `git stash`: the stash list is shared by every worktree.
- **You do the visual checks.** A frontend session writes the code; the owner looks at the result and says what is wrong. UI conventions that get settled go into this file.
- **Do not add a dependency for something you can write in an afternoon.**
