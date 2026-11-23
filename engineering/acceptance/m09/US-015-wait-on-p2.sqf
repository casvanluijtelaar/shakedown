// @check US-015 US-016 a check waits 60 s on p2 for a condition that never holds
// @tags US-015 US-016 acceptance
// @setups ded
// @timeout 120

["wait on p2 for a condition that never holds", {
    ["p2", { false }, "a condition that never holds", 60] call skd_fnc_waitUntil;
}] call skd_fnc_step;
