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
// The server sets the date and broadcasts it: each machine sets it too, and reports the date it
// then reads, polled to the ceiling (SCN-03). In singleplayer every machine is this one.
private _skd_apply = {
    params ["_d", "_c"];
    setDate _d;
    private _s = diag_tickTime;
    waitUntil { (date select [0, 5]) isEqualTo (_d select [0, 5]) || { diag_tickTime - _s > _c } };
    date select [0, 5]
};
private _skd_seen = createHashMap;
private _skd_start = diag_tickTime;
{
    private _skd_left = (_skd_ceiling - (diag_tickTime - _skd_start)) max 0.001;
    ([_x, _skd_apply, [_skd_date, _skd_left], _skd_left + 1] call skd_fnc_runOn) params ["_skd_done", "_skd_value"];
    _skd_seen set [_x, if (_skd_value isEqualTo []) then { nil } else { _skd_value select 0 }];
} forEach ([skd_machine] + ((skd_check get "machines") - [skd_machine]));
private _skd_off = (keys _skd_seen) select { !((_skd_seen getOrDefault [_x, []]) isEqualTo (_skd_date select [0, 5])) };
if (_skd_off isEqualTo []) exitWith { true };
call skd_fnc_atCeiling;
["fail", "wait-ceiling", format ["skd_fnc_setDate: the machines did not report %1 within %2 s; %3.", _skd_date, _skd_ceiling,
    (_skd_off apply { format ["%1 reported %2", _x, _skd_seen getOrDefault [_x, "nothing"]] }) joinString ", "], _skd_seen] call skd_fnc_end;
