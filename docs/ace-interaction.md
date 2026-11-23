---
type: Documentation
title: ACE interaction actions
description: How a check makes a player use a mod's ACE interaction action, on an object or on itself, by its path of action names with skd_fnc_useAceAction, how to find an action's path, what the function does through ACE's own key-up code, how it fails, and the ACE version it is pinned to.
status: draft
---

# ACE interaction actions

A mod that offers its actions through ACE3's interaction menu promises something about a player who opens that menu and picks the action. `skd_fnc_useAceAction` makes the player of a game client do that, as ACE itself performs the action when the player lets go of the interaction key. The reference is [The check API](check-api.md#acting-through-the-mod).

## A first check

```sqf
// @check US-089 p1 opens the crate through its ACE action
// @setups dedicated-ace
private _crate = ["mymod_Crate", AGLToASL [130, 132, 0], 180] call skd_fnc_create;
["p1", AGLToASL [130, 130, 0], 0] call skd_fnc_place;
["p1", _crate, ["ACE_MainActions", "mymod_open"]] call skd_fnc_useAceAction;
["all", { (_this select 0) getVariable ["mymod_open", false] }, "==", true, "every machine sees the crate open", 10, [_crate]] call skd_fnc_expectEach;

// The player's own menu: the target is the acting machine's own player.
["p1", "p1", ["ACE_SelfActions", "mymod_signal"]] call skd_fnc_useAceAction;
```

`[machine, target, path, ceiling] call skd_fnc_useAceAction`:

- `machine` is the game client whose player acts: `p1` to `pN`, or in singleplayer `p1` (`server` names it too).
- `target` is the object whose menu holds the action, or a machine name for that machine's player. When the target is the acting player itself, by its machine name or its unit, the function opens the player's own menu, ACE's self-interaction; any other target opens that object's menu, ACE's interaction.
- `path` is the action names from the menu's root, as an array of one or more strings.
- `ceiling` defaults to 10 s.

It returns `true` once ACE ran the action. The action's statement runs on the acting client; what the mod does elsewhere, such as asking the server to change something, the check reads with a wait or an assertion on those machines.

The setup needs ACE3 and CBA_A3 in its mods. Give the ACE mod a `patch` key so the result file records the ACE version each machine loaded:

```toml
[mods.ace]
path = 'C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\463939057'
patch = "ace_main"
version = "3.21.2.113"   # optional: refuse the start when the Workshop has updated ACE

[setups.dedicated-ace]
mode = "dedicated"
clients = 2
mods = ["cba", "ace", "mymod"]
```

## Finding an action's path

The names in a path are ACE's action names, not the text the menu shows: the first argument a mod gives `ace_interact_menu_fnc_createAction`, or the class name of an action a mod declares in config under `ACE_Actions` or `ACE_SelfActions`. Actions on an object sit under `ACE_MainActions`; a player's own actions sit under `ACE_SelfActions`. A mod that calls

```sqf
private _a = ["agh_getOnRope", "Get on rope", "", { ... }, { ... }] call ace_interact_menu_fnc_createAction;
["agh_Rope", 0, ["ACE_MainActions"], _a] call ace_interact_menu_fnc_addActionToClass;
```

has the path `["ACE_MainActions", "agh_getOnRope"]` on a rope. An action nested in a folder of actions has the folder's name in between, as `["ACE_SelfActions", "agh_menu", "agh_cutRope"]`. The match is exact and case-sensitive.

A wrong name tells you the right ones: the failure lists every action that exists where the name was looked for. To list a menu by hand in a running session, read the class tree on the client:

```
shakedown exec --machine p1 --code "((ace_interact_menu_ActNamespace get 'mymod_Crate') select 0 select 1) apply { (_x select 0) select 0 }"
```

## What it does

On the acting client, every frame up to the ceiling, the function walks the path through the target's menu as ACE builds it: the actions declared for the target's class, the actions added to that one object, and the actions an action's insert-children code makes at that moment. It calls each action's condition, and ACE's modifier code where an action has one, with `_target`, `_player` and `_actionParams` as ACE sets them. Once every name is found and every condition holds, it selects the last action in the same frame, as ACE's menu does when the player hovers it, and calls ACE's own key-up code, `ace_interact_menu_fnc_keyUp`, which checks the condition once more and runs the statement with `this` the target. The player's real menu stays closed, and ACE raises its `ace_interactMenuClosed` event as it does for a player.

The target travels by net id and is looked up every frame, so an object the server has just created is found once it reaches the acting machine.

ACE's key-up code checks neither distance nor line of sight, so neither does the function: a check that promises the player can reach an action places the player and asserts the distance itself:

```sqf
["p1", { player distance (_this select 0) }, "<", 2.5, "p1 is within reach of the crate", 5, [_crate]] call skd_fnc_expectOn;
```

## When it fails

| What went wrong | Outcome | The message says |
|---|---|---|
| A name of the path does not exist | `fail` / `assertion` at the ceiling | which name, where it was looked for, and every action name there, as `"ACE_MainActions" has no action "mymod_noSuchAction"; the actions there are: "ACE_OpenBox", "mymod_open", ...` |
| The condition of an action on the path is false | `fail` / `assertion` at the ceiling | the path to that action, as `the condition of "ACE_SelfActions > mymod_never" was false` |
| The path ends at a folder of actions with no statement of its own | `fail` / `assertion` at the ceiling | the folder and its actions |
| ACE is not loaded on the acting machine | `fail` / `check-error` at once | `ACE is not loaded on p1 (no ace_interact_menu), ...` |
| The loaded ACE lacks an internal the function uses | `fail` / `check-error` at once | the ACE version the machine loaded, the internal it lacks, and 3.21.2 |
| `server` or a headless client acts, the target is `objNull`, or the path is not an array of strings | `fail` / `check-error` at once | what to give instead; for `objNull` on a dedicated session, that `player` in the body is `objNull` and to use `skd_fnc_player` |

`reason.machine` is the acting machine.

## The ACE version it is pinned to

ACE has no public function that performs an action by its path, so the function uses ACE's interaction internals, each marked `Public: No`. It is checked against ACE3 3.21.2. An ACE that renames one of them fails the check with the version found. The project file's `version` key on the ACE mod stops a start on another ACE version before any check runs ([A mod in another version](starting-and-failed-starts.md#a-mod-in-another-version)).

## Trying it with the fixture mod

An install from source comes with the fixture mod `@skd_fixture_cba`, which registers three ACE actions where ACE is loaded: `["ACE_MainActions", "skdfx_aceOpen"]` on the fixture's crate `skdfx_Crate`, and `["ACE_SelfActions", "skdfx_aceSignal"]` and `["ACE_SelfActions", "skdfx_aceNever"]`, whose condition is always false, on every player ([engineering/spec/contract/fixture-mod.md](../engineering/spec/contract/fixture-mod.md)).
