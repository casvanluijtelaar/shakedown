// @check US-062 OBS-01 a waitUntil whose condition raises a script error ends script-error at once, not at its ceiling
// @tags US-062 OBS-01 ERR-03 acceptance
// @setups ded

["p1", { _skdfx_undefined > 1 }, "flag above one", 10] call skd_fnc_waitUntil;
