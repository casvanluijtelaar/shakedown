# Shakedown

Shakedown is a test automation framework for Arma 3 addons and mods, in singleplayer and on a dedicated server: Playwright for the game. This repository is built with agentic engineering, following the Eidra agentic engineering guide (`git.aurora.eidra.com/Eidra/agentic-engineering-guide`) as adapted below: the specs, user stories and contract are the source, and models write all of the code. This file is the map, which document answers which question, and the working rules. It does not restate the specs, and it holds no status. The state of each milestone lives in its concept in `plan/milestones/`.

## How this project runs

- **MVP first.** A working framework early beats a careful one late. Each milestone takes the shortest path to its demo. Edge cases, hardening and `should` and `could` requirements wait until the MVP works end to end, and then come in only where use shows they matter. An edge case that no story of the current milestone promises goes into `plan/deferred/`, not into the milestone.
- **Local first.** Everything that starts the game runs on the Windows test machine. There is no CI/CD that runs the game. GitHub Actions runs only the gates that need no game: the bundle check, and the framework's own tests that do not start Arma 3.
- **The first user is `arma-grappling-hook`.** Its milestones demo against that mod, or against a small fixture mod in this repository. Nothing in the framework is specific to one mod.

## Map

Start at the `index.md` of a bundle and open only what the task needs.

| Bundle | Index | Needed when |
|---|---|---|
| Research | [research/index.md](research/index.md) | Grounding a spec change or settling a finding that questions the spec. Never as a direct source of what to build. |
| Specification | [spec/index.md](spec/index.md) | Every task. What the framework must do. |
| Plan | [plan/index.md](plan/index.md) | Starting or closing a milestone, deferring work. |
| Documentation | [docs/index.md](docs/index.md) | How the built framework works. Part of the contract for the tester. |
| Acceptance docs | [acceptance/docs/index.md](acceptance/docs/index.md) | Test developer only: how the harness works. |
| Findings | [acceptance/findings/index.md](acceptance/findings/index.md) | Writing, answering or verifying a finding. The format is in [acceptance/findings/format.md](acceptance/findings/format.md). |
| Coverage | [acceptance/coverage/index.md](acceptance/coverage/index.md) | Closing a milestone. |

Documents, by task:

- **`spec/client-requirements.md`** is the PRD: the problem, goals, users, scope, the `CR-nn` requirements with their priority, the key scenarios, the constraints and the first release. Needed for research, for writing the spec, and for grounding any spec change. It wins over the research on goals and scope. Section 2, "How we build it", and section 9, "First release", decide what a milestone may leave out.
- **`spec/00-overview.md`** to **`spec/06-open-questions.md`** (written in step 2): overview and scope, domain model with status machines, processes, functional requirements, integrations, non-functional requirements, open questions.
- **`spec/07-tech-stack.md`** (written in step 2): what the framework is built from and why. Needed before any code, dependency or gate change. It names the source code folders and the contract files.
- **The contract** (written in step 2, named in `spec/07-tech-stack.md`): the framework's public interface, which the tester works from and never past. That is the SQF check API, the command line, the agent tools and the format of the result file. A change to any of them is a deliberate edit to the contract file first.
- **`spec/roles/`** and **`spec/stories/<role>/`**: who the framework serves and what each story promises. Needed for every check, every interface change and every error path.
- **`plan/implementation-plan.md`** (written in step 3): phases, milestone order and the rules behind the order. **`plan/milestones/mNN.md`**: one milestone's goal, demo, stories and exit criteria.
- **`coordinator/playbook.md`**: coordinator sessions only. No role reads it.
- **`tools/okf_check.py`**: the bundle check. Run `python3 tools/okf_check.py` after adding, moving or editing any concept. It also prints open findings and uncovered requirements and stories.

Source code folders: none yet. `spec/07-tech-stack.md` names them, and this line is updated to list them when they are created.

## Which role you are working in

Every session takes one of three roles. Only the framework developer and the technical writer share folders, and the technical writer's entry says how. If the request does not make it clear which role, ask before touching anything. A research, spec or plan session in steps 1 to 3 works as the framework developer.

**Framework developer.** Owns the source code folders, any fixture mods, `research/`, `spec/`, `plan/`, `docs/` and every unit, integration and end-to-end test in the code. Of the acceptance suite it reads two things and no others: `acceptance/README.md` to know how to run it, and `acceptance/findings/` to see what it reported. It runs the checks a finding names to close it. Running a check is not reading it: the suite's source stays unread and no check is edited. In `acceptance/findings/` it writes only `addressed` entries. A failing acceptance check is fixed in the framework, never in the check.

**Test developer.** Owns `acceptance/`, which includes `acceptance/findings/` and `acceptance/coverage/`. Works from the contract: `docs/`, the contract files, the requirements and the user stories in `spec/`, and the milestones in `plan/`. It does not open the source code folders or the fixture mods' source, and does not edit them. The one exception is the installed copy of the fixtures: a reload check may replace the reload target files that `spec/contract/fixture-mod.md` names, with the content it allows, and puts them back after. It may start and drive the built framework and the game, because running them is not reading them. A defect goes in `acceptance/findings/` for the framework developer. Fixing it is not the test developer's to do.

**Technical writer.** Works on `spec/`, `docs/` and the root `README.md`. Writes no code and runs no tests or checks. It does not edit the source code folders or `acceptance/`, except to write findings and `addressed` entries on document gaps. It may read the source to confirm what a document says about the built framework. The framework developer also edits `spec/` and `docs/` when its work changes what they say. The technical writer edits them when no code changes: wording, structure, a gap or a disagreement between two documents, and the spec of work not yet built. An edit that changes what the framework must do goes in `acceptance/findings/` for the framework developer. A writer round runs only when a document needs work and no code changes.

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
- `Finding`: `id` (`mNN-n`), `milestone`, `raised_by` (`test`, `developer` or `writer`), `for` (`developer` or `writer`), `checks`, `stories`, `addressed` (`{ by, at, note }`), `verified` (`{ by, at }`).
- `Coverage`: `milestone`, `run` (`{ at, sha }`), `results` (`{ story, state, checks }`).

Every folder has an `index.md` with one line per concept: `* [Title](file.md) - description`. The line for a subfolder links to the folder and ends in `/`. The index at a bundle root carries `okf_version: "0.2"`. Whoever adds, renames or removes a file updates the index in the same commit.

Every `by` value is an OKF actor: `human:casvanluijtelaar` for the owner, `claude-code/<role>` for an agent, with the role `research`, `developer`, `tester`, `writer` or `coordinator`, and `process:<id>` for an automated process such as `process:ci`.

`status` is only `draft`, `stable` or `deprecated`. The progress of work goes in `state`. Review shows in `verified`. Never write a `log.md`.

**Requirement IDs.** Every requirement, rule and process has a stable ID of the form `AREA-nn` (such as `CR-01`, `R-01`, `NFR-PF-02`, `B3`). An ID is defined where it is written in bold, `**R-01**`, at the start of a list item, heading or table row in a `spec/*.md` file. IDs are never reused. Commits, test names, check names, findings and comments use the ID instead of restating the requirement. A test or check that proves a story carries the story ID in its name.

**Failure stories.** Until the MVP is done, every `feature` story in a milestone has at least one story of another kind next to it, for the way that feature most plausibly fails. After the MVP, the plan sets the share per milestone. `malicious intent` stories wait for the `CR-13` work unless a `must` requirement needs one.

## Working rules

- **Shortest path to the demo.** Build what the milestone's stories and exit criteria need, and stop. Do not add options, abstractions, hardening or error handling no story asks for. Note what you left out in the milestone report, and put anything that matters into `plan/deferred/`.
- **The spec is edited in place.** When work reveals a gap, change the original file so it reads as if it had always said that. No addendum files, no "updated" or "new in M2" markers, no revision notes. Git holds the history.
- **Write for the current state.** Never word an edit as a change from what came before. Do not write "previously", "no longer", "changed from", "used to be", "now". State what holds. A comment explains what the code does and why it has that shape, never the bug it fixed.
- **A changed rule is searched first.** Before changing a rule, search every live document (every concept not `deprecated`) for it, and change every place.
- **Never use agent memory.** A rule about how to work here goes into this file. What to build goes into `spec/`, in what order into `plan/`, how the built framework works into `docs/`. If a fact is worth remembering, it is worth committing.
- **No TODO markers in code.** Deferred work is a `Deferred` concept in `plan/deferred/` with the sections summary, current behaviour, the gap, options, recommendation, acceptance criteria.
- **Nothing is left running.** A run stops every game, server and helper process it started, passed or failed, by process id, never by name. Before a turn ends, list the `arma3_x64` and `arma3server_x64` processes and stop any you started. If something has to stay up, such as a session the owner will use, say so and give the command that stops it. Bound every wait with a counted loop.
- **One game at a time.** The test machine has one Steam session, one set of game ports and limited memory. Never start the game while another session's run holds it.
- **Never take over the desktop.** No code moves the machine's mouse or types on its keyboard. Input goes to the game only.
- **Fetch the web cheaply.** Search with WebSearch, read pages with WebFetch or `curl`, and read GitHub with `gh`. Use the Firecrawl tools only for a page none of those can read; their credits are limited.
- **Look up versions instead of recalling them.** Every dependency version, including Arma 3, CBA and ACE3, is looked up at the time of the change.
- **Run only the tests the turn touched.** Before a turn ends, run the tests of the parts you changed and `python3 tools/okf_check.py` if you touched a document. The full acceptance suite runs only when the driver asks for it.
- **Never sleep a fixed time in a test or a check.** Poll for the outcome with a ceiling.
- **The owner does the in-game look and feel.** Screenshots are evidence for a person or an agent to interpret, never a verdict. When a milestone needs a look in the game, the session sets up the situation and tells the owner what to look for.
- **Committing and pushing are the owner's call.** Wait to be asked, unless a coordinator's prompt says otherwise. Never push unless told to.
- **Git: trunk based.** All work lands on `main`. When sessions run in parallel, each works in its own worktree, `../<repo>-<role>-<topic>` on branch `<role>/<topic>`, rebases on `main` and fast-forward merges. Never use `git stash`: the stash list is shared by every worktree.
- **Do not add a dependency for something you can write in an afternoon.** Code taken from another project needs a license that allows this project to be open-sourced later. Record where it came from and its license.

## Machines

- **The Windows test machine** is a cloud VM with Arma 3 and Steam. Every session that starts the game runs there. A game client window renders only while the owner is connected through Shadow PC. A dedicated server and headless clients need no window.
- **The Mac** runs sessions that need no game: research, spec, plan, documents.
- `origin` (`github.com/casvanluijtelaar/shakedown`) carries work between the two. Pull before you start a session on either machine.
- Steam must be running and logged in before any run. Without it the game exits at once.
