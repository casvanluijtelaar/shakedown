// @check US-089 p1 opens the fixture's crate through its ACE action, and every machine sees it
// @tags US-089 ace
// @setups sp-ace dedicated-ace
private _crate = ["skdfx_Crate", AGLToASL [130, 132, 0], 180] call skd_fnc_create;
["p1", AGLToASL [130, 130, 0], 0] call skd_fnc_place;
["p1", _crate, ["ACE_MainActions", "skdfx_aceOpen"]] call skd_fnc_useAceAction;
["all", { (_this select 0) getVariable ["skdfx_open", false] }, "==", true, "every machine sees the crate open", 10, [_crate]] call skd_fnc_expectEach;
["server", { missionNamespace getVariable ["skdfx_openCount", 0] }, ">=", 1, "the server counts the crate opened"] call skd_fnc_expectOn;
