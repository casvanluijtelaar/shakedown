/*
    skd_fnc_waitUntil [machine, condition, description, ceiling, args] - waits until the condition
    holds on the machine; fails the check fail / wait-ceiling at the ceiling (check API, "Waits").

    Returns: true.
*/
if !(["skd_fnc_waitUntil"] call skd_fnc_guard) exitWith {};
["skd_fnc_waitUntil", false, _this] call skd_fnc_wait
