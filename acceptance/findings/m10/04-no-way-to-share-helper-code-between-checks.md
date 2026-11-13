---
type: Finding
id: m10-4
milestone: M10
title: "No way to share helper code between checks, and the porting page does not say what to do with an old suite's shared library"
description: "With no #include, no preprocessor and no file a check can load, the port repeats a 200-line helper block in eight check files, and the porting page does not say what to do with an old suite's shared library."
status: draft
raised_by: test
for: developer
checks: []
stories: [US-011]
generated: { by: "claude-code/tester", at: 2026-11-13T07:10:46-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-13T08:14:00-08:00, note: "Answered in the docs and deferred as DEF-27: docs/porting-a-suite.md says plainly that each check carries its own copy of the helpers it uses, as local code values kept identical across files; a library folder compiled once per session on every machine is DEF-27, after the MVP. No framework change." }
verified: []
---

# No way to share helper code between checks, and the porting page does not say what to do with an old suite's shared library

Raised by arma-grappling-hook's test developer while porting that mod's M00 checks to Shakedown (`plan/milestones/m10.md`), against the install `shakedown 0.1.0` built 2026-11-13 06:18, and copied here word for word. The checks that hold it are the ported checks named in it, in arma-grappling-hook's `acceptance/shakedown/` on branch `shakedown/m10`; no acceptance check of this repository holds it yet.

## The finding as raised

4. **No way to share helper code between checks, and the porting page does not say what to do with an old suite's shared library.**
   `docs/porting-a-suite.md`, "Points to watch": "No `#include` and no macros in a check." and "What stays is the SQF that sets a scene, makes the player act and reads the outcome; it moves into one check file per check". The old suite's checks share about 400 lines (`acc_lib.sqf`, `acc_base.inc.sqf`, `acc_rope.inc.sqf`). With no `#include`, no preprocessor and no file a check can load, the port repeats a 200-line "rope kit" verbatim in eight check files, and two of them a further 70-line block; a change to it is eight edits that nothing keeps in step. The page should say how a suite shares code (for example a non-check SQF file in the project that the session compiles once, or a supported "library" folder), or say plainly that each check must carry its own copy.
