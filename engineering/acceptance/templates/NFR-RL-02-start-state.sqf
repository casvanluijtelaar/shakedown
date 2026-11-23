// @check NFR-RL-02 Check {N} of 20 sees the start state, then changes units, vehicles, objects, time, weather and the player
// @tags NFR-RL-02 RST-01 RST-02 RST-05 SCN-04 acceptance

/* The harness writes 20 copies of this file, with {N} replaced by 01 to 20. The first check of a
   session records the start state in acc_m01_base, which the reset leaves alone; every later one
   compares against it. */

private _read = {
    private _u = ["p1"] call skd_fnc_player;
    createHashMapFromArray [
        ["date", dateToNumber date], ["year", date select 0],
        ["overcast", overcast], ["fog", fog], ["rain", rain],
        ["vehicles", count vehicles], ["units", count allUnits], ["groups", count allGroups],
        ["objects", count allMissionObjects "All"], ["dead", count allDeadMen],
        ["alive", alive _u], ["onFoot", vehicle _u == _u], ["pos", getPosASL _u], ["dir", getDir _u],
        ["damage", damage _u], ["loadout", getUnitLoadout _u], ["anim", animationState _u]
    ]
};

private _now = call _read;
if (isNil "acc_m01_base") then { acc_m01_base = _now };
private _base = acc_m01_base;
["start state", _now] call skd_fnc_evidence;

["the check sees the start state", {
    params ["_now", "_base"];
    [_now get "alive", "==", true, "p1 is alive"] call skd_fnc_expect;
    [_now get "onFoot", "==", true, "p1 is on foot"] call skd_fnc_expect;
    [_now get "damage", "==", 0, "p1's damage"] call skd_fnc_expect;
    [_now get "pos", "~=", [_base get "pos", 0.1], "p1's position"] call skd_fnc_expect;
    private _turn = abs ((_now get "dir") - (_base get "dir"));
    [_turn min (360 - _turn), "<=", 3, "degrees p1 is turned from its start direction"] call skd_fnc_expect;
    [_now get "loadout", "==", _base get "loadout", "p1's loadout"] call skd_fnc_expect;
    [_now get "anim", "==", _base get "anim", "p1's animation state"] call skd_fnc_expect;
    [_now get "year", "==", _base get "year", "the year"] call skd_fnc_expect;
    [((_now get "date") - (_base get "date")) * 525600, "~=", [0, 1], "minutes the date is off the start date"] call skd_fnc_expect;
    [_now get "overcast", "~=", [_base get "overcast", 0.01], "the overcast"] call skd_fnc_expect;
    [_now get "fog", "~=", [_base get "fog", 0.01], "the fog"] call skd_fnc_expect;
    [_now get "rain", "~=", [_base get "rain", 0.01], "the rain"] call skd_fnc_expect;
    {
        [_now get _x, "==", _base get _x, format ["the count of %1", _x]] call skd_fnc_expect;
    } forEach ["vehicles", "units", "groups", "objects", "dead"];
}, [_now, _base]] call skd_fnc_step;

private _mrap = ["create a unit, a vehicle and an object through the API", {
    ["B_Soldier_F", AGLToASL [160, 160, 0], 10] call skd_fnc_create;
    ["Land_WoodenBox_F", AGLToASL [163, 160, 0], 20] call skd_fnc_create;
    ["B_MRAP_01_F", AGLToASL [170, 170, 0], 30] call skd_fnc_create
}] call skd_fnc_step;

["create a unit, a vehicle and an object with the check's own SQF", {
    createVehicle ["Land_WoodenBox_F", [166, 160, 0], [], 0, "CAN_COLLIDE"];
    private _group = createGroup east;
    _group createUnit ["O_Soldier_F", [180, 160, 0], [], 0, "CAN_COLLIDE"];
    ["p1", { createVehicle ["B_Quadbike_01_F", [185, 170, 0], [], 0, "CAN_COLLIDE"]; true }] call skd_fnc_on;
}] call skd_fnc_step;

["change the date and the weather", {
    [[2031, 1, 2, 3, 45]] call skd_fnc_setDate;
    [1, 0.5, 1] call skd_fnc_setWeather;
}] call skd_fnc_step;

["change the player", {
    params ["_mrap"];
    ["p1", AGLToASL [150, 170, 0], 77] call skd_fnc_place;
    ["p1", {
        removeAllWeapons player;
        removeHeadgear player;
        player setDamage 0.4;
        player switchMove "AmovPpneMstpSnonWnonDnon";
        true
    }] call skd_fnc_on;
    acc_m01_rl_count = (missionNamespace getVariable ["acc_m01_rl_count", 0]) + 1;
    if (acc_m01_rl_count % 4 == 0) then {
        private _old = ["p1"] call skd_fnc_player;
        ["p1", { player setDamage 1; true }] call skd_fnc_on;
        ["p1", { alive player && { player != (_this select 0) } }, "p1 has a new living unit", 10, [_old]] call skd_fnc_waitUntil;
    } else {
        ["p1", { player moveInDriver (_this select 0); true }, [_mrap]] call skd_fnc_on;
        ["p1", { vehicle player == (_this select 0) }, "p1 sits in the MRAP", 10, [_mrap]] call skd_fnc_waitUntil;
    };
}, [_mrap]] call skd_fnc_step;

["the changes hold", {
    params ["_read", "_base"];
    private _after = call _read;
    [_after get "vehicles", ">", _base get "vehicles", "the count of vehicles after the changes"] call skd_fnc_expect;
    [_after get "units", ">", _base get "units", "the count of units after the changes"] call skd_fnc_expect;
    [_after get "year", "==", 2031, "the year after the change"] call skd_fnc_expect;
}, [_read, _base]] call skd_fnc_step;
