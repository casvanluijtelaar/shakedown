// @check US-054 a third short check of the finding: p1 stands at the start, alive, on foot and unhurt
// @tags m07-demo US-054 acceptance
// @setups rld7 rld7sp

["p1 is alive, on foot and unhurt", {
    private _u = ["p1"] call skd_fnc_player;
    [alive _u, "==", true, "p1 is alive"] call skd_fnc_expect;
    [vehicle _u == _u, "==", true, "p1 is on foot"] call skd_fnc_expect;
    [damage _u, "==", 0, "the damage of p1"] call skd_fnc_expect;
}] call skd_fnc_step;
