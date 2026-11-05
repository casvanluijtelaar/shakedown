/*
    skd_fnc_expectOn [machine, value, op, expected, description, ceiling, args] - reads the value
    on the machine every frame until the comparison holds; fails the check fail / assertion with
    the last value read at the ceiling (check API, "Assertions"; OBS-04).

    Returns: the last value read.
*/
if !(["skd_fnc_expectOn"] call skd_fnc_guard) exitWith {};
["skd_fnc_expectOn", "on", _this] call skd_fnc_expectPoll
