// @check US-074 a check that names sp-cba alone runs only there and is the only check tagged solo
// @tags US-074 acceptance solo
// @setups sp-cba

["CBA is loaded on p1", {
    ["p1", { isClass (configFile >> "CfgPatches" >> "cba_main") }, "==", true, "CBA is loaded on p1"] call skd_fnc_expectOn;
}] call skd_fnc_step;
