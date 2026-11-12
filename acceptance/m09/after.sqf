// @check US-041 a check that passes at once, run after the check whose machine or reset went wrong
// @tags US-041 US-010 US-015 US-016 US-059 acceptance

["one is one", {
    [1, "==", 1, "one"] call skd_fnc_expect;
}] call skd_fnc_step;
