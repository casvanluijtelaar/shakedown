// @check US-067 p2 leaves, the fixture's PlayerDisconnected handler records it on the server, and p2 joins again with a new unit
// @tags US-067 acceptance
// @setups ded
// @timeout 300

["server", { missionNamespace setVariable ["skd_m11_go", false]; true }] call skd_fnc_on;
private _old = ["p2"] call skd_fnc_player;
private _oldId = netId _old;
["p2's unit before the leave", _oldId] call skd_fnc_evidence;
private _before = ["server", { { _x == "p2" } count (missionNamespace getVariable ["skdfx_disconnected", []]) }] call skd_fnc_on;
["p2 in skdfx_disconnected before the leave", _before] call skd_fnc_evidence;

["p2"] call skd_fnc_leave;

["the server recorded p2's disconnect", {
    ["server", { { _x == "p2" } count (missionNamespace getVariable ["skdfx_disconnected", []]) }, "==", _before + 1,
        "the times skdfx_disconnected holds p2", 10] call skd_fnc_expectOn;
    private _machines = [] call skd_fnc_machines;
    ["machines after the leave", _machines] call skd_fnc_evidence;
    [_machines, "==", ["server", "p1", "hc1"], "skd_fnc_machines after the leave"] call skd_fnc_expect;
}] call skd_fnc_step;

// The harness sets skd_m11_go on the server once status shows no p2 and p2's process has ended.
["wait for the harness to see p2 gone", {
    ["server", { missionNamespace getVariable ["skd_m11_go", false] }, "the harness saw p2 gone", 60] call skd_fnc_waitUntil;
}] call skd_fnc_step;

["p2"] call skd_fnc_join;

["p2 has a new unit", {
    private _new = ["p2"] call skd_fnc_player;
    ["p2's unit after the join", netId _new] call skd_fnc_evidence;
    [isNull _new, "==", false, "skd_fnc_player p2 is a unit"] call skd_fnc_expect;
    [netId _new, "!=", _oldId, "p2's unit is not the one it had before the leave"] call skd_fnc_expect;
    ["p2", { netId player }, "==", netId _new, "p2's own player is the unit skd_fnc_player gives", 10] call skd_fnc_expectOn;
    ["p2", { alive player }, "==", true, "p2's player is alive", 10] call skd_fnc_expectOn;
    private _machines = [] call skd_fnc_machines;
    [_machines, "==", ["server", "p1", "p2", "hc1"], "skd_fnc_machines after the join"] call skd_fnc_expect;
}] call skd_fnc_step;
