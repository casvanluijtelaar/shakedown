// @check US-014 CTL-04 code for p3 in a session with two clients ends fail check-error naming p3 and the machines
// @tags US-014 CTL-04 DM-04 acceptance
// @setups ded

["ask p3", {
    ["p3", { 1 }] call skd_fnc_on;
}] call skd_fnc_step;
["never runs", { true }] call skd_fnc_step;
