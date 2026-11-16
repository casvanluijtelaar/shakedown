// @check US-074 a check with no @setups names every setup and runs in each, attaching what it found
// @tags US-074 acceptance

["read the setup", {
    ["multiplayer", isMultiplayer] call skd_fnc_evidence;
    ["cba on p1", ["p1", { isClass (configFile >> "CfgPatches" >> "cba_main") }] call skd_fnc_on] call skd_fnc_evidence;
}] call skd_fnc_step;
