// @check US-041 DM-06 an assertion fails at its 2 s ceiling and p2, crashed 1 s in, is lost just after it, inside the check
// @tags US-041 acceptance
// @setups ded

// The crash ends p2 about 1.3 s after its fault (fixture-mod.md: within 3 s), so a fault 1 s in ends p2
// just after the assertion fails at 2 s, while the execution waits for errors in flight.
["p2", { [1] call skdfx_fnc_crash }] call skd_fnc_on;
["an assertion that fails before p2 is lost", {
    ["server", { false }, "==", true, "a value that never holds", 2] call skd_fnc_expectOn;
}] call skd_fnc_step;
