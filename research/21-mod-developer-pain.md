---
type: Research
title: What mod developers say about testing
description: A short first pass on what Arma 3 mod and mission developers report about testing their work, the workarounds they use and the failures that keep coming up, from GitHub issues, the Bohemia forums and Reddit.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T21:55:00+01:00 }
sources:
  - { id: ace-8199, resource: "https://github.com/acemod/ACE3/issues/8199", read: 2026-11-04 }
  - { id: ace-5329, resource: "https://github.com/acemod/ACE3/issues/5329", read: 2026-11-04 }
  - { id: ace-5406, resource: "https://github.com/acemod/ACE3/issues/5406", read: 2026-11-04 }
  - { id: ace-template, resource: "https://github.com/acemod/ACE3/blob/master/.github/ISSUE_TEMPLATE/bug_report.md at commit 45c25247f7599687e85c5981f150efe37227393b", read: 2026-11-04 }
  - { id: ace-testmissions, resource: "https://github.com/acemod/ACE3/blob/master/docs/wiki/feature/testmissions.md at commit 45c25247f7599687e85c5981f150efe37227393b", read: 2026-11-04 }
  - { id: cba-1336, resource: "https://github.com/CBATeam/CBA_A3/issues/1336", read: 2026-11-04 }
  - { id: cba-tests, resource: "https://github.com/CBATeam/CBA_A3 at commit 8b15a04bd52ff7acc568a93cf6e90343e43a8a56, addons/arrays/test.sqf and addons/main/script_macros_common.hpp", read: 2026-11-04 }
  - { id: bif-sqfvm, resource: "https://forums.bohemia.net/forums/topic/210118-sqf-vm-an-sqf-emulator/", read: 2026-11-04 }
  - { id: rd-g83sxn, resource: "https://www.reddit.com/r/armadev/comments/g83sxn/simulate_multiplayer_traffic_for_mission_testing/ (search excerpt only; Reddit refused the fetch)", read: 2026-11-04 }
  - { id: rd-1i4683f, resource: "https://www.reddit.com/r/armadev/comments/1i4683f/a3ds_ai_created_script_working_in_local_testing/ (search excerpt only; Reddit refused the fetch)", read: 2026-11-04 }
  - { id: rd-48fxub, resource: "https://www.reddit.com/r/armadev/comments/48fxub/how_to_create_working_coop_missions_locality/ (search excerpt only; Reddit refused the fetch)", read: 2026-11-04 }
  - { id: rd-1o0t782, resource: "https://www.reddit.com/r/armadev/comments/1o0t782/how_to_debug_script_for_dedicated_server/ (search excerpt only; Reddit refused the fetch)", read: 2026-11-04 }
  - { id: rd-4b85bg, resource: "https://www.reddit.com/r/armadev/comments/4b85bg/anyway_to_fake_a_player_to_test_mp_mission_alone/ (search excerpt only; Reddit refused the fetch)", read: 2026-11-04 }
  - { id: steam-2inst, resource: "https://steamcommunity.com/app/107410/discussions/0/358416640402930005/ (search excerpt only)", read: 2026-11-04 }
---

# What mod developers say about testing

This is the short first pass the plan asks for (T12 is a **later** topic). It reads GitHub issues of ACE3 and CBA_A3, the Bohemia forums and Reddit's r/armadev. Discord is left to the owner, per the plan's decisions.

Two limits on what was read. Reddit refused every fetch, and the Bohemia forums and the BI wiki answered with HTTP 403. The Reddit quotes below come from the excerpts a search engine shows for each thread, not from the thread itself, so each is **unverified** until someone opens the thread. The one forum thread quoted was read through a second fetch tool that got through.

## Works in singleplayer, breaks on a dedicated server

The pain that comes up most is behaviour that differs between singleplayer and a dedicated server, found by players after a release.

- ACE3 #5329: rearming shows its progress bar but "the vehicles ammo is not actually replenished", reported as "Dedicated only (works fine in SP/listen server)"[^ace-5329].
- ACE3 #5406: "It seems like that rearming in multiplayer seems broken in all three rearm settings. It works fine in singleplayer"[^ace-5406].
- ACE3 #8199: crates airdropped in singleplayer are ejected as scripted, while "in MP airdropping cargo results in crate spawning at correct position but with no velocity". The reporter could not reproduce it "on empty mission but the issue reliably happens on our server where a lot of objects are placed". A maintainer traced it to locality: "setVelocity is AL, so if the paradropped box is not local, it won't do anything", and "Locality can change during the lifetime of an object"[^ace-8199].
- ACE3's bug template asks every reporter "Where did the issue occur?" with the choices "Dedicated / Self-Hosted Multiplayer / Singleplayer / Editor (Singleplayer) / Editor (Multiplayer) / Virtual Arsenal", and asks for "the client and/or server RPT file"[^ace-template].
- On Reddit, a mission script "works fine when i test it locally, but on dedicated the radio is spawned in, but there is no addaction to it"[^rd-1i4683f] (**unverified**).

**Forces:** the same check runs in singleplayer and on a dedicated server with clients, and asserts on every machine, since a pass on one setup says nothing about the other. Locality is the usual cause, so a failure report shows which machine owned the object at the failing step. Load matters too (#8199 reproduced only on a full server), so a check can set up a heavier scene. Touches `CR-41`, `CR-22`, `CR-10`, `CR-11`, `CR-31`.

## Testing multiplayer needs other people or a hand-built setup

- "Tl;Dr: You can try your best to avoid problems, but you'll need to test with actual players. Edit: Always keep locality in mind!"[^rd-g83sxn] (**unverified**).
- "Anyway to 'fake' a player to test MP mission alone? ... there is some stuff I need to test out with more than juste 1 player connected"[^rd-4b85bg] (**unverified**).
- "i setup myself a dedicated lan server with TADST to test all my missions. this testing inside the editor only ..."[^rd-48fxub] (**unverified**).
- "Setup a local server on your machine and load debug addons for easier testing"[^rd-1o0t782] (**unverified**).
- Running two copies on one PC: "just adjust your server.cfg. You have to add loopback = true; and disable duplicate kick (kickduplicate=0;)"[^steam-2inst] (**unverified**).

**Forces:** the framework's multiplayer session (a local dedicated server, clients and a headless client on one machine, started by one command) is exactly the workaround people build by hand, and it should need no hand-edited `server.cfg`. Each client stands in for a player the developer would otherwise need. T5 covers the start options. Touches `CR-01`, `CR-04`, G4.

## Every change is a restart, and reload is fragile

- CBA #1336: "sometimes RECOMPILE is needed to edit functions while game is running. But when RECOMPILE is defined ... on game start the functions are not compiled to missionNamespace and can't be used in console by name"[^cba-1336]. A reload setting meant to speed the loop broke other things.
- The PRD's own measurement (12 starts of 3.5 minutes to chase one defect) is the same pain on the first user's mod.

**Forces:** reload into a running session is the speed lever (`CR-03`), and its side effects are a risk to report: after a reload the framework checks that the reloaded functions are where a check expects them, in every namespace and on every machine, and says so when they are not. T4 covers what can be reloaded. Touches `CR-03`, G2.

## Tests exist only for functions, run by hand

- CBA keeps SQF tests in its addons, run with `execVM` and `waitUntil { scriptDone _test }`, that log "Test OK" or raise "Test FAIL"[^cba-tests]. They test functions; nothing in them starts a session, drives a player or produces a result file.
- ACE3 had a "Test Missions" component; its page marks it `removed: true` as of 3.6.0[^ace-testmissions].
- On the Bohemia forums, SQF-VM, an SQF emulator, is offered for testing outside the game: "More advanced users might even use this for unit-testing their code", and one user writes "I've been using for testing SQF when not on a PC with arma, someone else suggested it could be used for unit testing"[^bif-sqfvm].

**Forces:** nobody in these sources has an in-game acceptance runner, which is the gap the framework fills. Unit tests of SQF outside the game stay out of scope (PRD section 4); the framework should not compete with SQF-VM or CBA's tests but can run CBA-style test files as checks. Touches `CR-50`, `CR-40`.

## Evidence is a log file pasted by hand

ACE3 closes issues that come without logs, and asks for client and server RPT files linked from a gist or pastebin[^ace-template]. In #5406 the reporter wrote "can't acces rpt file"[^ace-5406].

**Forces:** a result file that carries each machine's log for the run, cut to the failing check, is what a bug report needs. A failure on one client names that client. Touches `CR-24`, `CR-23`, `CR-51`.

## Recurring failure modes, in short

1. Locality: code that works where the object is local and does nothing elsewhere[^ace-8199][^ace-5329].
2. Singleplayer and editor tests that pass while the dedicated server fails[^ace-5406][^rd-1i4683f].
3. Bugs that show only under load or with many objects[^ace-8199].
4. Multiplayer tests that need other people or a hand-built local server[^rd-g83sxn][^rd-48fxub].
5. A slow edit loop and fragile reload settings[^cba-1336].
6. Logs gathered by hand from each machine[^ace-template].

Next pass, when T12 comes up again: open the Reddit threads by hand to confirm the quotes, read the Bohemia forums' "Mission Testing Workflow" thread (found by search, not opened), and add the owner's Discord searches from the plan.

[^ace-8199]: ACE3 issue #8199, "ParadropItem inconsistency between SP and MP", 2021-04-06, and its comments of 2021-04-07.
[^ace-5329]: ACE3 issue #5329, "Vehicle Rearming interaction no longer rearms vehicle on dedicated", 2017-07-05.
[^ace-5406]: ACE3 issue #5406, "Rearming in Multiplayer seems broken", 2017-08-05.
[^ace-template]: ACE3 bug report template at the named commit.
[^ace-testmissions]: ACE3 wiki page "Test Missions" at the named commit.
[^cba-1336]: CBA_A3 issue #1336, "Functions are inaccessible in Eden if RECOMPILE is defined on game start", 2020-05-14.
[^cba-tests]: CBA_A3 at the named commit: arrays test runner and the `TEST_*` macros.
[^bif-sqfvm]: Bohemia forums, "SQF-VM - An SQF Emulator" (page last updated 2020-08-29).
[^rd-g83sxn]: r/armadev, "Simulate Multiplayer Traffic for Mission Testing?", search excerpt.
[^rd-1i4683f]: r/armadev, "[A3][DS] AI created script working in local testing SP and MP, but not on ...", search excerpt.
[^rd-48fxub]: r/armadev, "How to create working COOP missions? (locality)", search excerpt.
[^rd-1o0t782]: r/armadev, "How to Debug Script for Dedicated Server", search excerpt.
[^rd-4b85bg]: r/armadev, "Anyway to 'fake' a player to test MP mission alone?", search excerpt.
[^steam-2inst]: Steam community, "(SOLVED) is it possible to have 2 instances of arma 3 running?", search excerpt.
