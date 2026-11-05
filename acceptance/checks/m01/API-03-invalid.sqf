// @check API-03 skd_fnc_invalid ends the check invalid with its message
// @tags API-03 acceptance

["detect a broken environment", {
    ["the environment is broken on purpose"] call skd_fnc_invalid;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
