// @check US-061 CTL-05 ERR-03 code on p1 that raises a script error returns 7, the body runs on, and the check ends script-error
// @tags US-061 CTL-05 ERR-01 ERR-03 acceptance
// @setups ded

private _v = ["throw on p1", {
    ["p1", { [] call skdfx_fnc_throw; 7 }] call skd_fnc_on
}] call skd_fnc_step;
["value", _v] call skd_fnc_evidence;
[_v, "==", 7, "the code returned 7 after the error"] call skd_fnc_expect;
["after the error", { true }] call skd_fnc_step;
["reached the end", true] call skd_fnc_evidence;
