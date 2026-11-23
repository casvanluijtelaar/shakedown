// @check US-078 a step waits 12 s on a condition that then holds, and the check passes
// @tags US-078 acceptance
// @setups sp
// @timeout 60

["the quick step", { true }] call skd_fnc_step;
["the step that waits 12 s", {
    private _until = ["p1", { diag_tickTime + 12 }] call skd_fnc_on;
    ["p1", { diag_tickTime > (_this select 0) }, "12 s passed on p1", 30, [_until]] call skd_fnc_waitUntil;
}] call skd_fnc_step;
