// @check SCN-02 skd_fnc_place puts p1's player and a created unit where it says, facing the direction
// @tags SCN-02 acceptance

["place p1 by its machine name", {
    private _unit = ["p1"] call skd_fnc_player;
    private _pos = AGLToASL [140, 140, 0];
    private _done = ["p1", _pos, 270] call skd_fnc_place;
    [_done, "==", true, "the value skd_fnc_place returned"] call skd_fnc_expect;
    ["server", { getPosASL (_this select 0) }, "~=", [_pos, 0.1], "where the server sees p1", 10, [_unit]] call skd_fnc_expectOn;
    ["server", { getDir (_this select 0) }, "~=", [270, 3], "where the server sees p1 face", 10, [_unit]] call skd_fnc_expectOn;
}] call skd_fnc_step;

["place a created unit by the object", {
    private _unit = ["B_Soldier_F", AGLToASL [150, 150, 0], 0] call skd_fnc_create;
    private _pos = AGLToASL [155, 150, 0];
    private _done = [_unit, _pos, 135] call skd_fnc_place;
    [_done, "==", true, "the value skd_fnc_place returned"] call skd_fnc_expect;
    [getPosASL _unit, "~=", [_pos, 0.1], "where the unit stands"] call skd_fnc_expect;
    [getDir _unit, "~=", [135, 3], "where the unit faces"] call skd_fnc_expect;
}] call skd_fnc_step;
