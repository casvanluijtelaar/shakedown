# Arma automation framework

An automation framework for Arma, built with agentic engineering: the specifications, user stories and contract in this repository are the source, and language models write the code, the tests and the documentation against them. The method follows the Eidra agentic engineering guide.

## Where things are

| Folder | What it holds |
|---|---|
| `research/` | What is known about the domain, each claim traced to a source. |
| `spec/` | The client requirements, the specification, the roles and the user stories. |
| `plan/` | The implementation plan, one concept per milestone, and deferred work. |
| `docs/` | How the built system works, and a report per finished milestone. |
| `acceptance/` | The acceptance suite, its findings and its coverage, owned by the test developer. |
| `coordinator/` | The playbook a coordinator session loads to drive the loop. |
| `tools/` | The OKF bundle check. |

Every markdown document in the bundles is an Open Knowledge Format (OKF v0.2) concept, so `python3 tools/okf_check.py` can read the state of the project from the files. [CLAUDE.md](CLAUDE.md) is the map for agent sessions and holds the working rules.

## How work runs

1. **Research** the domain into `research/`.
2. **Specify** the system and its user stories in `spec/`.
3. **Plan** the milestones in `plan/`.
4. **Implement** and 5. **test**, as a loop per milestone: a framework developer builds, a test developer checks the result against the stories without seeing the code and writes findings, the developer fixes, a fresh tester verifies.

The project is MVP first and local first: each milestone takes the shortest path to its demo, and everything that starts the game runs on the Windows test machine. GitHub Actions runs only the gates that need no game. `CLAUDE.md` has the details.

A coordinator session can drive the loop. Start it with:

> You are the coordinator for this repository. Read CLAUDE.md and coordinator/playbook.md in full, and follow the playbook.

## Checks

- `python3 tools/okf_check.py` - the bundle check. Needs Python 3.9+ and PyYAML. CI runs it on every push.
