/*
    skd_fnc_cleanup [code, args, machine] - registers code to run on machine (default the lead) at
    the start of the next reset, each machine's cleanups last registered first (check API; RST-01,
    P-06 step 1). A cleanup for another machine is handed to it at once and kept there, so it runs
    even when the check ends before its body does.
*/
if !(["skd_fnc_cleanup"] call skd_fnc_guard) exitWith {};
params ["_skd_code", "_skd_args", "_skd_machine"];
if (isNil "_skd_code" || { !(_skd_code isEqualType {}) }) exitWith {
    ["skd_fnc_cleanup", 1, "code", "Code", if (isNil "_skd_code") then { nil } else { _skd_code }] call skd_fnc_argError;
};
if (isNil "_skd_args") then { _skd_args = [] };
if !(_skd_args isEqualType []) exitWith { ["skd_fnc_cleanup", 2, "args", "an Array", _skd_args] call skd_fnc_argError };
private _skd_on = skd_machine;
if (!isNil "_skd_machine") then {
    private _skd_targets = ["skd_fnc_cleanup", 3, _skd_machine, false] call skd_fnc_targets;
    if (isNil "_skd_targets") exitWith { _skd_on = "" };
    _skd_on = _skd_targets select 0;
};
if (_skd_on isEqualTo "") exitWith {};
if (_skd_on isEqualTo skd_machine || { (skd_check get "mode") isEqualTo "singleplayer" }) exitWith {
    skd_cleanups pushBack [_skd_code, _skd_args, skd_check get "id"];
    nil
};
private _skd_compiled = ["skd_fnc_cleanup", _skd_on, _skd_code] call skd_fnc_compileFor;
if (isNil "_skd_compiled") exitWith {};
([_skd_on, {
    params ["_source", "_args", "_check"];
    skd_cleanups pushBack [compile _source, _args, _check];
    true
}, [toString _skd_code, _skd_args, skd_check get "id"], 10] call skd_fnc_runOn) params ["_skd_done"];
if (!_skd_done) exitWith {
    ["fail", "wait-ceiling", format ["skd_fnc_cleanup: %1 did not take the cleanup within 10 s.", _skd_on], nil, _skd_on] call skd_fnc_end;
};
nil
