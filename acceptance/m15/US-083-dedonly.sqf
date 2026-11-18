// @check US-083 RUN-07 a check that names only the second setup, which a hand over in the first keeps from starting
// @tags US-083 acceptance
// @setups ded

["p1 has a player", {
    ["p1", { !isNull player }, "==", true, "p1 has a player"] call skd_fnc_expectOn;
}] call skd_fnc_step;
