// @check US-039 A wait whose ceiling reaches past the timeout is cut to the time left
// @tags US-039 acceptance
// @timeout 3

["wait past the timeout", {
    ["p1", {false}, "a condition that never holds", 30] call skd_fnc_waitUntil;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
