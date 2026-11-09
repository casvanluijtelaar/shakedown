// @check US-011 a check with reset mission after it sees acc_m06_flag nil on every machine
// @tags US-011 RST-06 RST-02 API-01 acceptance
// @reset mission

["every machine starts the mission with no acc_m06_flag", {
    ["all", { isNil "acc_m06_flag" }, "==", true, "acc_m06_flag is nil", 10] call skd_fnc_expectEach;
}] call skd_fnc_step;
