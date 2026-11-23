// @check OBS-01 A wait whose condition returns a number is the author's to fix
// @tags OBS-01 acceptance

["wait on a number", {
    ["p1", { 5 }, "a condition that returns a number", 2] call skd_fnc_waitUntil;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
