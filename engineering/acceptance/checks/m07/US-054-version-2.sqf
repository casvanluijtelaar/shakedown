// @check US-054 skdfx_fnc_version returns "2" on every machine after the reload of the staged change
// @tags m07-demo US-054 RLD-01 RUN-01 acceptance
// @setups rld7 rld7sp

["onEach of skdfx_fnc_version answers '2' from every machine of the session", {
    private _names = [] call skd_fnc_machines;
    private _each = ["all", { call skdfx_fnc_version }] call skd_fnc_onEach;
    ["versions", _each] call skd_fnc_evidence;
    private _keys = keys _each;
    _keys sort true;
    _names sort true;
    [_keys, "==", _names, "the machines that answered"] call skd_fnc_expect;
    { [_y, "==", "2", format ["skdfx_fnc_version on %1", _x]] call skd_fnc_expect } forEach _each;
}] call skd_fnc_step;
