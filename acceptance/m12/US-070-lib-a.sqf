// @check US-070 the library's rope helper is callable in the body and in code sent to every machine
// @tags US-070 acceptance
// @setups ded sp

["the body calls the library", {
    private _rope = call skd_lib_rope;
    ["lead", _rope get "version"] call skd_fnc_evidence;
    [[21] call (_rope get "double"), "==", 42, "double from the body"] call skd_fnc_expect;
}] call skd_fnc_step;

["code sent to every machine calls the library", {
    private _machines = [] call skd_fnc_machines;
    private _got = [_machines, { private _r = call skd_lib_rope; [_r get "version", [21] call (_r get "double")] }] call skd_fnc_onEach;
    {
        private _m = _x;
        private _v = _got getOrDefault [_m, [-1, -1]];
        ["on " + _m, _v select 0] call skd_fnc_evidence;
        [_v select 1, "==", 42, "double on " + _m] call skd_fnc_expect;
    } forEach _machines;
}] call skd_fnc_step;
