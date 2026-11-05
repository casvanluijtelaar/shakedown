/*
    Compares an actual value with an expected one by an operator of the check API ("Assertions").

    Arguments: [actual, op, expected] - actual and expected wrapped: [] for nothing, else [value].
    Returns: [held, typeError] - typeError is "" when the operands fit the operator, else what the
    operator needs.
*/
params ["_actualW", "_op", "_expectedW"];
private _noActual = _actualW isEqualTo [] || { isNil { _actualW select 0 } };
private _noExpected = _expectedW isEqualTo [] || { isNil { _expectedW select 0 } };
private _actual = if (_noActual) then { 0 } else { _actualW select 0 };
private _expected = if (_noExpected) then { 0 } else { _expectedW select 0 };
private _isPos = {
    params ["_p"];
    _p isEqualType [] && { count _p in [2, 3] } && { (_p findIf { !(_x isEqualType 0) }) < 0 }
};
private _held = false;
private _typeError = "";
switch (_op) do {
    case "==": {
        _held = if (_noActual || _noExpected) then { _noActual && _noExpected } else { _actual isEqualTo _expected };
    };
    case "!=": {
        _held = if (_noActual || _noExpected) then { !(_noActual && _noExpected) } else { !(_actual isEqualTo _expected) };
    };
    case "<";
    case "<=";
    case ">";
    case ">=": {
        if (_noActual || { _noExpected } || { !(_actual isEqualType 0) } || { !(_expected isEqualType 0) }) then {
            _typeError = "two numbers";
        } else {
            _held = switch (_op) do {
                case "<": { _actual < _expected };
                case "<=": { _actual <= _expected };
                case ">": { _actual > _expected };
                default { _actual >= _expected };
            };
        };
    };
    case "~=": {
        if (_noActual || { _noExpected } || { !(_expected isEqualType []) } || { count _expected != 2 } || { !((_expected select 1) isEqualType 0) }) then {
            _typeError = "expected as [target, tolerance]";
        } else {
            _expected params ["_target", "_tolerance"];
            if (_actual isEqualType 0 && { _target isEqualType 0 }) then {
                _held = abs (_actual - _target) <= _tolerance;
            } else {
                if ([_actual] call _isPos && { [_target] call _isPos }) then {
                    _held = (_actual distance _target) <= _tolerance;
                } else {
                    _typeError = "a number or a position, and a target of the same kind";
                };
            };
        };
    };
    case "in": {
        if (_noExpected || { !(_expected isEqualType []) }) then {
            _typeError = "an array as expected";
        } else {
            _held = !_noActual && { (_expected findIf { _x isEqualTo _actual }) >= 0 };
        };
    };
    case "contains": {
        if (!_noActual && { _actual isEqualType [] }) then {
            _held = !_noExpected && { (_actual findIf { _x isEqualTo _expected }) >= 0 };
        } else {
            if (!_noActual && { _actual isEqualType "" } && { !_noExpected } && { _expected isEqualType "" }) then {
                _held = (_actual find _expected) >= 0;
            } else {
                _typeError = "an array or a string as actual";
            };
        };
    };
    default { _typeError = "a known operator" };
};
[_held, _typeError]
