// @check US-069 ["p9"] call skd_fnc_join ends the check fail check-error naming the machines that can join and leave
// @tags US-069 acceptance
// @setups ded
// @timeout 60

["p9"] call skd_fnc_join;

["the step after the join", {
    [1, "==", 1, "one"] call skd_fnc_expect;
}] call skd_fnc_step;
