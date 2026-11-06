// @check {NAME}
// @tags US-008 NFR-RL-02 RST-02 RST-05 SCN-04 acceptance

/* The harness writes copies 01 to 20 of this file, which check the start state and then change
   it, and a copy 21 that only checks. The first copy records each machine's start state in
   acc_m02_base on the server, which the reset leaves alone; every copy that changes the scene
   records the net ids of what it made, on any machine, in acc_m02_made for the next one. */

private _change = {CHANGE};
private _players = [["p1"] call skd_fnc_player, ["p2"] call skd_fnc_player];

// Sent to every machine: the date, the weather and what that machine sees of p1 and p2.
private _read = {
    [dateToNumber date, date select 0, overcast, fog, rain, _this apply {
        [alive _x, vehicle _x == _x, getPosASL _x, getDir _x, damage _x, getUnitLoadout _x]
    }]
};

private _now = ["all", _read, _players] call skd_fnc_onEach;
if (isNil "acc_m02_base") then { acc_m02_base = _now };
private _base = acc_m02_base;
private _made = missionNamespace getVariable ["acc_m02_made", []];
["start state", _now] call skd_fnc_evidence;
["made by the check before", _made] call skd_fnc_evidence;

["nothing the check before made exists on any machine", {
    params ["_made"];
    ["all", { { !isNull objectFromNetId _x } count (_this select 0) }, "==", 0,
        "objects of the check before that still exist", 1, [_made]] call skd_fnc_expectEach;
    [count allDeadMen, "==", 0, "dead men the server sees"] call skd_fnc_expect;
}, [_made]] call skd_fnc_step;

["every machine has the start date and weather", {
    params ["_now", "_base"];
    {
        private _n = _now get _x;
        private _b = _base get _x;
        [_n select 1, "==", _b select 1, format ["the year on %1", _x]] call skd_fnc_expect;
        [((_n select 0) - (_b select 0)) * 525600, "~=", [0, 1], format ["minutes the date on %1 is off the start date", _x]] call skd_fnc_expect;
        [_n select 2, "~=", [_b select 2, 0.01], format ["the overcast on %1", _x]] call skd_fnc_expect;
        [_n select 3, "~=", [_b select 3, 0.01], format ["the fog on %1", _x]] call skd_fnc_expect;
        [_n select 4, "~=", [_b select 4, 0.01], format ["the rain on %1", _x]] call skd_fnc_expect;
    } forEach ["server", "p1", "p2", "hc1"];
}, [_now, _base]] call skd_fnc_step;

["p1 and p2 stand at their start, unhurt, on foot, with their start loadout, as the server and their own client see them", {
    params ["_now", "_base"];
    {
        _x params ["_machine", "_i", "_who"];
        private _n = ((_now get _machine) select 5) select _i;
        private _b = ((_base get _machine) select 5) select _i;
        private _on = format ["%1 as %2 sees it", _who, _machine];
        [_n select 0, "==", true, format ["%1 is alive", _on]] call skd_fnc_expect;
        [_n select 1, "==", true, format ["%1 is on foot", _on]] call skd_fnc_expect;
        [_n select 2, "~=", [_b select 2, 0.1], format ["the position of %1", _on]] call skd_fnc_expect;
        private _turn = abs ((_n select 3) - (_b select 3));
        [_turn min (360 - _turn), "<=", 3, format ["degrees %1 is turned from its start", _on]] call skd_fnc_expect;
        [_n select 4, "==", 0, format ["the damage of %1", _on]] call skd_fnc_expect;
        [_n select 5, "==", _b select 5, format ["the loadout of %1", _on]] call skd_fnc_expect;
    } forEach [["server", 0, "p1"], ["server", 1, "p2"], ["p1", 0, "p1"], ["p2", 1, "p2"]];
}, [_now, _base]] call skd_fnc_step;

if (!_change) exitWith {};

private _mrap = ["create a unit, a vehicle and an object through the API, and objects with SQF on the server, a client and the headless client", {
    private _unit = ["B_Soldier_F", AGLToASL [160, 160, 0], 10] call skd_fnc_create;
    private _box = ["Land_WoodenBox_F", AGLToASL [163, 160, 0], 20] call skd_fnc_create;
    private _mrap = ["B_MRAP_01_F", AGLToASL [170, 170, 0], 30] call skd_fnc_create;
    private _own = createVehicle ["Land_WoodenBox_F", [166, 160, 0], [], 0, "CAN_COLLIDE"];
    private _quad = ["p2", { netId createVehicle ["B_Quadbike_01_F", [185, 170, 0], [], 0, "CAN_COLLIDE"] }] call skd_fnc_on;
    private _hcBox = ["hc1", { netId createVehicle ["Land_WoodenBox_F", [190, 160, 0], [], 0, "CAN_COLLIDE"] }] call skd_fnc_on;
    acc_m02_made = [netId _unit, netId _box, netId _mrap, netId _own, _quad, _hcBox];
    _mrap
}] call skd_fnc_step;

["change the date and the weather", {
    [[2031, 1, 2, 3, 45]] call skd_fnc_setDate;
    [1, 0.5, 1] call skd_fnc_setWeather;
}] call skd_fnc_step;

["move, hurt and strip p1 and put it in the MRAP; hurt and move p2, or kill it every fifth check", {
    params ["_mrap"];
    ["p1", AGLToASL [150, 170, 0], 77] call skd_fnc_place;
    ["p1", {
        removeAllWeapons player;
        removeHeadgear player;
        player setDamage 0.4;
        player moveInDriver (_this select 0);
        true
    }, [_mrap]] call skd_fnc_on;
    ["p1", { vehicle player == (_this select 0) }, "p1 sits in the MRAP", 10, [_mrap]] call skd_fnc_waitUntil;
    acc_m02_rl_count = (missionNamespace getVariable ["acc_m02_rl_count", 0]) + 1;
    if (acc_m02_rl_count % 5 == 0) then {
        private _old = ["p2"] call skd_fnc_player;
        ["p2", { player setDamage 1; true }] call skd_fnc_on;
        ["server", { !alive (_this select 0) }, "the server sees p2's unit dead", 10, [_old]] call skd_fnc_waitUntil;
    } else {
        ["p2", AGLToASL [140, 180, 0], 200] call skd_fnc_place;
        ["p2", { removeAllItems player; player setDamage 0.3; true }] call skd_fnc_on;
    };
}, [_mrap]] call skd_fnc_step;
