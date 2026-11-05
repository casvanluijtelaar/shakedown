/*
    skd_fnc_expectAgree [machines, value, description, tolerance, ceiling, args] - reads the value
    on every machine every frame until every machine agrees with the first one named, within the
    tolerance for numbers and positions (check API, "Assertions"; OBS-06).

    Returns: HashMap from machine name to the last value read.
*/
if !(["skd_fnc_expectAgree"] call skd_fnc_guard) exitWith {};
["skd_fnc_expectAgree", "agree", _this] call skd_fnc_expectPoll
