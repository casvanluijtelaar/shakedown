// @check US-089 p1 uses its own ACE self action and every machine sees its effect once
// @setups ded-ace sp-ace
private _unit = ["p1"] call skd_fnc_player;
private _before = ["p1", { (_this select 0) getVariable ["skdfx_aceSignals", 0] }, [_unit]] call skd_fnc_on;
["p1", "p1", ["ACE_SelfActions", "skdfx_aceSignal"]] call skd_fnc_useAceAction;
["all", { (_this select 0) getVariable ["skdfx_aceSignals", 0] }, "==", _before + 1, "every machine sees p1's ACE signal counted once", 10, [_unit]] call skd_fnc_expectEach;
