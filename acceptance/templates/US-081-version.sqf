// @check US-081 skdfx_cba_fnc_version returns the staged value on every machine right after the reload
// @tags US-081 US-082 acceptance
// @setups m14 m14sp

["onEach of skdfx_cba_fnc_version answers '{V}' from every machine of the session", {
    private _names = [] call skd_fnc_machines;
    private _each = ["all", { call skdfx_cba_fnc_version }] call skd_fnc_onEach;
    ["versions", _each] call skd_fnc_evidence;
    private _keys = keys _each;
    _keys sort true;
    _names sort true;
    [_keys, "==", _names, "the machines that answered"] call skd_fnc_expect;
    { [_y, "==", "{V}", format ["skdfx_cba_fnc_version on %1", _x]] call skd_fnc_expect } forEach _each;
}] call skd_fnc_step;
