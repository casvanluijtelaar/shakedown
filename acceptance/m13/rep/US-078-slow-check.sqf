// @check US-078 a check of timeout 20 that takes 11 s
// @tags US-078 acceptance
// @setups sp
// @timeout 20

["the step that waits 11 s", {
    private _until = ["p1", { diag_tickTime + 11 }] call skd_fnc_on;
    ["p1", { diag_tickTime > (_this select 0) }, "11 s passed on p1", 18, [_until]] call skd_fnc_waitUntil;
}] call skd_fnc_step;
