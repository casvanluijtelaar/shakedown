// @check OBS-01 A wait that reaches its ceiling ends the check with the description
// @tags OBS-01 acceptance

["wait for a door", {
    ["p1", { false }, "the door that never opens is open", 1] call skd_fnc_waitUntil;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
