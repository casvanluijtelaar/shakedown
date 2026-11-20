// @check US-086 ENV-01 a check run in a session whose start took the fixture's version
// @tags US-086 acceptance
// @setups sp ded

["p1 has a player", {
    ["p1", { !isNull player }, "==", true, "p1 has a player"] call skd_fnc_expectOn;
}] call skd_fnc_step;
