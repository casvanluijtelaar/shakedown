// @check US-090 an ACE action whose condition is false fails at the ceiling saying so
// @setups sp-ace
["p1", "p1", ["ACE_SelfActions", "skdfx_aceNever"], 3] call skd_fnc_useAceAction;
