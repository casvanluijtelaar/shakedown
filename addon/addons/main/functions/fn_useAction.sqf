/*
    skd_fnc_useAction [machine, target, title, ceiling] - makes the player of a machine use an
    addAction action on a target, found by its title after removing structured-text tags (check
    API, "Acting through the mod"; ACT-01). It waits, every frame to the ceiling, until the action
    exists, its condition holds and the player is within its radius, then fires it with
    action ["User", target, id]. The engine does not check the condition or the radius of an
    action fired that way (SPK-10), so this function checks both.

    Returns: true once the action was fired.
*/
if !(["skd_fnc_useAction"] call skd_fnc_guard) exitWith {};
params ["_skd_machine", "_skd_target", "_skd_title", "_skd_ceiling"];
private _skd_targets = ["skd_fnc_useAction", 1, if (isNil "_skd_machine") then { nil } else { _skd_machine }, false] call skd_fnc_targets;
if (isNil "_skd_targets") exitWith {};
if (!isNil "_skd_target" && { _skd_target isEqualTo objNull } && { (skd_check get "mode") isEqualTo "dedicated" }) exitWith {
    ["fail", "check-error", "skd_fnc_useAction: argument 2 (target) is objNull. On a dedicated session the check body runs on the server, which has no player, so `player` there is objNull; name a player's unit with [""p1""] call skd_fnc_player."] call skd_fnc_end;
};
if (isNil "_skd_target" || { !(_skd_target isEqualType objNull) } || { isNull _skd_target }) exitWith {
    ["skd_fnc_useAction", 2, "target", "the object carrying the action", if (isNil "_skd_target") then { nil } else { _skd_target }] call skd_fnc_argError;
};
if (isNil "_skd_title" || { !(_skd_title isEqualType "") }) exitWith {
    ["skd_fnc_useAction", 3, "title", "the action's title (String)", if (isNil "_skd_title") then { nil } else { _skd_title }] call skd_fnc_argError;
};
_skd_ceiling = ["skd_fnc_useAction", 4, if (isNil "_skd_ceiling") then { nil } else { _skd_ceiling }] call skd_fnc_ceiling;
if (isNil "_skd_ceiling") exitWith {};
private _skd_on = _skd_targets select 0;
if ((skd_check get "mode") isEqualTo "dedicated" && { !((_skd_on select [0, 1]) isEqualTo "p") }) exitWith {
    ["fail", "check-error", format ["skd_fnc_useAction: %1 has no player to use an action. Name a client, p1 to pN.", _skd_on]] call skd_fnc_end;
};
// The acting machine waits for the action and fires it; its own ceiling ends first.
([_skd_on, { _this call skd_fnc_useActionLocal }, [netId _skd_target, _skd_title, _skd_ceiling], _skd_ceiling + 5] call skd_fnc_runOn) params ["_skd_done", "_skd_value"];
private _skd_shown = format ["%1 %2", typeOf _skd_target, netId _skd_target];
private _skd_state = if (_skd_value isEqualTo []) then { ["missing", []] } else { _skd_value select 0 };
if ((_skd_state select 0) isEqualTo "fired") exitWith { true };
if ((_skd_state select 0) isEqualTo "no-player") exitWith {
    ["fail", "check-error", format ["skd_fnc_useAction: %1 has no player to use an action.", _skd_on]] call skd_fnc_end;
};
call skd_fnc_atCeiling;
private _skd_why = switch (_skd_state select 0) do {
    case "missing": { format ["%1 has no action titled %2; its actions are: %3.", _skd_shown, str _skd_title, (_skd_state select 1) apply { str _x } joinString ", "] };
    case "condition": { format ["the condition of %1 on %2 was false: %3.", str _skd_title, _skd_shown, _skd_state select 1] };
    default { format ["the player was %1 m from %2, outside the radius %3 m of %4.", (_skd_state select 1) toFixed 2, _skd_shown, _skd_state select 2, str _skd_title] };
};
["fail", "assertion", format ["skd_fnc_useAction: %1 could not use %2 within %3 s: %4", _skd_on, str _skd_title, _skd_ceiling, _skd_why], nil, _skd_on] call skd_fnc_end;
