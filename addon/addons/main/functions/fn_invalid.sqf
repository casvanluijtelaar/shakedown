/*
    skd_fnc_invalid [message] - ends the check invalid / explicit, for a cause outside the mod that
    the check detects itself. Does not return.
*/
if !(["skd_fnc_invalid"] call skd_fnc_guard) exitWith {};
params ["_skd_message"];
if (isNil "_skd_message" || { !(_skd_message isEqualType "") }) exitWith {
    ["skd_fnc_invalid", 1, "message", "a String", if (isNil "_skd_message") then { nil } else { _skd_message }] call skd_fnc_argError;
};
["invalid", "explicit", _skd_message] call skd_fnc_end;
