// @check US-018 A ceiling given as a string to skd_fnc_waitUntil is the author's to fix
// @tags US-018 acceptance

["wait for p1", {
    ["p1", {true}, "ready", "10"] call skd_fnc_waitUntil;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
