// @check US-071 a check that calls no library helper
// @tags US-071 acceptance
// @setups ded sp

["a plain step", {
    [1 + 1, "==", 2, "one and one"] call skd_fnc_expect;
}] call skd_fnc_step;
