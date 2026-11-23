// @check US-027 The expected text matches the error case-insensitively
// @tags US-027 ERR-04 acceptance

["p2 rejects the bad call", {
    ["_SKDFX_UNDEFINED", { ["p2", {[] call skdfx_fnc_throw}] call skd_fnc_on }] call skd_fnc_expectError;
}] call skd_fnc_step;
