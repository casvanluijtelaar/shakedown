/*
    skd_fnc_fail [message] - ends the check fail / explicit. Does not return.
*/
if !(["skd_fnc_fail"] call skd_fnc_guard) exitWith {};
params ["_skd_message"];
if (isNil "_skd_message" || { !(_skd_message isEqualType "") }) exitWith {
    ["skd_fnc_fail", 1, "message", "a String", if (isNil "_skd_message") then { nil } else { _skd_message }] call skd_fnc_argError;
};
["fail", "explicit", _skd_message] call skd_fnc_end;
