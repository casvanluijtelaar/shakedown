// @check US-044 a check creates two boxes near p1 and fails, leaving them for inspect
// @tags US-044 acceptance
// @setups ded

private _at = ["p1", { getPosASL player }] call skd_fnc_on;
private _a = ["Land_WoodenBox_F", _at vectorAdd [6, 0, 0], 0] call skd_fnc_create;
private _b = ["Box_NATO_Ammo_F", _at vectorAdd [0, 9, 0], 90] call skd_fnc_create;
["created", [netId _a, netId _b]] call skd_fnc_evidence;
["the boxes stay for inspect"] call skd_fnc_fail;
