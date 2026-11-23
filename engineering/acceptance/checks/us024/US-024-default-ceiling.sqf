// @check US-024 NFR-PF-06 A wait given no ceiling ends at the default of 10 s
// @tags US-024 NFR-PF-06 acceptance

["wait with no ceiling", {
    ["p1", { false }, "a condition that never holds"] call skd_fnc_waitUntil;
}] call skd_fnc_step;
