/*
    skd_fnc_create [class, position, direction] - creates an object, a vehicle or a unit of a
    CfgVehicles class on the server at a position (ASL) and direction (check API, "Building a
    scene"; SCN-01). A kind of CAManBase is a unit in a new group of its config's side. The entity
    recorder notes it for the next reset (SCN-04).

    Returns: the object.
*/
if !(["skd_fnc_create"] call skd_fnc_guard) exitWith {};
params ["_skd_class", "_skd_position", "_skd_direction"];
if (isNil "_skd_class" || { !(_skd_class isEqualType "") }) exitWith {
    ["skd_fnc_create", 1, "class", "a CfgVehicles class name (String)", if (isNil "_skd_class") then { nil } else { _skd_class }] call skd_fnc_argError;
};
if (isNil "_skd_position" || { !(_skd_position isEqualTypeArray [0, 0, 0]) }) exitWith {
    ["skd_fnc_create", 2, "position", "an ASL position [x, y, z] of three numbers", if (isNil "_skd_position") then { nil } else { _skd_position }] call skd_fnc_argError;
};
if (isNil "_skd_direction") then { _skd_direction = 0 };
if !(_skd_direction isEqualType 0) exitWith {
    ["skd_fnc_create", 3, "direction", "a number of degrees", _skd_direction] call skd_fnc_argError;
};
private _skd_config = configFile >> "CfgVehicles" >> _skd_class;
if (!isClass _skd_config || { getNumber (_skd_config >> "scope") < 1 }) exitWith {
    ["fail", "check-error", format ["skd_fnc_create: the class %1 is not in CfgVehicles (or is not creatable). Check its spelling and that its mod is in the setup.", str _skd_class]] call skd_fnc_end;
};
// In singleplayer the server is this machine.
private _skd_object = objNull;
if (_skd_class isKindOf "CAManBase") then {
    private _skd_side = [east, west, independent, civilian] param [getNumber (_skd_config >> "side"), civilian];
    private _skd_group = createGroup [_skd_side, true];
    _skd_object = _skd_group createUnit [_skd_class, [0, 0, 0], [], 0, "CAN_COLLIDE"];
} else {
    _skd_object = createVehicle [_skd_class, [0, 0, 0], [], 0, "CAN_COLLIDE"];
};
_skd_object setDir _skd_direction;
_skd_object setPosASL _skd_position;
_skd_object
