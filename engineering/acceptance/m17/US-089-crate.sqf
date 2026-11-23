// @check US-089 p1 opens the crate through its ACE action and the server and every client see it open
// @setups ded-ace
private _before = ["server", { missionNamespace getVariable ["skdfx_openCount", 0] }] call skd_fnc_on;
private _crate = ["skdfx_Crate", AGLToASL [130, 132, 0], 180] call skd_fnc_create;
["p1", AGLToASL [130, 130, 0], 0] call skd_fnc_place;
["p1", _crate, ["ACE_MainActions", "skdfx_aceOpen"]] call skd_fnc_useAceAction;
["all", { (_this select 0) getVariable ["skdfx_open", false] }, "==", true, "every machine sees the crate open", 10, [_crate]] call skd_fnc_expectEach;
["server", { missionNamespace getVariable ["skdfx_openCount", 0] }, "==", _before + 1, "the server counted one opening", 10] call skd_fnc_expectOn;
