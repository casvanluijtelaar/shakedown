---
type: User Story
title: "One command runs a selection in every setup it needs"
description: "With no session running, one command runs the selected checks in each setup their headers name, one session per setup in turn, and leaves a result file per setup and nothing running."
status: draft
id: US-074
role: test-author
kind: feature
requirements: [RUN-07, RUN-04, STP-02]
---

# US-074 One command runs a selection in every setup it needs

As a test author, I want one command for my whole suite across singleplayer and dedicated setups, so that I can walk away from it instead of running and waiting for each setup in turn.

## Acceptance criteria

1. In a project with the setups `sp`, `sp-ace` and `dedicated-2`, whose checks name one, two or all three in `@setups`, `shakedown run --each-setup` starts a session of each setup in the project file's order, one at a time, runs the checks that name it, and stops it before the next starts.
2. It leaves one run folder and `result.json` per setup, prints one line per check and one per setup, and exits with the highest exit code of the three runs.
3. After it, no `arma3_x64` or `arma3server_x64` process runs and `status` shows no session.
4. With a selection, such as `--tag m00`, it starts only the setups the selected checks name.
5. Through MCP, `run_start` with each setup returns a handle at once, and `run_status` names the setup it is in.

## Next to

- [US-075](US-075-setup-session-fails-in-each-setup-run.md) (system fail): one setup's session fails to start.
- [US-079](US-079-unknown-setup-in-check-header.md) (mistake): a check names a setup the project does not declare.
