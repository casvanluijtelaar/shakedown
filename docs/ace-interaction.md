---
type: Documentation
title: ACE interaction actions
description: How a check makes a player use a mod's ACE interaction action, on an object or on itself, by its path of action names with skd_fnc_useAceAction, how to find an action's path, what the function does through ACE's own key-up code, how it fails, the ACE version it is pinned to, and why a check cannot send a real key to a game window.
status: draft
---

# ACE interaction actions

A mod that offers its actions through ACE3's interaction menu promises something about a player who opens that menu and picks the action. `skd_fnc_useAceAction` makes the player of a game client do that, as ACE itself performs the action when the player lets go of the interaction key (`ACT-04`). The contract is in [the check API](../spec/contract/check-api.md), "ACE interaction actions".

## A first check

```sqf
// @check US-089 p1 opens the crate through its ACE action
// @setups dedicated-ace
private _crate = ["skdfx_Crate", AGLToASL [130, 132, 0], 180] call skd_fnc_create;
["p1", AGLToASL [130, 130, 0], 0] call skd_fnc_place;
["p1", _crate, ["ACE_MainActions", "skdfx_aceOpen"]] call skd_fnc_useAceAction;
["all", { (_this select 0) getVariable ["skdfx_open", false] }, "==", true, "every machine sees the crate open", 10, [_crate]] call skd_fnc_expectEach;

// The player's own menu: the target is the acting machine's own player.
["p1", "p1", ["ACE_SelfActions", "skdfx_aceSignal"]] call skd_fnc_useAceAction;
```

`skd_fnc_useAceAction [machine, target, path, ceiling]`:

- `machine` is the game client whose player acts: `p1` to `pN`, or in singleplayer `p1` (`server` names it too).
- `target` is the object whose menu holds the action, or a machine name for that machine's player. When the target is the acting player itself, by its machine name or its unit, the function opens the player's own menu, ACE's self-interaction; any other target opens that object's menu, ACE's interaction.
- `path` is the action names from the menu's root, as an array of strings. The ceiling defaults to 10 s.

It returns `true` once ACE ran the action. The action's statement runs on the acting client; what the mod does elsewhere, such as asking the server to change something, the check reads with a wait or an assertion on those machines.

The setup needs ACE3 and CBA_A3 in its mods. Give the ACE mod a `patch` key so the result file records the ACE version each machine loaded:

```toml
[mods.ace]
path = 'C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\463939057'
patch = "ace_main"
version = "3.21.2.113"   # optional: refuse the start when the Workshop has updated ACE
```

## Finding an action's path

The names in a path are ACE's action names, not the text the menu shows: the first argument a mod gives `ace_interact_menu_fnc_createAction`, or the class name of an action a mod declares in config under `ACE_Actions` or `ACE_SelfActions`. Actions on an object sit under `ACE_MainActions`; a player's own actions sit under `ACE_SelfActions`. A mod that calls

```sqf
private _a = ["agh_getOnRope", "Get on rope", "", { ... }, { ... }] call ace_interact_menu_fnc_createAction;
["agh_Rope", 0, ["ACE_MainActions"], _a] call ace_interact_menu_fnc_addActionToClass;
```

has the path `["ACE_MainActions", "agh_getOnRope"]` on a rope. A wrong name tells you the right ones: the failure lists every action that exists where the name was looked for.

To list a menu by hand in a running session, read the class tree on the client:

```
shakedown exec --machine p1 --code "((ace_interact_menu_ActNamespace get 'skdfx_Crate') select 0 select 1) apply { (_x select 0) select 0 }"
```

## What it does

On the acting client, every frame up to the ceiling, the function walks the path through the target's menu as ACE builds it: the actions declared for the target's class, the actions added to that one object, and the actions an action's insert-children code makes at that moment. It calls each action's condition, and ACE's modifier code where an action has one, with `_target`, `_player` and `_actionParams` as ACE sets them. Once every name is found and every condition holds, it selects the last action as ACE's menu does when the player hovers it and calls ACE's own key-up code, `ace_interact_menu_fnc_keyUp`, which checks the condition once more and runs the statement with `this` the target. The player's real menu stays closed, and ACE raises its `ace_interactMenuClosed` event as it does for a player.

ACE's key-up code checks neither distance nor line of sight, so neither does the function: a check that promises the player can reach an action places the player and asserts the distance itself.

## When it fails

| What went wrong | Outcome | The message says |
|---|---|---|
| A name of the path does not exist | `fail` / `assertion` at the ceiling | which name, where it was looked for, and every action name there, as `"ACE_MainActions" has no action "skdfx_noSuchAction"; the actions there are: "ACE_OpenBox", "skdfx_aceOpen", ...` |
| The condition of an action on the path is false | `fail` / `assertion` at the ceiling | the path to that action, as `the condition of "ACE_SelfActions > skdfx_aceNever" was false` |
| The path ends at a folder of actions | `fail` / `assertion` at the ceiling | the folder and its actions |
| ACE is not loaded on the acting machine | `fail` / `check-error` at once | `ACE is not loaded on p1 (no ace_interact_menu), ...` |
| The loaded ACE lacks an internal the function uses | `fail` / `check-error` at once | the ACE version the machine loaded, the internal it lacks, and 3.21.2 |
| `server` or a headless client acts, the target is `objNull`, or the path is not an array of strings | `fail` / `check-error` at once | what to give instead |

`reason.machine` is the acting machine.

## The ACE version it is pinned to

ACE has no public function that performs an action by its path, so the function uses ACE's interaction internals, each marked `Public: No` (`INT-ACE-03`). It was checked against ACE3 3.21.2 (`SPK-21`, research `35-spikes-ace-and-keys.md`). An ACE that renames one of them fails the check with the version found; the project file's `version` key on the ACE mod stops a start on another ACE version before any check runs ([When a start is refused or fails](starting-and-failed-starts.md)).

## Real keys

A check cannot send a real key to a game client's window. Arma 3 reads the keyboard through DirectInput, so a key posted to its window reaches neither a key handler nor a CBA keybind, even with the game in the foreground (`SPK-11`), and anything that does reach it would type on the machine's own keyboard. `ACT-05` is dropped. A check presses a keybind with `skd_fnc_pressKey`, and reads whether the key passed on to the game by calling the keybind's code, as [the check API by example](check-api.md) shows.

## The fixture's ACE actions

`@skd_fixture_cba` registers three ACE actions where ACE is loaded, for the framework's own tests and the acceptance suite: `["ACE_MainActions", "skdfx_aceOpen"]` on the fixture's crate, and `["ACE_SelfActions", "skdfx_aceSignal"]` and `["ACE_SelfActions", "skdfx_aceNever"]`, whose condition is always false, on every player ([fixture mods](../spec/contract/fixture-mod.md)). The e2e project's checks in `e2e/tests/ace/` use them in the setups `sp-ace` and `dedicated-ace`.
