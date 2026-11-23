// @check US-022 A check that presses no key passes in a setup without CBA
// @tags US-022 API-05 acceptance

["the setup has the fixture and no CBA", {
    [isClass (configFile >> "CfgPatches" >> "cba_main"), "==", false, "cba_main is loaded"] call skd_fnc_expect;
    [isClass (configFile >> "CfgPatches" >> "skdfx_main"), "==", true, "skdfx_main is loaded"] call skd_fnc_expect;
}] call skd_fnc_step;

["run code on p1", {
    private _sum = ["p1", { (_this select 0) + (_this select 1) }, [2, 3]] call skd_fnc_on;
    [_sum, "==", 5, "2 + 3 on p1"] call skd_fnc_expect;
}] call skd_fnc_step;
