// @check US-090 an ACE action on a machine without ACE ends check-error saying ACE is not loaded
// @setups sp-noace
["p1", "p1", ["ACE_SelfActions", "skdfx_aceSignal"], 5] call skd_fnc_useAceAction;
