// @check US-016 US-043 p2 crashes in the fixture's extension while the check waits on it
// @tags US-016 US-043 acceptance
// @setups ded
// @timeout 120

// fixture-mod.md: a crash with no delay followed by a wait puts the lost machine first.
["p2", { [0] call skdfx_fnc_crash }] call skd_fnc_on;
["wait on p2 for a condition that never holds", {
    ["p2", { false }, "a condition that never holds", 60] call skd_fnc_waitUntil;
}] call skd_fnc_step;
