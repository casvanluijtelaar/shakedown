// [value, delay] call skdfx_fnc_setFlagLater: sets skdfx_flag as skdfx_fnc_setFlag does, delay
// seconds of real time later, and returns at once.
params ["_value", "_delay"];
if (!isServer) exitWith { false };
[_value, diag_tickTime + _delay] spawn {
    params ["_value", "_at"];
    waitUntil { diag_tickTime >= _at };
    missionNamespace setVariable ["skdfx_flag", _value, true];
};
true
