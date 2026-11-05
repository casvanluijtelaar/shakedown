/*
    skd_fnc_expect [actual, op, expected, description] - an immediate assertion (check API,
    "Assertions"). Fails the check at once with fail / assertion when it does not hold.

    Returns: true.
*/
if !(["skd_fnc_expect"] call skd_fnc_guard) exitWith {};
params ["_skd_actual", "_skd_op", "_skd_expected", "_skd_description"];
private _skd_ops = ["==", "!=", "<", "<=", ">", ">=", "~=", "in", "contains"];
if (isNil "_skd_op" || { !(_skd_op isEqualType "") } || { !(_skd_op in _skd_ops) }) exitWith {
    ["skd_fnc_expect", 2, "op", "one of " + (_skd_ops joinString " "), if (isNil "_skd_op") then { nil } else { _skd_op }] call skd_fnc_argError;
};
if (isNil "_skd_description" || { !(_skd_description isEqualType "") }) exitWith {
    ["skd_fnc_expect", 4, "description", "a String", if (isNil "_skd_description") then { nil } else { _skd_description }] call skd_fnc_argError;
};
private _skd_show = { params ["_v"]; if (isNil "_v") then { "nil" } else { str _v } };
private _skd_noActual = isNil "_skd_actual";
private _skd_noExpected = isNil "_skd_expected";
private _skd_isPos = {
    params ["_p"];
    _p isEqualType [] && { count _p in [2, 3] } && { (_p findIf { !(_x isEqualType 0) }) < 0 }
};
private _skd_held = false;
private _skd_typeError = "";
switch (_skd_op) do {
    case "==": {
        _skd_held = if (_skd_noActual || _skd_noExpected) then { _skd_noActual && _skd_noExpected } else { _skd_actual isEqualTo _skd_expected };
    };
    case "!=": {
        _skd_held = if (_skd_noActual || _skd_noExpected) then { !(_skd_noActual && _skd_noExpected) } else { !(_skd_actual isEqualTo _skd_expected) };
    };
    case "<";
    case "<=";
    case ">";
    case ">=": {
        if (_skd_noActual || { _skd_noExpected } || { !(_skd_actual isEqualType 0) } || { !(_skd_expected isEqualType 0) }) then {
            _skd_typeError = "two numbers";
        } else {
            _skd_held = switch (_skd_op) do {
                case "<": { _skd_actual < _skd_expected };
                case "<=": { _skd_actual <= _skd_expected };
                case ">": { _skd_actual > _skd_expected };
                default { _skd_actual >= _skd_expected };
            };
        };
    };
    case "~=": {
        if (_skd_noActual || { _skd_noExpected } || { !(_skd_expected isEqualType []) } || { count _skd_expected != 2 } || { !((_skd_expected select 1) isEqualType 0) }) then {
            _skd_typeError = "expected as [target, tolerance]";
        } else {
            _skd_expected params ["_skd_target", "_skd_tolerance"];
            if (_skd_actual isEqualType 0 && { _skd_target isEqualType 0 }) then {
                _skd_held = abs (_skd_actual - _skd_target) <= _skd_tolerance;
            } else {
                if ([_skd_actual] call _skd_isPos && { [_skd_target] call _skd_isPos }) then {
                    _skd_held = (_skd_actual distance _skd_target) <= _skd_tolerance;
                } else {
                    _skd_typeError = "a number or a position, and a target of the same kind";
                };
            };
        };
    };
    case "in": {
        if (_skd_noExpected || { !(_skd_expected isEqualType []) }) then {
            _skd_typeError = "an array as expected";
        } else {
            _skd_held = !_skd_noActual && { (_skd_expected findIf { _x isEqualTo _skd_actual }) >= 0 };
        };
    };
    case "contains": {
        if (!_skd_noActual && { _skd_actual isEqualType [] }) then {
            _skd_held = !_skd_noExpected && { (_skd_actual findIf { _x isEqualTo _skd_expected }) >= 0 };
        } else {
            if (!_skd_noActual && { _skd_actual isEqualType "" } && { !_skd_noExpected } && { _skd_expected isEqualType "" }) then {
                _skd_held = (_skd_actual find _skd_expected) >= 0;
            } else {
                _skd_typeError = "an array or a string as actual";
            };
        };
    };
};
private _skd_actualText = [if (_skd_noActual) then { nil } else { _skd_actual }] call _skd_show;
private _skd_expectedText = [if (_skd_noExpected) then { nil } else { _skd_expected }] call _skd_show;
if (_skd_typeError != "") exitWith {
    ["fail", "check-error", format ["skd_fnc_expect: operator %1 needs %2; got actual %3 and expected %4.",
        _skd_op, _skd_typeError, _skd_actualText, _skd_expectedText]] call skd_fnc_end;
};
if (!_skd_held) exitWith {
    private _skd_observed = createHashMap;
    if (!_skd_noActual) then { _skd_observed set [skd_machine, _skd_actual] };
    ["fail", "assertion", format ["%1: expected %2 %3, got %4.", _skd_description, _skd_op, _skd_expectedText, _skd_actualText],
        _skd_observed] call skd_fnc_end;
};
true
