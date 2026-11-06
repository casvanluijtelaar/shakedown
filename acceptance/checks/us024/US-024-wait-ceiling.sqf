// @check US-024 A wait for the flag on p1 that never holds ends at its ceiling of 3 s
// @tags US-024 OBS-04 DM-07 acceptance

["wait for the flag", {
    ["p1", { (missionNamespace getVariable ["skdfx_flag", 0]) isEqualTo "never" }, "flag reached p1", 3] call skd_fnc_waitUntil;
}] call skd_fnc_step;

["never runs", {}] call skd_fnc_step;
