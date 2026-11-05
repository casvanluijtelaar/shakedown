/*
    skd_fnc_expectEach [machines, value, op, expected, description, ceiling, args] - reads the value
    on every machine every frame until the comparison holds on all of them at once (check API,
    "Assertions"; OBS-06).

    Returns: HashMap from machine name to the last value read.
*/
if !(["skd_fnc_expectEach"] call skd_fnc_guard) exitWith {};
["skd_fnc_expectEach", "each", _this] call skd_fnc_expectPoll
