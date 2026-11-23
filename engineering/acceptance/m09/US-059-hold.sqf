// @check US-059 p1's mission time is held for 8 s while the check waits 15 s of real time
// @tags US-059 acceptance
// @setups sp
// @timeout 60

private _t0 = ["p1", { diag_tickTime }] call skd_fnc_on;
private _held = ["hold the clock for 8 s", {
    ["p1", { [8] call skdfx_fnc_holdClock }] call skd_fnc_on
}] call skd_fnc_step;
["holdClock", _held] call skd_fnc_evidence;
["wait 15 s of real time", {
    ["p1", { diag_tickTime > (_this select 0) + 15 }, "15 s of real time passed", 30, [_this select 0]] call skd_fnc_waitUntil;
}, [_t0]] call skd_fnc_step;
