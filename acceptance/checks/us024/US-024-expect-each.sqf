// @check US-024 An expectEach on all that never holds keeps every machine's last value, the failed step and a log per machine
// @tags US-024 OBS-04 RES-03 acceptance

["set 5 on the server and wait until every machine sees it", {
    [5] call skdfx_fnc_setFlag;
    ["all", { missionNamespace getVariable ["skdfx_flag", -1] }, "==", 5, "every machine sees 5"] call skd_fnc_expectEach;
}] call skd_fnc_step;

["every machine sees 99", {
    ["all", { missionNamespace getVariable ["skdfx_flag", -1] }, "==", 99, "the flag reads 99", 2] call skd_fnc_expectEach;
}] call skd_fnc_step;

["never runs", {}] call skd_fnc_step;
