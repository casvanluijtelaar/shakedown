// @check US-067 p2 leaves and the check ends with p2 out of the session
// @tags US-067 acceptance
// @setups ded
// @timeout 120

["p2"] call skd_fnc_leave;

["p2 is out", {
    [[] call skd_fnc_machines, "==", ["server", "p1", "hc1"], "skd_fnc_machines after the leave"] call skd_fnc_expect;
}] call skd_fnc_step;
