// @check US-023 OBS-07 tryWaitUntil with a ceiling of 1 s on a condition that never holds returns false and the check passes
// @tags US-023 OBS-07 acceptance
// @setups ded

private _r = ["p1", { false }, "never holds", 1] call skd_fnc_tryWaitUntil;
["returned", _r] call skd_fnc_evidence;
[_r, "==", false, "tryWaitUntil returned false"] call skd_fnc_expect;
["after the wait", { true }] call skd_fnc_step;
