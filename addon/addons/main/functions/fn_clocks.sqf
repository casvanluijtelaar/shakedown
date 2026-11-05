/*
    The clocks of OBS-05 on this machine, now: wall clock (UTC), real time, mission time and frame.

    The numbers travel as text with fixed decimals, because toJSON keeps only 6 significant digits.

    Returns: HashMap with "utc", "tick", "time", "frame".
*/
private _t = systemTimeUTC;
private _pad = {
    params ["_n", "_width"];
    private _s = str _n;
    while {count _s < _width} do { _s = "0" + _s };
    _s
};
createHashMapFromArray [
    ["utc", format ["%1-%2-%3T%4:%5:%6.%7Z",
        [_t select 0, 4] call _pad, [_t select 1, 2] call _pad, [_t select 2, 2] call _pad,
        [_t select 3, 2] call _pad, [_t select 4, 2] call _pad, [_t select 5, 2] call _pad,
        [_t select 6, 3] call _pad]],
    ["tick", diag_tickTime toFixed 3],
    ["time", time toFixed 3],
    ["frame", diag_frameNo toFixed 0]
]
