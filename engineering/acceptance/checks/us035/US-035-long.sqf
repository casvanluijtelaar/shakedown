// @check US-035 NFR-RL-03 a check on the dedicated session that takes about 60 s, for the runner to be killed during it
// @tags US-035 NFR-RL-03 acceptance
// @setups ded
// @timeout 120

private _r = ["take 60 s", {
    ["server", { false }, "a condition that never holds", 60] call skd_fnc_tryWaitUntil
}] call skd_fnc_step;
[_r, "==", false, "the wait ran to its ceiling"] call skd_fnc_expect;
