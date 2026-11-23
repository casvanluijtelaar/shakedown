// @check US-076 a check that always passes, repeated
// @tags US-076 acceptance
// @setups sp

["p1 has a player", {
    ["p1", { !isNull player }, "==", true, "p1 has a player"] call skd_fnc_expectOn;
}] call skd_fnc_step;
