// @check US-009 The check after p2 was killed sees p2 alive at its start position on the server
// @tags US-009 RST-05 acceptance

private _p2 = ["p2"] call skd_fnc_player;
private _start = missionNamespace getVariable ["acc_m02_p2start", []];

["the check before recorded where p2 started", {
    [count (_this select 0), "==", 3, "the start position US-009-kill-p2 recorded"] call skd_fnc_expect;
}, [_start]] call skd_fnc_step;

["the server sees p2 alive, unhurt and on foot at its start position", {
    params ["_p2", "_start"];
    [isNull _p2, "==", false, "p2 has a unit"] call skd_fnc_expect;
    [alive _p2, "==", true, "the server sees p2 alive"] call skd_fnc_expect;
    [damage _p2, "==", 0, "p2's damage"] call skd_fnc_expect;
    [vehicle _p2 == _p2, "==", true, "p2 is on foot"] call skd_fnc_expect;
    [getPosASL _p2, "~=", [_start, 0.1], "where the server sees p2"] call skd_fnc_expect;
    [count allDeadMen, "==", 0, "dead men the server sees"] call skd_fnc_expect;
}, [_p2, _start]] call skd_fnc_step;

["p2 itself is alive", {
    ["p2", { alive player }, "==", true, "p2 alive"] call skd_fnc_expectOn;
}] call skd_fnc_step;
