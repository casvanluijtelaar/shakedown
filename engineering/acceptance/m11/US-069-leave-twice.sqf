// @check US-069 a second ["p2"] call skd_fnc_leave after p2 left ends the check fail check-error naming the machines that can join and leave
// @tags US-069 acceptance
// @setups ded
// @timeout 60

["p2"] call skd_fnc_leave;
["p2"] call skd_fnc_leave;

["the step after the second leave", {
    [1, "==", 1, "one"] call skd_fnc_expect;
}] call skd_fnc_step;
