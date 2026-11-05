/*
    skd_fnc_on [machine, code, args, ceiling] - runs code on a machine and returns its value
    (check API, "Running code on machines").

    The code is sent as text and compiled on the target, so it does not see the check's local
    variables; args arrive as _this. It runs scheduled with an in-game ceiling.

    Returns: the value code returned.
*/
if !(["skd_fnc_on"] call skd_fnc_guard) exitWith {};
params ["_skd_machine", "_skd_code", "_skd_args", "_skd_ceiling"];
private _skd_targets = ["skd_fnc_on", 1, if (isNil "_skd_machine") then { nil } else { _skd_machine }, false] call skd_fnc_targets;
if (isNil "_skd_targets") exitWith {};
if (isNil "_skd_code" || { !(_skd_code isEqualType {}) }) exitWith {
    ["skd_fnc_on", 2, "code", "Code", if (isNil "_skd_code") then { nil } else { _skd_code }] call skd_fnc_argError;
};
if (isNil "_skd_args") then { _skd_args = [] };
if !(_skd_args isEqualType []) exitWith { ["skd_fnc_on", 3, "args", "an Array", _skd_args] call skd_fnc_argError };
_skd_ceiling = ["skd_fnc_on", 4, if (isNil "_skd_ceiling") then { nil } else { _skd_ceiling }] call skd_fnc_ceiling;
if (isNil "_skd_ceiling") exitWith {};
private _skd_target = _skd_targets select 0;
private _skd_compiled = ["skd_fnc_on", _skd_target, _skd_code] call skd_fnc_compileFor;
if (isNil "_skd_compiled") exitWith {};

([_skd_target, _skd_compiled, _skd_args, _skd_ceiling] call skd_fnc_runOn) params ["_skd_done", "_skd_value"];
if (!_skd_done) exitWith {
    call skd_fnc_atCeiling;
    ["fail", "wait-ceiling", format ["skd_fnc_on: the code on %1 was still running at its ceiling of %2 s.", _skd_target, _skd_ceiling]] call skd_fnc_end;
};
if (_skd_value isEqualTo []) exitWith { nil };
_skd_value select 0
