// @check OBS-06 A polled assertion on one machine that never holds reports the last value read
// @tags OBS-06 acceptance

["expect seven to be eight", {
    ["p1", { 7 }, "==", 8, "seven is eight on p1", 0.5] call skd_fnc_expectOn;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
