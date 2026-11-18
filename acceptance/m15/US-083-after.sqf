// @check US-083 MAN-01 a check selected after the one that hands over, which the run does not reach
// @tags US-083 acceptance
// @setups ded sp

["p1 has a player", {
    ["p1", { !isNull player }, "==", true, "p1 has a player"] call skd_fnc_expectOn;
}] call skd_fnc_step;
