// @check RST-05 A player killed during a check is handed a new unit and the mission goes on
// @tags RST-05 acceptance

private _old = ["p1"] call skd_fnc_player;

["kill p1", {
    ["p1", { player setDamage 1; true }] call skd_fnc_on;
}] call skd_fnc_step;

["p1 controls a living unit again", {
    ["p1", { alive player && { player != (_this select 0) } }, "p1 has a new living unit", 10, [_this select 0]] call skd_fnc_waitUntil;
}, [_old]] call skd_fnc_step;
