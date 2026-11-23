// @check US-074 a check that names sp and dedicated-2 runs only in those two
// @tags US-074 acceptance
// @setups sp dedicated-2

["read the setup", {
    ["multiplayer", isMultiplayer] call skd_fnc_evidence;
}] call skd_fnc_step;
