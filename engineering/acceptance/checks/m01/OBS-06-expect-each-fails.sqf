// @check OBS-06 A polled assertion on each machine that never holds reports every machine's value
// @tags OBS-06 acceptance

["expect one to be two", {
    ["all", { 1 }, "==", 2, "one is two on every machine", 0.5] call skd_fnc_expectEach;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
