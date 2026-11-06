// @check US-029 An expected error that never comes fails the check at its end, quoting the text
// @tags US-029 ERR-04 DM-03 acceptance

["expect an error that never comes", {
    ["_skdfx_undefined", { 1 }] call skd_fnc_expectError;
}] call skd_fnc_step;

["the rest of the check passes", {
    [1, "==", 1, "one is one"] call skd_fnc_expect;
}] call skd_fnc_step;
