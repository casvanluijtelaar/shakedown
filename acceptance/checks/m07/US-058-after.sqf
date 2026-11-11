// @check US-058 the check after the slow one still sees skdfx_fnc_version "1" on every machine, the code the run started with
// @tags US-058 acceptance
// @setups rld7 rld7sp

["every machine returns '1' from skdfx_fnc_version", {
    ["all", { call skdfx_fnc_version }, "==", "1", "skdfx_fnc_version", 10] call skd_fnc_expectEach;
}] call skd_fnc_step;
