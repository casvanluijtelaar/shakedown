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
([_skd_on, { _this call skd_fnc_pressKeyLocal }, [_skd_mod, _skd_action, _skd_mode], 10] call skd_fnc_runOn) params ["_skd_done", "_skd_value"];
private _skd_state = if (_skd_value isEqualTo []) then { ["no-cba"] } else { _skd_value select 0 };
switch (_skd_state select 0) do {
    case "pressed": { true };
    case "no-cba": {
        ["fail", "check-error", format ["skd_fnc_pressKey: CBA is not loaded on %1, so it has no keybind %2 / %3. Add CBA to the setup's mods, or press no key in this setup.", _skd_on, str _skd_mod, str _skd_action]] call skd_fnc_end;
    };
    default {
        _skd_state params ["", "_skd_ofMod", "_skd_mods"];
        private _skd_message = if (_skd_ofMod isEqualTo []) then {
            format ["skd_fnc_pressKey: %1 has no keybind %2 / %3, and no keybind is registered for the mod %2. The mods with keybinds are (in lower case): %4.",
                _skd_on, str _skd_mod, str _skd_action, _skd_mods joinString ", "]
        } else {
            format ["skd_fnc_pressKey: %1 has no keybind %2 / %3. The keybinds registered for %2 are (in lower case): %4.",
                _skd_on, str _skd_mod, str _skd_action, _skd_ofMod joinString ", "]
        };
        ["fail", "check-error", _skd_message] call skd_fnc_end;
    };
};
