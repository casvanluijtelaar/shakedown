// @check US-027 An error expected from skdfx_fnc_throw on p2 passes the check and is listed as expected
// @tags US-027 ERR-04 ERR-01 acceptance

["p2 rejects the bad call", {
    ["_skdfx_undefined", { ["p2", {[] call skdfx_fnc_throw}] call skd_fnc_on }] call skd_fnc_expectError;
}] call skd_fnc_step;
