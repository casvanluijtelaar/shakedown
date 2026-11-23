// @check US-009 A check kills p1
// @tags US-009 RST-05 acceptance

private _p1 = ["p1"] call skd_fnc_player;

["kill p1", {
    ["p1", { player setDamage 1; true }] call skd_fnc_on;
    ["p1", { !alive (_this select 0) }, "p1's unit is dead", 10, [_this select 0]] call skd_fnc_waitUntil;
}, [_p1]] call skd_fnc_step;
