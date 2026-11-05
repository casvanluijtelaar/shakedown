// @check US-039 A step that runs for 20 s ends at the check's timeout of 5 s
// @tags US-039 acceptance
// @timeout 5

["slow part", {
    private _start = diag_tickTime;
    while {diag_tickTime - _start < 20} do {
        ["p1", {false}, "a condition that never holds", 1] call skd_fnc_tryWaitUntil;
    };
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
