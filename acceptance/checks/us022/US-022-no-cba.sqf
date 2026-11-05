// @check US-022 Pressing a keybind in a setup without CBA is the author's to fix
// @tags US-022 acceptance

["the setup has no CBA", {
    [isClass (configFile >> "CfgPatches" >> "cba_main"), "==", false, "cba_main is loaded"] call skd_fnc_expect;
}] call skd_fnc_step;

["press the fixture signal", {
    ["p1", "skd_fixture", "signal"] call skd_fnc_pressKey;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
