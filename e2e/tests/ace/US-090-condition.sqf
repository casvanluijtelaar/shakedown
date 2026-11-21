// @check US-090 An ACE action whose condition is false fails at the ceiling, saying so
// @tags US-090 ace
// @setups sp-ace dedicated-ace
["p1", "p1", ["ACE_SelfActions", "skdfx_aceNever"], 2] call skd_fnc_useAceAction;
