// @check US-066 US-067 US-068 US-069 the check after a join or a leave starts with server p1 p2 hc1 and no late client
// @tags US-066 US-067 US-068 US-069 acceptance
// @setups ded

["the session's machines are the ones it started with", {
    private _machines = [] call skd_fnc_machines;
    ["machines", _machines] call skd_fnc_evidence;
    [_machines, "==", ["server", "p1", "p2", "hc1"], "skd_fnc_machines"] call skd_fnc_expect;
}] call skd_fnc_step;

["every client's player is alive", {
    ["clients", { alive player }, "==", true, "the player is alive", 10] call skd_fnc_expectEach;
}] call skd_fnc_step;
