/*
    The wait behind skd_fnc_waitUntil and skd_fnc_tryWaitUntil (check API, "Waits"; OBS-01, OBS-07):
    evaluates the condition on the machine every frame until it returns true or the ceiling passes.
    A condition that raises a script error ends the check fail / script-error at that evaluation;
    one that returns anything but a Boolean ends it fail / check-error naming the type it returned.

    Arguments: [function, try, [machine, condition, description, ceiling, args]]
    Returns: true when the condition held; false at the ceiling for skd_fnc_tryWaitUntil.
*/
params ["_function", "_try", "_given"];
_given params ["_machine", "_condition", "_description", "_ceiling", "_args"];
private _targets = [_function, 1, if (isNil "_machine") then { nil } else { _machine }, false] call skd_fnc_targets;
if (isNil "_targets") exitWith {};
if (isNil "_condition" || { !(_condition isEqualType {}) }) exitWith {
    [_function, 2, "condition", "Code returning a Boolean", if (isNil "_condition") then { nil } else { _condition }] call skd_fnc_argError;
};
if (isNil "_description" || { !(_description isEqualType "") }) exitWith {
    [_function, 3, "description", "a String saying what is awaited", if (isNil "_description") then { nil } else { _description }] call skd_fnc_argError;
};
_ceiling = [_function, 4, if (isNil "_ceiling") then { nil } else { _ceiling }] call skd_fnc_ceiling;
if (isNil "_ceiling") exitWith {};
if (isNil "_args") then { _args = [] };
if !(_args isEqualType []) exitWith { [_function, 5, "args", "an Array", _args] call skd_fnc_argError };
private _target = _targets select 0;
private _compiled = [_function, _target, _condition] call skd_fnc_compileFor;
if (isNil "_compiled") exitWith {};

private _poller = [_target, _compiled, _args, _ceiling] call skd_fnc_poller;
private _start = diag_tickTime;
private _met = false;
private _wrong = [];
private _raised = "";
waitUntil {
    _raised = _poller select 5;
    if (_raised isEqualTo "" && { (_poller select 1) > 0 }) then {
        private _last = _poller select 0;
        if (_last isEqualTo [true]) then { _met = true } else {
            if !(_last isEqualTo [false]) then { _wrong = [[_last] call skd_fnc_show, [_last] call skd_fnc_typeName] };
        };
    };
    _met || { _raised isNotEqualTo "" } || { _wrong isNotEqualTo [] } || { diag_tickTime - _start > _ceiling }
};
[_poller] call skd_fnc_stopPoller;
if (_met) exitWith { true };
if (_raised isNotEqualTo "") exitWith {
    ["fail", "script-error", format ["%1: the condition for ""%2"" on %3 raised a script error: %4. Fix the condition or the code it calls.",
        _function, _description, _target, _raised], nil, _target] call skd_fnc_end;
};
if (_wrong isNotEqualTo []) exitWith {
    ["fail", "check-error", format ["%1: the condition for ""%2"" on %3 returned %4 (%5), not a Boolean. Make it return true or false.",
        _function, _description, _target, _wrong select 1, _wrong select 0]] call skd_fnc_end;
};
call skd_fnc_atCeiling;
if (_try) exitWith { false };
private _observed = createHashMap;
_observed set [_target, false];
["fail", "wait-ceiling", format ["%1: ""%2"" was not met on %3 within its ceiling of %4 s.", _function, _description, _target, _ceiling], _observed] call skd_fnc_end;
