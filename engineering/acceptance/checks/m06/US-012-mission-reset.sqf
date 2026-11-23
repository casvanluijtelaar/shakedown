// @check US-012 a check with reset mission whose mission restart loses p2
// @tags US-012 RST-06 P-05 acceptance
// @reset mission

["the mission restarted", {
    [true, "==", true, "the check body runs"] call skd_fnc_expect;
}] call skd_fnc_step;
