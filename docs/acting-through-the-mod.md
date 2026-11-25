---
type: Documentation
title: Acting through the mod
description: How a check makes a player use a mod's scroll-menu action by its title with skd_fnc_useAction and press a CBA keybind by its names with skd_fnc_pressKey, what each checks and does, how to read whether a key passed on to the game, how each fails, and why a check cannot send a real key to the game.
status: draft
---

# Acting through the mod

A check proves what a player gets, so it acts the way a player does: through the mod's own scroll-menu actions, CBA keybinds and ACE interaction menu, on the machine of the player who acts. Nothing touches the machine's own mouse or keyboard. This page covers actions and keybinds; [ACE interaction actions](ace-interaction.md) covers ACE's menu. The reference is [The check API](check-api.md#acting-through-the-mod).

## Scroll-menu actions: `skd_fnc_useAction`

```sqf
// @check US-019 p1 opens the crate and gets a first aid kit
private _p1 = ["p1"] call skd_fnc_player;
private _crate = ["mymod_Crate", AGLToASL [100, 102, 0], 0] call skd_fnc_create;
["p1", AGLToASL [100, 100, 0], 0] call skd_fnc_place;
private _kits = { { _x == "FirstAidKit" } count items (_this select 0) };
private _before = ["p1", _kits, [_p1]] call skd_fnc_on;
["p1", _crate, "Open crate"] call skd_fnc_useAction;
["p1", _kits, "==", _before + 1, "p1's first aid kits", 10, [_p1]] call skd_fnc_expectOn;
```

`[machine, target, title, ceiling] call skd_fnc_useAction`:

- `machine` is the game client whose player acts: `p1` to `pN`, or `p1` in singleplayer (`server` names it too).
- `target` is the object carrying the action: an object the check made, the player's own unit (`["p1"] call skd_fnc_player`) for an action on the player, or any other object.
- `title` is matched after removing structured-text tags such as `<t color='#ff0000'>` from the action's title, exactly and case-sensitively.
- `ceiling` defaults to 10 s.

On the acting machine it waits, every frame up to the ceiling, until the action exists on the target, its condition holds and the player is within its radius. Then it fires it there with `action ["User", target, id]`, as the player's own choice from the scroll menu, and returns `true`. The condition is evaluated with `_target` the object carrying the action and `_this` the player; a radius of -1 is no limit. The engine itself runs an action fired that way whatever its condition and radius, so the framework checks them first.

The target travels by net id and is looked up every frame, so an object the server has just created is found once it reaches the acting machine.

### When it fails

| What went wrong | Outcome | The message says |
|---|---|---|
| No action with that title on the target | `fail` / `assertion` at the ceiling | the titles there are |
| The action's condition is false | `fail` / `assertion` at the ceiling | that the condition is false |
| The player is out of the action's reach | `fail` / `assertion` at the ceiling | the player's distance and the action's radius |
| The machine has no player (`server` or a headless client on a dedicated session) | `fail` / `check-error` at once | that the machine has no player |
| `target` is `objNull` | `fail` / `check-error` at once | on a dedicated session, that `player` in the body is `objNull` and to use `skd_fnc_player` |

`reason.machine` is the acting machine. The commonest mistake on a dedicated session is `["p1", player, "..."]`: in the body `player` is the server's, `objNull`. Write `["p1", ["p1"] call skd_fnc_player, "..."]`.

An action shows only while the player is within its radius, measured from the player to the object's centre: place the player well inside it, not at its edge.

## CBA keybinds: `skd_fnc_pressKey`

```sqf
["p1", "mymod", "stowKit"] call skd_fnc_pressKey;                       // down, then up
["p1", "mymod", "signal", "down"] call skd_fnc_pressKey;                 // only the key-down code
["p1", "mymod", "signal", "up"] call skd_fnc_pressKey;                   // only the key-up code
```

`[machine, mod, action, mode] call skd_fnc_pressKey` presses a keybind a mod registered with CBA, by the mod and action names it registered it with (the first two arguments of its `CBA_fnc_addKeybind`). On the named machine it reads the keybind with `CBA_fnc_getKeybind` and calls its own key-down code, key-up code or both, as the game's key handler would, with `[findDisplay 46, DIK, shift, ctrl, alt]` for its first bound key. `mode` is `"press"` (down then up, the default), `"down"` or `"up"`. It returns `true` once the keybind's code ran.

It fails `fail` / `check-error`:

- when CBA is not loaded on that machine: `skd_fnc_pressKey: CBA is not loaded on p1, ...`;
- when the keybind is not registered, listing the keybinds registered for that mod, in lower case as CBA keeps them, or, when the mod has none, the mods that have keybinds.

A check that presses no key needs no CBA.

### Did the key pass on to the game?

`skd_fnc_pressKey` returns `true` once the keybind's code ran, not what that code returned, so it does not say whether the key was consumed or passed on to the game. A check that asserts that calls the keybind's code itself on the player's machine, as `skd_fnc_pressKey` does, and reads its value:

```sqf
private _passed = ["p1", {
    private _bind = ["mymod", "stowKit"] call CBA_fnc_getKeybind;
    private _key = (_bind select 8) param [0, _bind select 5];
    _key params ["_dik", "_mods"];
    _mods params ["_shift", "_ctrl", "_alt"];
    private _out = nil;
    isNil { _out = [findDisplay 46, _dik, _shift, _ctrl, _alt] call (_bind select 3) };
    _out
}] call skd_fnc_on;
[_passed, "==", false, "away from a crate the key passes on to the game"] call skd_fnc_expect;
```

## Reading what the action did

The action's code runs on the acting machine. What the mod does elsewhere, such as asking the server to change something, the check reads with a wait or an assertion on those machines:

```sqf
["p1", _crate, "Open crate"] call skd_fnc_useAction;
["all", { (_this select 0) getVariable ["mymod_open", false] }, "==", true, "every machine sees the crate open", 10, [_crate]] call skd_fnc_expectEach;
```

To know an event the mod raises came, and with which arguments, record it ([Recording the mod's events](recording-events.md)).

## Real keys

A check cannot send a real key to a game client's window. Arma 3 reads the keyboard through DirectInput, so a key posted to its window reaches neither a key handler nor a CBA keybind, even with the game in the foreground, and anything that did reach it would type on the machine's own keyboard. A check presses a keybind with `skd_fnc_pressKey` and reads whether the key passed on by calling the keybind's code, as above.
