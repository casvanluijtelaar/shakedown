// @check US-089 p1 uses the fixture's ACE self action, and every machine sees its count
// @tags US-089 ace
// @setups sp-ace dedicated-ace
private _p1 = ["p1"] call skd_fnc_player;
private _before = ["p1", { player getVariable ["skdfx_aceSignals", 0] }] call skd_fnc_on;
["p1", "p1", ["ACE_SelfActions", "skdfx_aceSignal"]] call skd_fnc_useAceAction;
["all", { (_this select 0) getVariable ["skdfx_aceSignals", 0] }, "==", _before + 1, "every machine sees one ACE signal more", 10, [_p1]] call skd_fnc_expectEach;
