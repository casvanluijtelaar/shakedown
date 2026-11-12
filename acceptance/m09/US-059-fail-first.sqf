// @check US-059 DM-06 an assertion fails at once and p1's clock is held 0.3 s later for 6 s
// @tags US-059 acceptance
// @setups sp

["p1", { [] spawn { uiSleep 0.3; [6] call skdfx_fnc_holdClock }; true }] call skd_fnc_on;
["an assertion that fails before the clock stops", {
    [1, "==", 2, "an assertion that fails at once"] call skd_fnc_expect;
}] call skd_fnc_step;
