// @check US-070 a second check calls both library files in the body and on every machine
// @tags US-070 acceptance
// @setups ded sp

["the body calls both library files", {
    [call skd_lib_kit, "==", 42, "skd_lib_kit in the body"] call skd_fnc_expect;
    ["lead", (call skd_lib_rope) get "version"] call skd_fnc_evidence;
}] call skd_fnc_step;

["each machine calls both library files", {
    {
        private _m = _x;
        [_m, { call skd_lib_kit }, "==", 42, "skd_lib_kit on " + _m] call skd_fnc_expectOn;
        ["on " + _m, [_m, { (call skd_lib_rope) get "version" }] call skd_fnc_on] call skd_fnc_evidence;
    } forEach ([] call skd_fnc_machines);
}] call skd_fnc_step;
