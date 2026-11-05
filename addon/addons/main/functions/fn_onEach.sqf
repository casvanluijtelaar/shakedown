/*
    skd_fnc_onEach [machines, code, args, ceiling] - runs code on each machine of a group or list
    and returns a HashMap from machine name to the value it returned there (check API, "Running
    code on machines"; CTL-01).
*/
if !(["skd_fnc_onEach"] call skd_fnc_guard) exitWith {};
params ["_skd_machines", "_skd_code", "_skd_args", "_skd_ceiling"];
private _skd_targets = ["skd_fnc_onEach", 1, if (isNil "_skd_machines") then { nil } else { _skd_machines }, true] call skd_fnc_targets;
if (isNil "_skd_targets") exitWith {};
if (isNil "_skd_code" || { !(_skd_code isEqualType {}) }) exitWith {
    ["skd_fnc_onEach", 2, "code", "Code", if (isNil "_skd_code") then { nil } else { _skd_code }] call skd_fnc_argError;
};
if (isNil "_skd_args") then { _skd_args = [] };
if !(_skd_args isEqualType []) exitWith { ["skd_fnc_onEach", 3, "args", "an Array", _skd_args] call skd_fnc_argError };
_skd_ceiling = ["skd_fnc_onEach", 4, if (isNil "_skd_ceiling") then { nil } else { _skd_ceiling }] call skd_fnc_ceiling;
if (isNil "_skd_ceiling") exitWith {};

private _skd_out = createHashMap;
private _skd_late = [];
private _skd_start = diag_tickTime;
{
    private _skd_compiled = ["skd_fnc_onEach", _x, _skd_code] call skd_fnc_compileFor;
    if (isNil "_skd_compiled") exitWith {};
    private _skd_left = (_skd_ceiling - (diag_tickTime - _skd_start)) max 0.001;
    ([_x, _skd_compiled, _skd_args, _skd_left] call skd_fnc_runOn) params ["_skd_done", "_skd_value"];
    if (_skd_done) then {
        // A machine whose code returned nothing has no key in the map.
        if (_skd_value isNotEqualTo []) then { _skd_out set [_x, _skd_value select 0] };
    } else {
        _skd_late pushBack _x;
    };
} forEach _skd_targets;
if (_skd_late isNotEqualTo []) exitWith {
    call skd_fnc_atCeiling;
    ["fail", "wait-ceiling", format ["skd_fnc_onEach: the code was still running at its ceiling of %1 s on %2.", _skd_ceiling, _skd_late joinString ", "]] call skd_fnc_end;
};
_skd_out
