// @check US-062 OBS-01 a tryWaitUntil whose condition raises a script error ends script-error instead of returning false
// @tags US-062 OBS-01 ERR-03 acceptance
// @setups ded

private _r = ["p1", { _skdfx_undefined > 1 }, "flag above one", 10] call skd_fnc_tryWaitUntil;
["returned", _r] call skd_fnc_evidence;
