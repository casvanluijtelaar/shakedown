/*
    The part of skd_fnc_pressKey that runs on the pressing machine (ACT-02, ACT-03): calls the code
    CBA_fnc_getKeybind returns for the keybind with [display 46, DIK, shift, ctrl, alt] of its
    first bound key. CBA keeps its keybinds in the namespace cba_keybinding_actions, one variable
    per "<mod>$<action>" in lower case (INT-CBA-01).

    Arguments: [mod, action, mode]
    Returns: ["pressed"], ["no-cba"], or ["missing", the keybinds of that mod, the mods with keybinds].
*/
params ["_mod", "_action", "_mode"];
if (isNil "CBA_fnc_getKeybind") exitWith { ["no-cba"] };
private _bind = [_mod, _action] call CBA_fnc_getKeybind;
if (isNil "_bind") exitWith {
    private _all = allVariables (missionNamespace getVariable ["cba_keybinding_actions", locationNull]);
    private _prefix = toLower _mod + "$";
    private _ofMod = (_all select { (_x find _prefix) == 0 }) apply { _x select [count _prefix] };
    private _mods = [];
    { _mods pushBackUnique ((_x splitString "$") select 0) } forEach _all;
    ["missing", _ofMod, _mods]
};
private _keys = _bind param [8, []];
private _key = if (_keys isEqualType [] && { _keys isNotEqualTo [] }) then { _keys select 0 } else { _bind select 5 };
_key params [["_dik", 0], ["_mods", [false, false, false]]];
_mods params [["_shift", false], ["_ctrl", false], ["_alt", false]];
private _params = [findDisplay 46, _dik, _shift, _ctrl, _alt];
// The engine calls a key handler unscheduled; isNil runs the code the same way.
if (_mode in ["press", "down"]) then {
    isNil { _params call (_bind select 3) };
};
if (_mode in ["press", "up"]) then {
    isNil { _params call (_bind select 4) };
};
["pressed"]
