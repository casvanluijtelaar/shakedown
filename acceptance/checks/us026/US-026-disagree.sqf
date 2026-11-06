// @check US-026 p2 holding 6 where the server and p1 hold 5 fails the agreement naming p2
// @tags US-026 OBS-06 OBS-04 RES-03 acceptance

["set 5 on the server and wait until both clients see it", {
    [5] call skdfx_fnc_setFlag;
    ["clients", { missionNamespace getVariable "skdfx_flag" }, "==", 5, "the clients see 5"] call skd_fnc_expectEach;
}] call skd_fnc_step;

["set 6 on p2 alone", {
    ["p2", { missionNamespace setVariable ["skdfx_flag", 6]; true }] call skd_fnc_on;
}] call skd_fnc_step;

["server, p1 and p2 agree", {
    [["server", "p1", "p2"], { missionNamespace getVariable "skdfx_flag" }, "flag agrees", 0, 2] call skd_fnc_expectAgree;
}] call skd_fnc_step;

["never runs", {}] call skd_fnc_step;
