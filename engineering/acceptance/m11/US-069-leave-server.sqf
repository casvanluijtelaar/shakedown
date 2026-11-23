// @check US-069 ["server"] call skd_fnc_leave ends the check fail check-error naming the machines that can join and leave
// @tags US-069 acceptance
// @setups ded
// @timeout 60

["server"] call skd_fnc_leave;

["the step after the leave", {
    [1, "==", 1, "one"] call skd_fnc_expect;
}] call skd_fnc_step;
