/*
    skd_fnc_tryWaitUntil [machine, condition, description, ceiling, args] - waits until the
    condition holds on the machine (check API, "Waits"; OBS-07).

    Returns: true if the condition held within the ceiling, false if not.
*/
if !(["skd_fnc_tryWaitUntil"] call skd_fnc_guard) exitWith {};
["skd_fnc_tryWaitUntil", true, _this] call skd_fnc_wait
