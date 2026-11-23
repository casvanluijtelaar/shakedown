---
type: Research
title: "Making a player act: actions, key handlers and key input"
description: How a check moves, turns and aims a player, uses a mod's addAction, ACE interaction and CBA keybind paths from code, whether key input can reach one game window without taking the desktop, and how a scene is built and removed.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T22:20:00+01:00 }
sources:
  - { id: setposasl, resource: "https://community.bistudio.com/wiki/setPosASL (oldid 377438)", read: 2026-11-04 }
  - { id: setdir, resource: "https://community.bistudio.com/wiki/setDir", read: 2026-11-04 }
  - { id: playmove, resource: "https://community.bistudio.com/wiki/playMove", read: 2026-11-04 }
  - { id: switchmove, resource: "https://community.bistudio.com/wiki/switchMove", read: 2026-11-04 }
  - { id: domove, resource: "https://community.bistudio.com/wiki/doMove", read: 2026-11-04 }
  - { id: setvelocity, resource: "https://community.bistudio.com/wiki/setVelocity", read: 2026-11-04 }
  - { id: lookat, resource: "https://community.bistudio.com/wiki/lookAt (Wayback snapshot 2025-01-13)", read: 2026-11-04 }
  - { id: dowatch, resource: "https://community.bistudio.com/wiki/doWatch", read: 2026-11-04 }
  - { id: forcefire, resource: "https://community.bistudio.com/wiki/forceWeaponFire (Wayback snapshot 2025-04-18)", read: 2026-11-04 }
  - { id: addaction, resource: "https://community.bistudio.com/wiki/addAction", read: 2026-11-04 }
  - { id: action, resource: "https://community.bistudio.com/wiki/action", read: 2026-11-04 }
  - { id: actions, resource: "https://community.bistudio.com/wiki/Arma_3:_Actions, entries User, UserType, GetInDriver, UseWeapon", read: 2026-11-04 }
  - { id: actionparams, resource: "https://community.bistudio.com/wiki/actionParams", read: 2026-11-04 }
  - { id: actionids, resource: "https://community.bistudio.com/wiki/actionIDs", read: 2026-11-04 }
  - { id: inguieh, resource: "https://community.bistudio.com/wiki/inGameUISetEventHandler", read: 2026-11-04 }
  - { id: displayeh, resource: "https://community.bistudio.com/wiki/displayAddEventHandler (Wayback snapshot 2025-04-19)", read: 2026-11-04 }
  - { id: startup, resource: "https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters, entries noPause and noPauseAudio", read: 2026-11-04 }
  - { id: createvehicle, resource: "https://community.bistudio.com/wiki/createVehicle (Wayback snapshot 2025-02-26)", read: 2026-11-04 }
  - { id: createunit, resource: "https://community.bistudio.com/wiki/createUnit (Wayback snapshot 2025-03-26)", read: 2026-11-04 }
  - { id: setdate, resource: "https://community.bistudio.com/wiki/setDate (Wayback snapshot 2025-03-18)", read: 2026-11-04 }
  - { id: setovercast, resource: "https://community.bistudio.com/wiki/setOvercast (Wayback snapshot 2025-04-15)", read: 2026-11-04 }
  - { id: forceweather, resource: "https://community.bistudio.com/wiki/forceWeatherChange (Wayback snapshot 2025-02-14)", read: 2026-11-04 }
  - { id: deletevehicle, resource: "https://community.bistudio.com/wiki/deleteVehicle (Wayback snapshot 2025-05-24)", read: 2026-11-04 }
  - { id: spotrep120, resource: "https://dev.arma3.com/post/spotrep-00120 (Game Update 2.22, 2026-08-11)", read: 2026-11-04 }
  - { id: spotrep122, resource: "https://dev.arma3.com/latest/spotrep (SPOTREP #00122, 2.22 hotfix, 2026-09-30)", read: 2026-11-04 }
  - { id: cbarel, resource: "https://github.com/CBATeam/CBA_A3/releases/tag/v3.19.0 (latest release, 2026-08-11)", read: 2026-11-04 }
  - { id: cba-addkeybind, resource: "CBATeam/CBA_A3@8b15a04:addons/keybinding/fnc_addKeybind.sqf", read: 2026-11-04 }
  - { id: cba-getkeybind, resource: "CBATeam/CBA_A3@8b15a04:addons/keybinding/fnc_getKeybind.sqf", read: 2026-11-04 }
  - { id: cba-addkeyhandler, resource: "CBATeam/CBA_A3@8b15a04:addons/events/fnc_addKeyHandler.sqf", read: 2026-11-04 }
  - { id: cba-keydown, resource: "CBATeam/CBA_A3@8b15a04:addons/events/fnc_keyHandlerDown.sqf", read: 2026-11-04 }
  - { id: cba-keyup, resource: "CBATeam/CBA_A3@8b15a04:addons/events/fnc_keyHandlerUp.sqf", read: 2026-11-04 }
  - { id: cba-display, resource: "CBATeam/CBA_A3@8b15a04:addons/events/fnc_initDisplayMission.sqf", read: 2026-11-04 }
  - { id: acerel, resource: "https://github.com/acemod/ACE3/releases/tag/v3.21.2 (latest release, 2026-08-18)", read: 2026-11-04 }
  - { id: ace-framework, resource: "acemod/ACE3@45c2524:docs/wiki/framework/interactionMenu-framework.md", read: 2026-11-04 }
  - { id: ace-createaction, resource: "acemod/ACE3@45c2524:addons/interact_menu/functions/fnc_createAction.sqf", read: 2026-11-04 }
  - { id: ace-keyup, resource: "acemod/ACE3@45c2524:addons/interact_menu/functions/fnc_keyUp.sqf", read: 2026-11-04 }
  - { id: ace-findnode, resource: "acemod/ACE3@45c2524:addons/interact_menu/functions/fnc_findActionNode.sqf", read: 2026-11-04 }
  - { id: ace-storage, resource: "acemod/ACE3@45c2524:addons/interact_menu/XEH_preInit.sqf, functions/fnc_addActionToObject.sqf, functions/fnc_addActionToClass.sqf, functions/fnc_compileMenu.sqf", read: 2026-11-04 }
  - { id: ms-sendinput, resource: "https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput", read: 2026-11-04 }
  - { id: ms-keyboard, resource: "https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input (updated 2026-07-08)", read: 2026-11-04 }
  - { id: ms-rawinput, resource: "https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input (updated 2026-03-18)", read: 2026-11-04 }
  - { id: ms-dicoop, resource: "https://learn.microsoft.com/en-us/previous-versions/windows/desktop/ee416848(v=vs.85) (DirectInput Cooperative Levels, archived)", read: 2026-11-04 }
  - { id: chen, resource: "https://devblogs.microsoft.com/oldnewthing/20050530-11/?p=35513 (Raymond Chen, You can't simulate keyboard input with PostMessage, 2005-05-30)", read: 2026-11-04 }
  - { id: mcp-input, resource: "Zihuange/arma3-mcp@380af42:adapters/input_tools.py (MIT)", read: 2026-11-04 }
  - { id: mcp-ext, resource: "Zihuange/arma3-mcp@380af42:game/extension/src/arma3_mcp_extension.cpp (MIT)", read: 2026-11-04 }
  - { id: mcp-decisions, resource: "Zihuange/arma3-mcp@380af42:docs/decisions.md, entries D29 to D34", read: 2026-11-04 }
  - { id: mcp-vision, resource: "Zihuange/arma3-mcp@380af42:docs/vision.md, test record 2026-09-17", read: 2026-11-04 }
  - { id: t1, resource: "research/10-prior-harness.md", read: 2026-11-04 }
---

# Making a player act: actions, key handlers and key input

This topic answers how a check puts a player where it wants them and makes them use a mod the way a player does (`CR-11`, `CR-12`), and whether key input can reach one game window without taking over the desktop (`CR-15`). Engine commands are checked against Arma 3 2.22, the current stable game update[^spotrep120][^spotrep122]. CBA is read at `8b15a04` (latest release v3.19.0)[^cbarel] and ACE3 at `45c2524` (latest release v3.21.2)[^acerel].

The short answer: a check drives a mod's real path by calling the code the mod registered (an `addAction` entry, an ACE interaction action, a CBA keybind handler) on the client where the player is local. Key input to one window in the background is not shown to work by any tool or document. The one tool that sends keys to Arma 3 uses the global input stream and refuses to run unless the game is the foreground window.

## 1. Moving, turning and aiming a player

| Command | Locality | What matters for a check |
|---|---|---|
| `setPosASL` | global argument, global effect | Places any object by sea-level position[^setposasl]. Since 2.10, using it on a remote unit inside a vehicle leaves the unit in the vehicle; the note says to remote-execute it to the unit's owner[^setposasl]. |
| `setDir` | local argument, global effect | Must run where the object is local. Set the direction before the position: setting it after "could lead to unpredictable behaviour"[^setdir]. For a player object a local `setDir` is enough to sync it[^setdir]. |
| `playMove` / `playMoveNow` | local argument, global effect | `playMove` queues a smooth transition, `playMoveNow` replaces the queue[^playmove]. |
| `switchMove` | local argument, global effect | Applies an animation at once, resetting aim and gesture states. Run on the unit's owner it syncs for JIP; on remote machines the change is transitional, so an immediate change everywhere needs global remote execution. An alternative syntax with time, blend and aim reset exists since 2.18[^switchmove]. |
| `setVelocity` | local argument, global effect | Sets a velocity vector in m/s, each component capped at 5000 m/s since 2.06[^setvelocity]. |
| `doMove` | local argument, global effect | Orders units to a position; `moveToCompleted` reports arrival[^domove]. The page says nothing about a unit controlled by a person[^domove]. |
| `lookAt`, `doWatch` | `doWatch`: local argument, global effect | Orders a unit to look at or watch a target or position[^lookat][^dowatch]. Neither page says it works on a unit a person controls[^lookat][^dowatch]. |
| `forceWeaponFire` | local argument, global effect | Fires a given muzzle and fire mode; the page says it "works on player"[^forcefire]. |

What it forces:

- **The place-player step runs on the unit's owner and sets direction, then position.** Locality decides which machine a command is sent to, so the scene API needs `CR-10`'s targeting by machine. Touches `CR-11`, `CR-10`.
- **Walking and looking are AI orders.** Whether `doMove`, `lookAt` and `doWatch` move or turn a unit that is a player's on a client is not documented: **needs a spike**. Pass condition: on a client, `player doMove _p` brings the player within 2 m of `_p` and `moveToCompleted player` turns true within a ceiling; `player lookAt _p` changes `eyeDirection player` towards `_p` by more than 45 degrees. Without them, a check moves a player by `setPosASL`/`setDir`, `setVelocity` and animations, and turns them with `setDir`. Touches `CR-11`.
- **Aiming the player's own view** (head and weapon, not the body) has no command found on these pages: **unverified**. A check that needs a weapon fired uses `forceWeaponFire`[^forcefire]. Touches `CR-11`, `CR-12`.

## 2. Using a mod's `addAction` entries

- `addAction` takes a global argument and has a local effect: the action exists only on the machine that added it, and the command is ignored on a dedicated server, so it must be added on each client[^addaction]. The script receives `[_target, _caller, _actionId, _arguments]`; the condition is evaluated each frame, unscheduled, with `_target`, `_this` and `_originalTarget` defined[^addaction].
- `actionIDs` (1.64) lists an object's user actions, and `actionParams` (1.64) returns each one's title, script, arguments, shortcut, condition, radius and memory point[^actionids][^actionparams]. A check can find a mod's action by its title and read its condition.
- `unit action ["User", target, index]` "executes the custom action with the given index of the target unit"[^actions]. The `UserType` variant is noted to crash the game (noted for ArmA 1.08)[^actions]. The `action` command's notes say some actions do not always execute, and that in singleplayer an action waits while the game is Alt-Tabbed[^action].
- `inGameUISetEventHandler` receives `PrevAction`, `NextAction` and `Action` events with the target, caller, action index, text and visibility, and overrides the action when it returns `true`[^inguieh]. It can observe and block an action, not perform one.
- Game update 2.22 added the `hideActions`, `hiddenActions` and `shownAction` commands and an `ActionTargetChanged` mission event handler[^spotrep120].

What it forces:

- **A "use action" step runs on the acting client**, finds the action by title through `actionIDs`/`actionParams`, evaluates its condition with the documented variables, fails the step with the reason when the condition is false, and fires it with `action ["User", ...]`. Touches `CR-12`, `CR-10`.
- **Whether `action ["User", ...]` honours the condition and radius is not documented: needs a spike.** Pass condition: with the player 50 m from an object whose action has radius 3, `player action ["User", obj, id]` does not run the action's script; at 1 m with the condition true it runs it once. If it ignores both, the framework checks them itself before firing. Touches `CR-12`.

## 3. Using ACE interaction menu entries

- An ACE action is an array built by `ace_interact_menu_fnc_createAction`: name, display name, icon, statement, condition, insert-children code, parameters, position, distance, flags and a modifier function. The function is client only[^ace-createaction]. Condition and statement receive `[_target, _player, _actionParams]`, with `_player` being `ace_player`[^ace-framework].
- Class actions live in the hash maps `ace_interact_menu_ActNamespace` (external) and `ace_interact_menu_ActSelfNamespace` (self), keyed by class; actions added to one object live in its variables `ace_interact_menu_actions` and `ace_interact_menu_selfActions`[^ace-storage]. `ace_interact_menu_fnc_findActionNode` returns the node for a path such as `["ACE_MainActions", "MyAction"]`, and is marked `Public: No`[^ace-findnode].
- When the player releases the menu key on a selected action, ACE clears its condition caches, sets `this` to the target, calls the condition with `[_target, _player, _actionParams]`, calls the statement if the condition holds, and raises the local CBA event `ace_interactMenuClosed`[^ace-keyup]. ACE has no public function that performs an action by its path[^ace-findnode][^ace-keyup].

What it forces:

- **A "use ACE action" step resolves the path and does what `fnc_keyUp` does**: condition, then statement, on the acting client, with `this` set. It also runs `insertChildren` to resolve children that exist only at run time. Touches `CR-12`.
- **This path uses ACE internals** (`findActionNode`, the namespaces), so it can break on an ACE update. The framework names the ACE version it was checked against and fails the step with that reason when a namespace is missing. Touches `CR-12`, `CR-06`.
- The menu's distance and line-of-sight checks are not part of `fnc_keyUp`[^ace-keyup], so a check that promises "the player can reach it" needs a separate assertion. Touches `CR-12`.

## 4. Firing a mod's CBA keybind handlers

- `CBA_fnc_addKeybind` exits on a machine without an interface. It stores the bound keys per action in `profileNamespace` (`cba_keybinding_registry_v3`), so the key a player has bound can differ from the mod's default. It registers the down and up code as CBA key handlers for every bound key[^cba-addkeybind].
- `CBA_fnc_getKeybind [mod, action]` returns the down code, the up code, the first key, hold settings and every bound key[^cba-getkeybind].
- CBA adds `KeyDown` and `KeyUp` handlers to the mission display, which call `cba_events_fnc_keyHandlerDown` and `cba_events_fnc_keyHandlerUp` unless the map is open[^cba-display]. Those take `[display, DIK, shift, ctrl, alt]`, track modifier state, match the modifiers exactly, apply the hold delay, run a key-up handler only after its matching key-down, and return whether to block the engine's own handling[^cba-keydown][^cba-keyup][^cba-addkeyhandler].
- The `arma-grappling-hook` harness already drives keys by calling the down and up code from `CBA_fnc_getKeybind` with `[display, dik, shift, ctrl, alt]`, and T1 recommends keeping that as the default way to act[^t1].

What it forces:

- **A "press keybind" step runs on the acting client** and has two levels. The default calls the down and up code from `CBA_fnc_getKeybind`, which is public. The stricter one calls `cba_events_fnc_keyHandlerDown` and `cba_events_fnc_keyHandlerUp` with the bound key from the profile, which also exercises CBA's modifier matching and down-up pairing, but uses internal functions. Touches `CR-12`.
- **The bound key comes from the client's profile**, so a run uses a fresh profile per machine or reads the binding before pressing. Touches `CR-12`, `CR-02`.
- A mod's own `displayAddEventHandler ["KeyDown", ...]` handler, not registered through CBA, is reachable only if the check knows its code; display handlers run from last to first added, and `KeyUp` cannot override engine handling[^displayeh]. Such a handler is the case where real key input (`CR-15`) is the only general path. Touches `CR-15`.

## 5. Key input to one window without taking the desktop

- `SendInput` inserts events "into the keyboard or mouse input stream", serially, is subject to UIPI, and does not reset the keyboard's current state[^ms-sendinput]. The system posts keyboard messages to "the foreground thread that created the window with the keyboard focus"[^ms-keyboard]. Input from `SendInput` therefore goes to whatever window has the focus, not to a chosen window.
- Posting `WM_KEYDOWN` to a window does not change the keyboard state that `GetKeyState` and `GetAsyncKeyState` report, and Raymond Chen's advice is to use `SendInput` instead[^chen]. A `WM_KEYDOWN` carries the scan code in `lParam`[^ms-keyboard].
- Raw input arrives as `WM_INPUT` and reaches a background window only if it registered with `RIDEV_INPUTSINK`[^ms-rawinput]. A DirectInput device at the foreground cooperative level is unacquired when its window goes to the background[^ms-dicoop].
- **Which of these Arma 3 reads** (window messages, raw input or DirectInput) is not documented on any page read here: **unverified**. The scripting side works in DIK scan codes[^cba-addkeybind][^displayeh].
- `Zihuange/arma3-mcp` (MIT) injects input from a DLL loaded in the game process. It uses `SendInput` with `KEYEVENTF_SCANCODE`, checks that the foreground window belongs to the game and refuses otherwise, because `SendInput` acts on the system input queue and would otherwise type into another program[^mcp-ext]. It has a `focus_game` tool that takes the foreground with `AttachThreadInput`, and keeps injection off by default for the same reason[^mcp-input]. Its log shows `SendInput` scan codes working in the game: Enter in the menus, holding Numpad 6 to turn the Eden camera, Shift and Alt with a mouse drag[^mcp-decisions]. Its test record from 2026-09-17 lists key input as still to test against a mission, needing the game in the foreground[^mcp-vision].
- With `-noPause` the game keeps running when its window has no focus[^startup].

What it forces:

- **`SendInput` breaks the rule "never take over the desktop"**: it needs the game in the foreground and types into whatever has the focus. The framework does not use it. Touches `CR-15` and PRD section 7.
- **Background key input is not shown possible. Needs a spike.** Pass condition: with the game client started with `-noPause`, its window not in the foreground and another window focused, a helper posts `WM_KEYDOWN` and `WM_KEYUP` with the right scan code in `lParam` to the game's window; a `KeyDown` handler on display 46 logs the DIK code, and a CBA keybind's down code runs once. The focused window receives nothing and the foreground does not change. If it fails, a second spike tries input from inside the game process (an extension that feeds the engine's input path), with the same pass condition; that path is **unverified** and may need hooking the engine.
- **Over Shadow: needs a spike.** The client renders only while the owner is connected through Shadow PC. Pass condition: the first spike passes both with the owner connected and with the stream disconnected.
- **Until a spike passes, `CR-15` stays a `should` that the MVP does not need**: every key path of `CR-12` is covered by calling handlers. Touches `CR-15`, `CR-12`.

## 6. Building a scene and removing it

- `createVehicle` creates an empty object with global effect, at the nearest free spot; Arma 3 randomises some vehicles unless `BIS_enableRandomization` is set to `false` at once, unscheduled[^createvehicle].
- `createUnit` needs an existing group and should run where the group is local; one syntax returns no unit reference[^createunit].
- `setDate` and `setOvercast` have a local effect, synchronised from the server periodically; `setDate` takes effect everywhere at once only through remote execution[^setdate][^setovercast]. `forceWeatherChange` runs on the server and skips the smooth transition[^forceweather].
- `deleteVehicle` cannot delete players or terrain objects, deletes on the next frame, and must not be used on crew: `deleteVehicleCrew` is for that. An array syntax exists since 2.18[^deletevehicle].

What it forces:

- **The scene API records every object it creates** so the reset between checks (`CR-02`) can delete them, crew first, and wait one frame for them to become null. Touches `CR-11`, `CR-02`.
- **Time and weather run on the server and are broadcast**, then forced, so every machine sees the same state before the step goes on. Touches `CR-11`, `CR-22`.
- **Created vehicles turn randomisation off by default**, so a scene is the same in every run. Touches `CR-11`, `CR-32`.

## Findings that need a spike

| Item | Pass condition |
|---|---|
| `doMove`, `lookAt`, `doWatch` on a player's unit | On a client, `player doMove _p` reaches within 2 m with `moveToCompleted` true within a ceiling; `player lookAt _p` turns `eyeDirection` towards `_p` by more than 45 degrees. |
| `action ["User", ...]` and the action's condition and radius | Out of radius the script does not run; in radius with the condition true it runs once. |
| Background key input by window messages | With `-noPause`, game not in the foreground, posted `WM_KEYDOWN`/`WM_KEYUP` fire a display-46 `KeyDown` handler and a CBA keybind once; the focused window and the foreground are unchanged. |
| The same over Shadow | Passes with the Shadow stream connected and disconnected. |

## Unverified

- Which input API Arma 3 reads the keyboard through.
- Any command that sets the view or weapon direction of a unit a person controls.
- Whether in-process input injection without the foreground is possible.

[^setposasl]: setPosASL
[^setdir]: setDir
[^playmove]: playMove
[^switchmove]: switchMove
[^domove]: doMove
[^setvelocity]: setVelocity
[^lookat]: lookAt
[^dowatch]: doWatch
[^forcefire]: forceWeaponFire
[^addaction]: addAction
[^action]: action
[^actions]: Arma 3: Actions
[^actionparams]: actionParams
[^actionids]: actionIDs
[^inguieh]: inGameUISetEventHandler
[^displayeh]: displayAddEventHandler
[^startup]: Arma 3: Startup Parameters
[^createvehicle]: createVehicle
[^createunit]: createUnit
[^setdate]: setDate
[^setovercast]: setOvercast
[^forceweather]: forceWeatherChange
[^deletevehicle]: deleteVehicle
[^spotrep120]: SPOTREP #00120, Game Update 2.22
[^spotrep122]: SPOTREP #00122, 2.22 hotfix
[^cbarel]: CBA_A3 release v3.19.0
[^cba-addkeybind]: CBA_fnc_addKeybind source
[^cba-getkeybind]: CBA_fnc_getKeybind source
[^cba-addkeyhandler]: CBA_fnc_addKeyHandler source
[^cba-keydown]: cba_events_fnc_keyHandlerDown source
[^cba-keyup]: cba_events_fnc_keyHandlerUp source
[^cba-display]: cba_events_fnc_initDisplayMission source
[^acerel]: ACE3 release v3.21.2
[^ace-framework]: ACE3 interaction menu framework documentation
[^ace-createaction]: ace_interact_menu_fnc_createAction source
[^ace-keyup]: ace_interact_menu_fnc_keyUp source
[^ace-findnode]: ace_interact_menu_fnc_findActionNode source
[^ace-storage]: ACE3 interact_menu action storage
[^ms-sendinput]: Microsoft, SendInput function
[^ms-keyboard]: Microsoft, Keyboard Input Overview
[^ms-rawinput]: Microsoft, Raw Input Overview
[^ms-dicoop]: Microsoft, DirectInput Cooperative Levels
[^chen]: Raymond Chen, You can't simulate keyboard input with PostMessage
[^mcp-input]: arma3-mcp input tools
[^mcp-ext]: arma3-mcp extension source
[^mcp-decisions]: arma3-mcp decisions log
[^mcp-vision]: arma3-mcp vision test record
[^t1]: T1, the harness of arma-grappling-hook
