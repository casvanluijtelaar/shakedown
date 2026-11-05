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
// In singleplayer the server and every machine are this one.
0 setOvercast _skd_overcast;
0 setFog _skd_fog;
0 setRain _skd_rain;
forceWeatherChange;
private _skd_start = diag_tickTime;
waitUntil { abs (overcast - _skd_overcast) <= 0.01 || { diag_tickTime - _skd_start > _skd_ceiling } };
if (abs (overcast - _skd_overcast) <= 0.01) exitWith { true };
call skd_fnc_atCeiling;
["fail", "wait-ceiling", format ["skd_fnc_setWeather: the machines did not report an overcast of %1 within %2 s; %3 reported overcast %4, fog %5, rain %6.",
    _skd_overcast, _skd_ceiling, skd_machine, overcast, fog, rain]] call skd_fnc_end;
