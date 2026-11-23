// @check US-069 in singleplayer ["p1"] call skd_fnc_leave ends the check fail check-error saying joining and leaving needs a dedicated session
// @tags US-069 acceptance
// @setups sp
// @timeout 60

["p1"] call skd_fnc_leave;

["the step after the leave", {
    [1, "==", 1, "one"] call skd_fnc_expect;
}] call skd_fnc_step;
