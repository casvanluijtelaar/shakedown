// @check US-090 An ACE path whose last name does not exist fails at the ceiling, listing the names there
// @tags US-090 ace
// @setups sp-ace dedicated-ace
private _crate = ["skdfx_Crate", AGLToASL [130, 132, 0], 180] call skd_fnc_create;
["p1", AGLToASL [130, 130, 0], 0] call skd_fnc_place;
["p1", _crate, ["ACE_MainActions", "skdfx_noSuchAction"], 2] call skd_fnc_useAceAction;
