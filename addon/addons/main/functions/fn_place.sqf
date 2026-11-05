/*
    skd_fnc_place [unit, position, direction, ceiling] - places a unit at a position (ASL) facing a
    direction, where the unit is local, direction first, and returns once the server sees it within
    0.1 m and 3 degrees (check API, "Building a scene"; SCN-02). unit is an object or a machine
    name, which stands for that machine's player.

    Returns: true.
*/
if !(["skd_fnc_place"] call skd_fnc_guard) exitWith {};
params ["_skd_unit", "_skd_position", "_skd_direction", "_skd_ceiling"];
if (!isNil "_skd_unit" && { _skd_unit isEqualType "" }) then {
    private _skd_targets = ["skd_fnc_place", 1, _skd_unit, false] call skd_fnc_targets;
    _skd_unit = if (isNil "_skd_targets") then { nil } else { player };
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

// In singleplayer the unit is local here, and this machine is the server.
_skd_unit setDir _skd_direction;
_skd_unit setPosASL _skd_position;
_skd_unit setVelocity [0, 0, 0];
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
