// @check US-066 a late p3 joins after a crate with a public variable is up and agrees with server p1 and p2 on it
// @tags US-066 acceptance
// @setups ded
// @timeout 300

["no late client before the join", {
    private _machines = [] call skd_fnc_machines;
    ["machines before the join", _machines] call skd_fnc_evidence;
    [_machines, "==", ["server", "p1", "p2", "hc1"], "skd_fnc_machines before the join"] call skd_fnc_expect;
}] call skd_fnc_step;

private _crate = ["a crate with a public variable on the server", {
    private _c = ["skdfx_Crate", AGLToASL [110, 105, 0], 30] call skd_fnc_create;
    ["server", { (_this select 0) setVariable ["skd_m11_flag", 7, true]; true }, [_c]] call skd_fnc_on;
    _c
}] call skd_fnc_step;

["p3"] call skd_fnc_join;

["p3 is in the session", {
    private _machines = [] call skd_fnc_machines;
    ["machines after the join", _machines] call skd_fnc_evidence;
    [_machines, "==", ["server", "p1", "p2", "p3", "hc1"], "skd_fnc_machines after the join"] call skd_fnc_expect;
}] call skd_fnc_step;

["every machine agrees on the crate", {
    [["server", "p1", "p2", "p3"], { getPosASL (_this select 0) }, "the crate's position", 0.1, 30, [_crate]] call skd_fnc_expectAgree;
    [["server", "p1", "p2", "p3"], { (_this select 0) getVariable ["skd_m11_flag", -1] }, "the crate's variable", 0, 30, [_crate]] call skd_fnc_expectAgree;
    ["p3", { (_this select 0) getVariable ["skd_m11_flag", -1] }, "==", 7, "the crate's variable on p3", 10, [_crate]] call skd_fnc_expectOn;
}] call skd_fnc_step;
