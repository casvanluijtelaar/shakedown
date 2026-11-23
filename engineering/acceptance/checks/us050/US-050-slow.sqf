// @check US-050 AGT-03 a check that takes about 55 s
// @tags US-050 AGT-03 acceptance

private _r = ["take 55 s", {
    ["p1", { false }, "a condition that never holds", 55] call skd_fnc_tryWaitUntil
}] call skd_fnc_step;
[_r, "==", false, "the wait ran to its ceiling"] call skd_fnc_expect;
