// @check US-083 RST-01 the check after a hand over starts from the baseline: no crate, p1 not where the hand over left it
// @tags US-083 acceptance
// @setups ded

["no crate is left", {
    ["server", { count (allMissionObjects "skdfx_Crate") }, "==", 0, "skdfx_Crate objects on the server"] call skd_fnc_expectOn;
}] call skd_fnc_step;
["p1 is back at its start", {
    ["p1", { (getPosASL player) distance2D [150, 150, 0] }, ">", 2, "p1's distance from where the hand over put it"] call skd_fnc_expectOn;
    ["p1", { (getPosASL player) distance2D [200, 200, 0] }, ">", 2, "p1's distance from where the refused hand over put it"] call skd_fnc_expectOn;
}] call skd_fnc_step;
