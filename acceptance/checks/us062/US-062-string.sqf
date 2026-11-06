// @check US-062 OBS-01 DM-04 a waitUntil whose condition returns a String ends check-error at once
// @tags US-062 OBS-01 DM-04 acceptance
// @setups ded

["p1", { "yes" }, "a string, not a Boolean"] call skd_fnc_waitUntil;
