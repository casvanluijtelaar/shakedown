// @check OBS-01 A wait returns once its condition holds and the try wait returns false at its ceiling
// @tags OBS-01 OBS-02 OBS-07 acceptance

["a condition that comes true after a second", {
    private _start = diag_tickTime;
    private _held = ["p1", { diag_tickTime > (_this select 0) }, "a second has passed", 5, [_start + 1]] call skd_fnc_waitUntil;
    [_held, "==", true, "the value skd_fnc_waitUntil returned"] call skd_fnc_expect;
    [diag_tickTime - _start, ">=", 1, "seconds the wait took"] call skd_fnc_expect;
    [diag_tickTime - _start, "<", 4, "seconds the wait took"] call skd_fnc_expect;
}] call skd_fnc_step;

["a try wait that holds", {
    private _held = ["p1", { true }, "always"] call skd_fnc_tryWaitUntil;
    [_held, "==", true, "the value skd_fnc_tryWaitUntil returned"] call skd_fnc_expect;
}] call skd_fnc_step;

["a try wait that reaches its ceiling", {
    private _start = diag_tickTime;
    private _held = ["p1", { false }, "never", 0.5] call skd_fnc_tryWaitUntil;
    [_held, "==", false, "the value skd_fnc_tryWaitUntil returned at its ceiling"] call skd_fnc_expect;
    [diag_tickTime - _start, ">=", 0.5, "seconds the try wait took"] call skd_fnc_expect;
    [diag_tickTime - _start, "<", 3, "seconds the try wait took"] call skd_fnc_expect;
}] call skd_fnc_step;
