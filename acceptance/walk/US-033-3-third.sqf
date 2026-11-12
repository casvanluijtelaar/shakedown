// @check US-033 the third check of the walk-away suite: p2 has a unit
// @tags US-033 acceptance
// @timeout 30

["p2 is there", {
    [["p2"] call skd_fnc_player, "!=", objNull, "the unit of p2"] call skd_fnc_expect;
}] call skd_fnc_step;
