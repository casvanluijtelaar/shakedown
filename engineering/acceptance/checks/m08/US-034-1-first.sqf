// @check US-034 the first of three checks, which passes before the run is stopped
// @tags US-034 acceptance
// @setups sp
// @timeout 30

["p1 is there", {
    [["p1"] call skd_fnc_player, "!=", objNull, "the unit of p1"] call skd_fnc_expect;
}] call skd_fnc_step;
