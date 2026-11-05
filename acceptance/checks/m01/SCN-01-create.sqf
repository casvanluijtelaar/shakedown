// @check SCN-01 skd_fnc_create makes an object, a vehicle and a unit at a position and direction
// @tags SCN-01 acceptance

["an object", {
    private _pos = AGLToASL [116, 116, 0];
    private _box = ["Land_WoodenBox_F", _pos, 45] call skd_fnc_create;
    [typeOf _box, "==", "Land_WoodenBox_F", "the box's class"] call skd_fnc_expect;
    [(getPosASL _box) select [0, 2], "~=", [_pos select [0, 2], 0.1], "the box's position"] call skd_fnc_expect;
    [getDir _box, "~=", [45, 1], "the box's direction"] call skd_fnc_expect;
}] call skd_fnc_step;

["a vehicle with no direction given", {
    private _pos = AGLToASL [125, 116, 0];
    private _quad = ["B_Quadbike_01_F", _pos] call skd_fnc_create;
    [typeOf _quad, "==", "B_Quadbike_01_F", "the vehicle's class"] call skd_fnc_expect;
    [(getPosASL _quad) select [0, 2], "~=", [_pos select [0, 2], 0.1], "the vehicle's position"] call skd_fnc_expect;
    private _dir = getDir _quad;
    [_dir min (360 - _dir), "<", 1, "the vehicle's direction from north"] call skd_fnc_expect;
}] call skd_fnc_step;

["a unit in a new group of its side", {
    private _pos = AGLToASL [116, 125, 0];
    private _unit = ["O_Soldier_F", _pos, 90] call skd_fnc_create;
    [_unit isKindOf "CAManBase", "==", true, "the soldier is a man"] call skd_fnc_expect;
    [str side group _unit, "==", str east, "the side of the soldier's group"] call skd_fnc_expect;
    [units group _unit, "==", [_unit], "the soldier's group holds only the soldier"] call skd_fnc_expect;
    [group _unit != group player, "==", true, "the soldier is not in the player's group"] call skd_fnc_expect;
    [getDir _unit, "~=", [90, 1], "the soldier's direction"] call skd_fnc_expect;
}] call skd_fnc_step;
