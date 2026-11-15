// @check US-068 p3 joins with a ceiling of 30 s while the harness suspends or ends its process
// @tags US-068 acceptance
// @setups ded
// @timeout 180

["p3", 30] call skd_fnc_join;

["the step after the join", {
    [1, "==", 1, "one"] call skd_fnc_expect;
}] call skd_fnc_step;
