/*
    skd_fnc_place [unit, position, direction, ceiling] - places a unit at a position (ASL) facing a
    direction, where the unit is local, direction first, and returns once the server sees it within
    0.1 m and 3 degrees (check API, "Building a scene"; SCN-02). unit is an object or a machine
    name, which stands for that machine's player.

    Returns: true.
*/
if !(["skd_fnc_place"] call skd_fnc_guard) exitWith {};
params ["_skd_unit", "_skd_position", "_skd_direction", "_skd_ceiling"];
private _skd_dedicated = (skd_check get "mode") isEqualTo "dedicated";
private _skd_on = [];
if (!isNil "_skd_unit" && { _skd_unit isEqualType "" }) then {
    private _skd_targets = ["skd_fnc_place", 1, _skd_unit, false] call skd_fnc_targets;
    if (isNil "_skd_targets") then { _skd_unit = nil } else {
        private _skd_name = _skd_targets select 0;
        _skd_unit = [_skd_name] call skd_fnc_unitOf;
        _skd_on = [_skd_name];
        if (isNull _skd_unit) then {
            ["fail", "check-error", format ["skd_fnc_place: %1 has no player to place. Name a client, p1 to pN, or give the unit.", _skd_name]] call skd_fnc_end;
            _skd_unit = nil;
        };
    };
} else {
    if (isNil "_skd_unit" || { !(_skd_unit isEqualType objNull) } || { isNull _skd_unit }) then {
        ["skd_fnc_place", 1, "unit", "an object or a machine name", if (isNil "_skd_unit") then { nil } else { _skd_unit }] call skd_fnc_argError;
        _skd_unit = nil;
    };
};
if (isNil "_skd_unit") exitWith {};
if (isNil "_skd_position" || { !(_skd_position isEqualTypeArray [0, 0, 0]) }) exitWith {
    ["skd_fnc_place", 2, "position", "an ASL position [x, y, z] of three numbers", if (isNil "_skd_position") then { nil } else { _skd_position }] call skd_fnc_argError;
};
if (isNil "_skd_direction" || { !(_skd_direction isEqualType 0) }) exitWith {
    ["skd_fnc_place", 3, "direction", "a number of degrees", if (isNil "_skd_direction") then { nil } else { _skd_direction }] call skd_fnc_argError;
};
_skd_ceiling = ["skd_fnc_place", 4, if (isNil "_skd_ceiling") then { nil } else { _skd_ceiling }] call skd_fnc_ceiling;
if (isNil "_skd_ceiling") exitWith {};

// The unit is set where it is local (SCN-02): here, in singleplayer and for a unit local to the
// server; otherwise on its player's machine, or on whichever machine holds it.
private _skd_set = {
    params ["_u", "_p", "_d"];
    if (!local _u) exitWith { false };
    _u setDir _d;
    _u setPosASL _p;
    _u setVelocity [0, 0, 0];
    true
};
if (!_skd_dedicated || { local _skd_unit }) then {
    [_skd_unit, _skd_position, _skd_direction] call _skd_set;
} else {
    if (_skd_on isEqualTo [] && { isPlayer _skd_unit } && { (name _skd_unit) in (skd_check get "machines") }) then { _skd_on = [name _skd_unit] };
    if (_skd_on isEqualTo []) then { _skd_on = (skd_check get "machines") - [skd_machine] };
    {
        ([_x, _skd_set, [_skd_unit, _skd_position, _skd_direction], _skd_ceiling] call skd_fnc_runOn) params ["_skd_done", "_skd_value"];
        if (_skd_value isEqualTo [true]) exitWith {};
    } forEach _skd_on;
};
private _skd_there = {
    private _d = abs ((getDir _skd_unit) - _skd_direction) % 360;
    ((getPosASL _skd_unit) distance _skd_position) <= 0.1 && { (_d min (360 - _d)) <= 3 }
};
private _skd_start = diag_tickTime;
waitUntil { call _skd_there || { diag_tickTime - _skd_start > _skd_ceiling } };
if (call _skd_there) exitWith { true };
call skd_fnc_atCeiling;
["fail", "wait-ceiling", format ["skd_fnc_place: the server did not see the unit at %1 facing %2 within %3 s; it last saw it at %4 facing %5.",
    _skd_position, _skd_direction, _skd_ceiling, getPosASL _skd_unit, getDir _skd_unit]] call skd_fnc_end;
