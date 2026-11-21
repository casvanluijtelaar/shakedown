/*
    skd_fnc_useAceAction [machine, target, path, ceiling] - makes the player of a machine use an
    action of a mod's ACE interaction menu, found by its path of action names, through ACE's own
    key-up code (check API, "ACE interaction actions"; ACT-04, INT-ACE-03). target is an object,
    or a machine name for that machine's player; the acting machine's own player opens its self
    menu. The acting machine waits, every frame to the ceiling, until the path resolves with every
    condition true, then runs the action.

    Returns: true once ACE's key-up code ran the action.
*/
if !(["skd_fnc_useAceAction"] call skd_fnc_guard) exitWith {};
params ["_skd_machine", "_skd_target", "_skd_path", "_skd_ceiling"];
private _skd_targets = ["skd_fnc_useAceAction", 1, if (isNil "_skd_machine") then { nil } else { _skd_machine }, false] call skd_fnc_targets;
if (isNil "_skd_targets") exitWith {};
private _skd_on = _skd_targets select 0;
private _skd_dedicated = (skd_check get "mode") isEqualTo "dedicated";
if (_skd_dedicated && { !((_skd_on select [0, 1]) isEqualTo "p") }) exitWith {
    ["fail", "check-error", format ["skd_fnc_useAceAction: %1 has no player to use an action. Name a client, p1 to pN.", _skd_on]] call skd_fnc_end;
};
if (!isNil "_skd_target" && { _skd_target isEqualType "" }) then {
    private _skd_named = ["skd_fnc_useAceAction", 2, _skd_target, false] call skd_fnc_targets;
    if (isNil "_skd_named") then { _skd_target = nil } else {
        _skd_target = [_skd_named select 0] call skd_fnc_unitOf;
        if (isNull _skd_target) then {
            ["fail", "check-error", format ["skd_fnc_useAceAction: %1 has no player whose menu to open. Name a client, p1 to pN, or give the object.", _skd_named select 0]] call skd_fnc_end;
            _skd_target = nil;
        };
    };
} else {
    if (!isNil "_skd_target" && { _skd_target isEqualTo objNull } && { _skd_dedicated }) then {
        ["fail", "check-error", "skd_fnc_useAceAction: argument 2 (target) is objNull. On a dedicated session the check body runs on the server, which has no player, so `player` there is objNull; name a player's unit with [""p1""] call skd_fnc_player, or give the machine name ""p1""."] call skd_fnc_end;
        _skd_target = nil;
    } else {
        if (isNil "_skd_target" || { !(_skd_target isEqualType objNull) } || { isNull _skd_target }) then {
            ["skd_fnc_useAceAction", 2, "target", "the object whose menu holds the action, or a machine name", if (isNil "_skd_target") then { nil } else { _skd_target }] call skd_fnc_argError;
            _skd_target = nil;
        };
    };
};
if (isNil "_skd_target") exitWith {};
if (isNil "_skd_path" || { !(_skd_path isEqualType []) } || { _skd_path isEqualTo [] } || { (_skd_path findIf { !(_x isEqualType "") }) >= 0 }) exitWith {
    ["skd_fnc_useAceAction", 3, "path", "an array of one or more action names (Strings), such as [""ACE_MainActions"", ""myAction""]", if (isNil "_skd_path") then { nil } else { _skd_path }] call skd_fnc_argError;
};
_skd_ceiling = ["skd_fnc_useAceAction", 4, if (isNil "_skd_ceiling") then { nil } else { _skd_ceiling }] call skd_fnc_ceiling;
if (isNil "_skd_ceiling") exitWith {};
// The acting machine waits for the path and runs the action; its own ceiling ends first.
([_skd_on, { _this call skd_fnc_useAceActionLocal }, [netId _skd_target, _skd_path, _skd_ceiling], _skd_ceiling + 5] call skd_fnc_runOn) params ["_skd_done", "_skd_value"];
private _skd_state = if (_skd_value isEqualTo []) then { ["no-target"] } else { _skd_value select 0 };
private _skd_shown = str (_skd_path joinString " > ");
switch (_skd_state select 0) do {
    case "fired": { true };
    case "no-player": {
        ["fail", "check-error", format ["skd_fnc_useAceAction: %1 has no player to use an action.", _skd_on]] call skd_fnc_end;
    };
    case "no-ace": {
        ["fail", "check-error", format ["skd_fnc_useAceAction: ACE is not loaded on %1 (no ace_interact_menu), so it has no ACE action %2. Add ACE to the setup's mods, or use no ACE action in this setup.", _skd_on, _skd_shown]] call skd_fnc_end;
    };
    case "ace-changed": {
        ["fail", "check-error", format ["skd_fnc_useAceAction: %1 loaded ACE %2, which lacks %3, an ACE internal the framework uses for interaction actions; the framework was checked against ACE 3.21.2 (INT-ACE-03).", _skd_on, _skd_state select 1, _skd_state select 2]] call skd_fnc_end;
    };
    default {
        call skd_fnc_atCeiling;
        private _skd_why = switch (_skd_state select 0) do {
            case "missing": {
                _skd_state params ["", "_skd_parent", "_skd_names"];
                format ["%1 has no action %2; the actions there are: %3.",
                    if (_skd_parent isEqualTo []) then { "the menu's root" } else { str (_skd_parent joinString " > ") },
                    str (_skd_path select (count _skd_parent)),
                    if (_skd_names isEqualTo []) then { "none" } else { (_skd_names apply { str _x }) joinString ", " }]
            };
            case "condition": { format ["the condition of %1 was false.", str ((_skd_state select 1) joinString " > ")] };
            case "folder": { format ["%1 is a folder of actions with no statement of its own; its actions are: %2.", _skd_shown, ((_skd_state select 1) apply { str _x }) joinString ", "] };
            default { format ["%1 did not know the target %2.", _skd_on, netId _skd_target] };
        };
        ["fail", "assertion", format ["skd_fnc_useAceAction: %1 could not use the ACE action %2 on %3 within %4 s: %5", _skd_on, _skd_shown, typeOf _skd_target, _skd_ceiling, _skd_why], nil, _skd_on] call skd_fnc_end;
    };
};
