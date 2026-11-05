/*
    skd_fnc_setDate [date, ceiling] - sets the date and time of day on the server and returns once
    every machine reports it to the minute (check API, "Building a scene"; SCN-03).

    Returns: true.
*/
if !(["skd_fnc_setDate"] call skd_fnc_guard) exitWith {};
params ["_skd_date", "_skd_ceiling"];
if (isNil "_skd_date" || { !(_skd_date isEqualTypeArray [0, 0, 0, 0, 0]) }) exitWith {
    ["skd_fnc_setDate", 1, "date", "[year, month, day, hour, minute] of five numbers", if (isNil "_skd_date") then { nil } else { _skd_date }] call skd_fnc_argError;
};
_skd_ceiling = ["skd_fnc_setDate", 2, if (isNil "_skd_ceiling") then { nil } else { _skd_ceiling }] call skd_fnc_ceiling;
if (isNil "_skd_ceiling") exitWith {};
// In singleplayer the server and every machine are this one.
setDate _skd_date;
private _skd_start = diag_tickTime;
waitUntil { (date select [0, 5]) isEqualTo (_skd_date select [0, 5]) || { diag_tickTime - _skd_start > _skd_ceiling } };
if ((date select [0, 5]) isEqualTo (_skd_date select [0, 5])) exitWith { true };
call skd_fnc_atCeiling;
["fail", "wait-ceiling", format ["skd_fnc_setDate: the machines did not report %1 within %2 s; %3 reported %4.", _skd_date, _skd_ceiling, skd_machine, date]] call skd_fnc_end;
