---
type: Finding
id: m10-1
milestone: M10
title: "A relative `mods.<name>.path` with `..` makes the game exit at start"
description: "With the project in acceptance\\shakedown\\ and mods.agh.path = '..\\..\\.hemttout\\build', session start ended exit 4 because the game refused the joined path with .. in -mod, though the docs say a relative path is relative to the project folder."
status: draft
raised_by: test
for: developer
checks:
  - 'US-001 US-006 CFG-01 m10-1 a project in acceptance\shakedown whose mod path is ..\..\build\@skd_fixture starts a dedicated session with two clients through run --setup, the mod loads on every machine, its version is recorded, and no game command line holds ..'
  - 'US-001 US-006 CFG-01 m10-1 the same project starts a singleplayer session through run --setup, the mod loads on p1 and no game command line holds ..'
stories: [US-001, US-006]
generated: { by: "claude-code/tester", at: 2026-11-13T07:10:46-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-13T08:14:00-08:00, note: "Fixed: every path of shakedown.toml is resolved to one without . or .. before it reaches the game, so a relative mods.<name>.path may climb out of the project folder (spec/contract/cli.md, docs/project-file.md, docs/porting-a-suite.md). The test cfg_01_m10_1_a_relative_path_that_climbs_out_of_the_project_folder_holds_no_dot_dot and the game tests cfg_01_rst_01_dm_14_m10_findings_in_singleplayer and _on_a_dedicated_server, whose fixture path is ..\\..\\shakedown\\fixtures\\@skd_fixture, prove it." }
verified:
  - { by: "claude-code/tester", at: 2026-11-13T09:20:00-08:00 }
---

# A relative `mods.<name>.path` with `..` makes the game exit at start

Raised by arma-grappling-hook's test developer while porting that mod's M00 checks to Shakedown (`plan/milestones/m10.md`), against the install `shakedown 0.1.0` built 2026-11-13 06:18, and copied here word for word. The checks that hold it are the ported checks named in it, in arma-grappling-hook's `acceptance/shakedown/` on branch `shakedown/m10`; no acceptance check of this repository holds it yet.

## The finding as raised

1. **A relative `mods.<name>.path` with `..` makes the game exit at start.**
   `docs/project-file.md`, "Keys": "A relative path is relative to the project folder." `spec/contract/cli.md`: "Relative paths in the file are relative to the project folder." `docs/porting-a-suite.md`, step 2: "Point `mods.<name>.path` at the build the old suite loaded, such as HEMTT's `.hemttout\build`" and "Write `shakedown.toml` in the mod's repository, in its own folder or at the root."
   With the project in `acceptance\shakedown\` and `path = '..\..\.hemttout\build'`, `shakedown session start --setup sp-cba` ended exit 4: `p1 exited with code 1 before it was ready`, the RPT reading `ErrorMessage: Cannot open file '...\acceptance\shakedown\..\..\.hemttout\build\addons\agh_climb'` and an `.mdmp` in the profile folder. The path is joined but not normalised, and the game refuses `..` in `-mod`. A project "in its own folder" can therefore not reach the mod's build by a relative path; the port uses an absolute path, which ties the committed `shakedown.toml` to one checkout. Either normalise the path before it goes to `-mod`, or say in the docs that a path must not climb out of the project folder.
