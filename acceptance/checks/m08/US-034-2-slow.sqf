// @check US-034 the second of three checks, which waits 45 s for a condition that never holds, so a stop comes during it
// @tags US-034 acceptance
// @setups sp
// @timeout 60

["wait 45 s for a condition that never holds", {
    ["p1", { false }, "a condition that never holds", 45] call skd_fnc_waitUntil
}] call skd_fnc_step;
