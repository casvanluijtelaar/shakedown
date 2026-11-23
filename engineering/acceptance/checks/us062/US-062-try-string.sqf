// @check US-062 OBS-01 DM-04 a tryWaitUntil whose condition returns a String ends check-error instead of returning false
// @tags US-062 OBS-01 DM-04 acceptance
// @setups ded

private _r = ["p1", { "yes" }, "a string, not a Boolean"] call skd_fnc_tryWaitUntil;
["returned", _r] call skd_fnc_evidence;
