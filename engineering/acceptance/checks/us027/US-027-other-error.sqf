// @check US-027 An error of another text in the expectError block still fails the check
// @tags US-027 ERR-04 ERR-03 acceptance

["p2 throws and p1 adds a string to a number", {
    ["_skdfx_undefined", {
        ["p2", {[] call skdfx_fnc_throw}] call skd_fnc_on;
        ["p1", { "a" + 1 }] call skd_fnc_on;
    }] call skd_fnc_expectError;
}] call skd_fnc_step;
