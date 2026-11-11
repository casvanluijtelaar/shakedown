// @check US-054 skdfx_fnc_version returns "1" on every machine after the staged file is put back and reloaded
// @tags US-054 FIX-03 acceptance
// @setups rld7 rld7sp

["every machine returns '1' from skdfx_fnc_version", {
    ["all", { call skdfx_fnc_version }, "==", "1", "skdfx_fnc_version", 10] call skd_fnc_expectEach;
}] call skd_fnc_step;
