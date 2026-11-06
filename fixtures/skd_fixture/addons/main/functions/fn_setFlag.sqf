// [value] call skdfx_fnc_setFlag: on the server, sets skdfx_flag public and returns true;
// elsewhere does nothing and returns false.
params ["_value"];
if (!isServer) exitWith { false };
missionNamespace setVariable ["skdfx_flag", _value, true];
true
