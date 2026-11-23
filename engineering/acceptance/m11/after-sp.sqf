// @check US-069 the check after a wrong leave in singleplayer passes with p1 alone
// @tags US-069 acceptance
// @setups sp

["p1 is the session", {
    [[] call skd_fnc_machines, "==", ["p1"], "skd_fnc_machines"] call skd_fnc_expect;
}] call skd_fnc_step;
