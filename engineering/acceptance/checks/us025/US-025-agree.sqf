// @check US-025 the server and both clients agree on the fixture's flag, the clients each see 5, and positions agree within a tolerance
// @tags US-025 OBS-06 CTL-01 API-03 acceptance

["set the flag to 5 on the server", {
    [[5] call skdfx_fnc_setFlag, "==", true, "skdfx_fnc_setFlag on the server"] call skd_fnc_expect;
}] call skd_fnc_step;

["server, p1 and p2 agree", {
    private _got = [["server", "p1", "p2"], { missionNamespace getVariable "skdfx_flag" }, "flag agrees"] call skd_fnc_expectAgree;
    ["agreed", _got] call skd_fnc_evidence;
    [count _got, "==", 3, "the machines in the HashMap expectAgree returned"] call skd_fnc_expect;
    { [_got get _x, "==", 5, format ["the value expectAgree returned for %1", _x]] call skd_fnc_expect } forEach ["server", "p1", "p2"];
}] call skd_fnc_step;

["the clients each see 5", {
    private _got = ["clients", { missionNamespace getVariable "skdfx_flag" }, "==", 5, "clients see 5"] call skd_fnc_expectEach;
    [count _got, "==", 2, "the machines in the HashMap expectEach returned"] call skd_fnc_expect;
}] call skd_fnc_step;

["every machine agrees where p1 stands, within 0.5 m", {
    private _p1 = ["p1"] call skd_fnc_player;
    ["p1", AGLToASL [260, 200, 0], 0] call skd_fnc_place;
    ["all", { getPosASL (_this select 0) }, "p1's position", 0.5, 10, [_p1]] call skd_fnc_expectAgree;
}] call skd_fnc_step;

["a number within the tolerance agrees", {
    ["p2", { missionNamespace setVariable ["skdfx_flag", 5.3]; true }] call skd_fnc_on;
    [["server", "p2"], { missionNamespace getVariable "skdfx_flag" }, "flag agrees within 0.5", 0.5] call skd_fnc_expectAgree;
    [5] call skdfx_fnc_setFlag;
}] call skd_fnc_step;
