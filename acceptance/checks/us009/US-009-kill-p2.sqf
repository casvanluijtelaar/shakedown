// @check US-009 A check on a dedicated session kills p2 after recording where the server sees it start
// @tags US-009 RST-05 acceptance

private _p2 = ["p2"] call skd_fnc_player;
acc_m02_p2start = getPosASL _p2;

["kill p2", {
    ["p2", { player setDamage 1; true }] call skd_fnc_on;
}] call skd_fnc_step;

["the server sees p2's unit dead", {
    ["server", { !alive (_this select 0) }, "p2's unit is dead", 10, [_this select 0]] call skd_fnc_waitUntil;
}, [_p2]] call skd_fnc_step;
