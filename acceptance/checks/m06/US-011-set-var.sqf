// @check US-011 a check sets a public missionNamespace variable, which the reset in place leaves
// @tags US-011 RST-06 acceptance

["set acc_m06_flag to 7, public, on the server", {
    ["server", { missionNamespace setVariable ["acc_m06_flag", 7, true]; true }] call skd_fnc_on;
    ["all", { missionNamespace getVariable ["acc_m06_flag", -1] }, "==", 7, "acc_m06_flag on every machine", 10] call skd_fnc_expectEach;
}] call skd_fnc_step;
