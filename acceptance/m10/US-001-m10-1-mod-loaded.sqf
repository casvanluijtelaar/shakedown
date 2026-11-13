// @check US-001 A mod named by a relative path that climbs out of the project folder loads on every machine
// @tags US-001 US-006 CFG-01 m10-1 acceptance

["the fixture's CfgPatches class is loaded on every machine", {
    ["all", { isClass (configFile >> "CfgPatches" >> "skdfx_main") }, "==", true, "skdfx_main in CfgPatches", 10] call skd_fnc_expectEach;
}] call skd_fnc_step;
