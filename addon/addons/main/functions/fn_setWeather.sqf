/*
    skd_fnc_setWeather [overcast, fog, rain, ceiling] - sets the overcast, fog and rain on the
    server, forces the weather change, and returns once every machine reports the overcast within
    0.01 (check API, "Building a scene"; SCN-03).

    Returns: true.
*/
if !(["skd_fnc_setWeather"] call skd_fnc_guard) exitWith {};
params ["_skd_overcast", "_skd_fog", "_skd_rain", "_skd_ceiling"];
private _skd_bad = -1;
{
    if (_skd_bad < 0 && { isNil "_x" || { !(_x isEqualType 0) } || { _x < 0 } || { _x > 1 } }) then { _skd_bad = _forEachIndex };
} forEach [
    if (isNil "_skd_overcast") then { nil } else { _skd_overcast },
    if (isNil "_skd_fog") then { nil } else { _skd_fog },
    if (isNil "_skd_rain") then { nil } else { _skd_rain }
];
if (_skd_bad >= 0) exitWith {
    private _skd_value = [_this, _skd_bad] call { params ["_a", "_i"]; if (_a isEqualType [] && { count _a > _i }) then { _a select _i } else { nil } };
    ["skd_fnc_setWeather", _skd_bad + 1, ["overcast", "fog", "rain"] select _skd_bad, "a number from 0 to 1", if (isNil "_skd_value") then { nil } else { _skd_value }] call skd_fnc_argError;
};
_skd_ceiling = ["skd_fnc_setWeather", 4, if (isNil "_skd_ceiling") then { nil } else { _skd_ceiling }] call skd_fnc_ceiling;
if (isNil "_skd_ceiling") exitWith {};
// The server sets the weather and forces the change; each machine sets it too, so it does not
// wait for the server's weather to reach it, and reports what it then reads, polled to the
// ceiling (SCN-03). In singleplayer every machine is this one.
private _skd_apply = {
    params ["_o", "_f", "_r", "_c"];
    0 setOvercast _o;
    0 setFog _f;
    0 setRain _r;
    forceWeatherChange;
    private _s = diag_tickTime;
    waitUntil { abs (overcast - _o) <= 0.01 || { diag_tickTime - _s > _c } };
    [overcast, fog, rain]
};
private _skd_seen = createHashMap;
private _skd_start = diag_tickTime;
{
    private _skd_left = (_skd_ceiling - (diag_tickTime - _skd_start)) max 0.001;
    ([_x, _skd_apply, [_skd_overcast, _skd_fog, _skd_rain, _skd_left], _skd_left + 1] call skd_fnc_runOn) params ["_skd_done", "_skd_value"];
    _skd_seen set [_x, if (_skd_value isEqualTo []) then { nil } else { _skd_value select 0 }];
} forEach ([skd_machine] + ((skd_check get "machines") - [skd_machine]));
private _skd_off = (keys _skd_seen) select {
    private _v = _skd_seen getOrDefault [_x, []];
    !(_v isEqualType []) || { count _v < 1 } || { abs ((_v select 0) - _skd_overcast) > 0.01 }
};
if (_skd_off isEqualTo []) exitWith { true };
call skd_fnc_atCeiling;
["fail", "wait-ceiling", format ["skd_fnc_setWeather: the machines did not report an overcast of %1 within %2 s; %3.", _skd_overcast, _skd_ceiling,
    (_skd_off apply { format ["%1 reported overcast, fog, rain %2", _x, _skd_seen getOrDefault [_x, "nothing"]] }) joinString ", "], _skd_seen] call skd_fnc_end;
