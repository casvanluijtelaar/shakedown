// @check US-077 a check that waits 8 s on p2 for a condition that then holds, repeated while p2 is ended
// @tags US-077 acceptance
// @setups ded
// @timeout 60

["p2 holds for 8 s", {
    private _until = ["p2", { diag_tickTime + 8 }] call skd_fnc_on;
    ["p2", { diag_tickTime > (_this select 0) }, "8 s passed on p2", 30, [_until]] call skd_fnc_waitUntil;
}] call skd_fnc_step;
