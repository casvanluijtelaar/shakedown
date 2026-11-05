/*
    skd_fnc_pressKey [machine, mod, action, mode] - presses a keybind a mod registered with CBA on
    a machine: calls the code CBA_fnc_getKeybind returns for it with [display 46, DIK, shift, ctrl,
    alt] of its first bound key (check API, "Acting through the mod"; ACT-02). mode is "press" (down
    then up), "down" or "up". Without CBA, or for a keybind that is not registered, the check ends
    fail / check-error with the keybinds that are (ACT-03).

    Returns: true once the keybind's code ran.
*/
if !(["skd_fnc_pressKey"] call skd_fnc_guard) exitWith {};
params ["_skd_machine", "_skd_mod", "_skd_action", "_skd_mode"];
private _skd_targets = ["skd_fnc_pressKey", 1, if (isNil "_skd_machine") then { nil } else { _skd_machine }, false] call skd_fnc_targets;
if (isNil "_skd_targets") exitWith {};
if (isNil "_skd_mod" || { !(_skd_mod isEqualType "") }) exitWith {
    ["skd_fnc_pressKey", 2, "mod", "the mod name the keybind was registered with (String)", if (isNil "_skd_mod") then { nil } else { _skd_mod }] call skd_fnc_argError;
};
if (isNil "_skd_action" || { !(_skd_action isEqualType "") }) exitWith {
    ["skd_fnc_pressKey", 3, "action", "the action name the keybind was registered with (String)", if (isNil "_skd_action") then { nil } else { _skd_action }] call skd_fnc_argError;
};
if (isNil "_skd_mode") then { _skd_mode = "press" };
if (!(_skd_mode isEqualType "") || { !(_skd_mode in ["press", "down", "up"]) }) exitWith {
    ["skd_fnc_pressKey", 4, "mode", "one of press down up", _skd_mode] call skd_fnc_argError;
};
private _skd_on = _skd_targets select 0;
// In singleplayer the machine is this one.
if (isNil "CBA_fnc_getKeybind") exitWith {
    ["fail", "check-error", format ["skd_fnc_pressKey: CBA is not loaded on %1, so it has no keybind %2 / %3. Add CBA to the setup's mods, or press no key in this setup.", _skd_on, str _skd_mod, str _skd_action]] call skd_fnc_end;
};
private _skd_bind = [_skd_mod, _skd_action] call CBA_fnc_getKeybind;
if (isNil "_skd_bind") exitWith {
    // CBA keeps its keybinds in the namespace cba_keybinding_actions, one variable per
    // "<mod>$<action>" in lower case (INT-CBA-01).
    private _skd_all = allVariables (missionNamespace getVariable ["cba_keybinding_actions", locationNull]);
    private _skd_prefix = toLower _skd_mod + "$";
    private _skd_ofMod = (_skd_all select { (_x find _skd_prefix) == 0 }) apply { _x select [count _skd_prefix] };
    private _skd_mods = [];
    { _skd_mods pushBackUnique ((_x splitString "$") select 0) } forEach _skd_all;
    private _skd_message = if (_skd_ofMod isEqualTo []) then {
        format ["skd_fnc_pressKey: %1 has no keybind %2 / %3, and no keybind is registered for the mod %2. The mods with keybinds are (in lower case): %4.",
            _skd_on, str _skd_mod, str _skd_action, _skd_mods joinString ", "]
    } else {
        format ["skd_fnc_pressKey: %1 has no keybind %2 / %3. The keybinds registered for %2 are (in lower case): %4.",
            _skd_on, str _skd_mod, str _skd_action, _skd_ofMod joinString ", "]
    };
    ["fail", "check-error", _skd_message] call skd_fnc_end;
};
private _skd_keys = _skd_bind param [8, []];
private _skd_key = if (_skd_keys isEqualType [] && { _skd_keys isNotEqualTo [] }) then { _skd_keys select 0 } else { _skd_bind select 5 };
_skd_key params [["_skd_dik", 0], ["_skd_mods", [false, false, false]]];
_skd_mods params [["_skd_shift", false], ["_skd_ctrl", false], ["_skd_alt", false]];
private _skd_params = [findDisplay 46, _skd_dik, _skd_shift, _skd_ctrl, _skd_alt];
// The engine calls a key handler unscheduled; isNil runs the code the same way.
if (_skd_mode in ["press", "down"]) then {
    isNil { _skd_params call (_skd_bind select 3) };
};
if (_skd_mode in ["press", "up"]) then {
    isNil { _skd_params call (_skd_bind select 4) };
};
true
