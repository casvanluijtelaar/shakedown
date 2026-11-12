// @check US-034 the third of three checks, which must not start once the run is stopped
// @tags US-034 acceptance
// @setups sp
// @timeout 30

["p1 is there", {
    [["p1"] call skd_fnc_player, "!=", objNull, "the unit of p1"] call skd_fnc_expect;
}] call skd_fnc_step;
