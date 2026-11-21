// @check US-090 An ACE action on a machine without ACE is a check error saying ACE is not loaded
// @tags US-090 ace
// @setups sp-cba
["p1", "p1", ["ACE_SelfActions", "skdfx_aceSignal"]] call skd_fnc_useAceAction;
