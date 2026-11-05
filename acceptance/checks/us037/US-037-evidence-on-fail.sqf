// @check US-037 Evidence attached by a failing check is kept in the result
// @tags US-037 acceptance

["attach two values", {
    ["number", 7] call skd_fnc_evidence;
    ["text", "kept on failure"] call skd_fnc_evidence;
}] call skd_fnc_step;

["fail on purpose", {
    ["deliberate failure for US-037"] call skd_fnc_fail;
}] call skd_fnc_step;

["never reached", {
    ["after the failure", true] call skd_fnc_evidence;
}] call skd_fnc_step;
