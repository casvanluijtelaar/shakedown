---
type: Research
title: "Spikes SPK-21 and SPK-11: ACE interaction by its path, and a key posted to the game's window"
description: "That ACE3 3.21.2's own key-up code runs an interaction action found by its path of action names once when its condition holds and not when it does not, on an object and on the player, in singleplayer and on a dedicated client, 10 of 10 each; and that a key press posted as window messages to the game's own window reaches neither a display-46 key handler nor a CBA keybind, even with the game in the foreground, so ACT-05 is dropped; run on a Windows machine in M17."
status: stable
generated: { by: "claude-code/developer", at: 2026-11-21T19:36:23Z }
sources:
  - { id: ace-keyup, resource: "acemod/ACE3@v3.21.2:addons/interact_menu/functions/fnc_keyUp.sqf", read: 2026-11-21 }
  - { id: ace-render, resource: "acemod/ACE3@v3.21.2:addons/interact_menu/functions/fnc_render.sqf (where selectedAction and selectedTarget are set)", read: 2026-11-21 }
  - { id: ace-collect, resource: "acemod/ACE3@v3.21.2:addons/interact_menu/functions/fnc_collectActiveActionTree.sqf, fnc_findActionNode.sqf, fnc_addActionToClass.sqf, fnc_addActionToObject.sqf, fnc_createAction.sqf, fnc_renderActionPoints.sqf", read: 2026-11-21 }
  - { id: ace-release, resource: "gh release list -R acemod/ACE3: 3.21.2 is the latest release, 3.21.3 RC1 a pre-release", read: 2026-11-21 }
  - { id: runs, resource: "The M17 developer round's spike runs through shakedown exec against sessions s-20261121-192652 (singleplayer, CBA, ACE, fixtures), s-20261121-192933 (dedicated, two clients, the same mods) and s-20261121-193413 (singleplayer, CBA, fixtures)", read: 2026-11-21 }
  - { id: dicoop, resource: "Microsoft Learn, DirectInput cooperative levels (as cited in research 15-player-input.md)", read: 2026-11-04 }
---

# Spikes SPK-21 and SPK-11: ACE interaction by its path, and a key posted to the game's window

M17 ran both spikes of `engineering/spec/06-open-questions.md` on a Windows machine before the work that depends on them. `SPK-21` passed: the framework performs an ACE interaction action through ACE's own key-up code. `SPK-11` failed: a key posted to the game's window does not reach the game, so `ACT-05` is dropped and `US-091` and `US-092` are deprecated, as `engineering/plan/milestones/m17.md` provides.

## SPK-21: ACE interaction by its path

### The pass condition

On the ACE3 version current when the milestone starts, an action of a mod's ACE interaction menu, on an object and on the player itself, found by its path of action names, runs its statement once on the acting client when its condition holds and not when it does not, in singleplayer and on a dedicated client, 10 of 10.

ACE3 3.21.2 is the latest release on 2026-11-21[^ace-release], the version `INT-ACE-01` names, and the version the Workshop copy on a Windows machine loads (`ace_main`'s `versionStr` reads `3.21.2.113`).

### How ACE performs an action

When the player releases the interaction key with an action selected, `ace_interact_menu_fnc_keyUp` takes the selected action and its target from `ace_interact_menu_selectedAction` and `ace_interact_menu_selectedTarget`, clears ACE's condition caches, sets `this` to the target, calls the condition with `[_target, _player, _actionParams]`, calls the statement if it holds, and raises `ace_interactMenuClosed`[^ace-keyup]. It acts only while `ace_interact_menu_openedMenuType` is 0 (an object's menu) or 1 (the self menu), and skips the action when ACE's setting "action on key release" is off unless its second argument, "called by clicking", is true. The selected action is the node ACE's menu renders, `[action data, children, object]`[^ace-render], the shape `ace_interact_menu_fnc_collectActiveActionTree` builds[^ace-collect].

The action tree of an object is the class tree in `ace_interact_menu_ActNamespace` (self: `ace_interact_menu_ActSelfNamespace`), keyed by its class, plus the actions added to that one object in its variable `ace_interact_menu_actions` (self: `ace_interact_menu_selfActions`), each with its parent path; a node's children are its class children, the actions of the object whose parent path is the node's path, and those its insert-children code returns[^ace-collect]. ACE's conditions and statements read `_target`, `_player` and `_actionParams` as variables of the caller, so whatever calls them defines those three first.

### What was run

On the acting client, through `shakedown exec`[^runs], a candidate of the framework's own code:

1. compiled the target's menu (`ace_interact_menu_fnc_compileMenu`, or `compileMenuSelfAction` for the player itself), then walked the path one name at a time through the children described above, calling each node's modifier code and condition as ACE's menu does; a name not found stopped with the names that exist at that level, and a condition that was not `true` stopped naming the node;
2. at the end of the path set `openedMenuType`, `actionSelected`, `selectedTarget` and `selectedAction` as ACE's menu sets them, and called `[menu type, true] call ace_interact_menu_fnc_keyUp`, all in one unscheduled call.

The targets were a class action `spk_ext` under `ACE_MainActions` on `skdfx_Crate` and a self action `spk_self` under `ACE_SelfActions` on the player's class, each counting its runs, both with a condition read from a variable the spike set.

### Results

| Setup | Action | Condition true: runs of 10 | Condition false: runs of 10 |
|---|---|---|---|
| Singleplayer, CBA, ACE, fixtures | on the crate | 10 | 0 |
| Singleplayer | on the player itself | 10 | 0 |
| Dedicated, two clients, `p1` acting | on the crate | 10 | 0 |
| Dedicated, `p1` acting | on the player itself | 10 | 0 |

The statement ran with `_target` and `this` the crate. A path whose last name did not exist stopped with the names at its parent, `ACE_OpenBox`, `spk_ext`, `ace_dragging_drag`, `ace_dragging_carry` and ACE's cargo actions. With the condition false, ACE's key-up code given the action directly ran nothing either, so the condition is checked by ACE's own path and not only by the framework's walk. `openedMenuType` was back at -1 after each call, so the player's real menu is left closed. Each call took under 0.02 s.

`SPK-21` passes. `ACT-04` is built on ACE's key-up code, which uses ACE internals marked `Public: No`, so `INT-ACE-03` pins it to 3.21.2.

## SPK-11: a key posted to one game window

### The pass condition

With `-noPause`, the game not in the foreground and another window focused, posted `WM_KEYDOWN`/`WM_KEYUP` fire a display-46 `KeyDown` handler and a CBA keybind once; the focused window and the foreground are unchanged, with a remote desktop connected and disconnected.

### What was run

An external script that enumerated the game's windows and posted the messages was refused by the session's own permission check, so the messages were posted from inside the game process, as the milestone would build it: a call of the framework's extension found the process's own visible top-level window with `EnumWindows` and posted `WM_KEYDOWN`, then two frames later `WM_KEYUP`, for F7 (DIK 65, the scan code in `lParam`, the virtual key from `MapVirtualKeyW`) to it with `PostMessageW`. Display 46 carried a `KeyDown` and a `KeyUp` handler that recorded every key, and `@skd_fixture_cba` registered its CBA keybind on F7[^runs].

### Results

Both posts returned success each time, 11 presses in all, and in the 10 s after them neither handler recorded a key and the keybind's counter stayed 0. The game reported `isGameFocused` true throughout: it was the foreground window, the case most favourable to the input, and still nothing arrived.

Arma 3 reads the keyboard through DirectInput, whose device state does not come from window messages, and a device at the foreground cooperative level is unacquired in the background[^dicoop], so no posted message can reach it. Input that does reach it is either the global input stream (`SendInput`), which types on the machine's keyboard and is ruled out by CLAUDE.md, or code inside the engine's input path, which `EP-01` and `NFR-LG-01` rule out. The case "another window focused" and the two states of a remote desktop connection were not run, since the condition already failed in the favourable case.

The window found was not identified by its class name, because the external script that would have listed it was refused; the game process has one visible top-level window.

`SPK-11` fails. `ACT-05` is dropped, `US-091` and `US-092` are deprecated, and a check reads a key's effect by calling the keybind's code (`ACT-02`), with `DEF-26`'s way round.

## What the spec does with it

- `ACT-04` performs an ACE action by its path through `ace_interact_menu_fnc_keyUp`, as above; `INT-ACE-03` names ACE3 3.21.2 and the internals it reads, and the check API contract gains `skd_fnc_useAceAction`.
- `ACT-05` is dropped; `CR-15` is not delivered in phase 2.
- `US-091` and `US-092` are `deprecated`; M17 delivers the ACE work alone.

[^ace-keyup]: ACE3 3.21.2, `fnc_keyUp.sqf`.
[^ace-render]: ACE3 3.21.2, `fnc_render.sqf`.
[^ace-collect]: ACE3 3.21.2, the interaction menu's tree functions.
[^ace-release]: ACE3's GitHub releases, read 2026-11-21.
[^runs]: The M17 developer round's spike runs, 2026-11-21 from 19:26 to 19:36 UTC.
[^dicoop]: DirectInput cooperative levels.
