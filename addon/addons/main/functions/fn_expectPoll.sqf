/*
    The polled assertion behind skd_fnc_expectOn, skd_fnc_expectEach and skd_fnc_expectAgree (check
    API, "Assertions"; OBS-04, OBS-06): reads the value on every machine every frame until the
    comparison holds on every machine at once, or the ceiling passes.

    Arguments: [function, kind, given] - kind "on", "each" or "agree"; given the function's
    arguments: [machines, value, op, expected, description, ceiling, args] for "on" and "each",
    [machines, value, description, tolerance, ceiling, args] for "agree".
    Returns: the last value for "on"; a HashMap from machine name to the last value otherwise.
*/
params ["_function", "_kind", "_given"];
private ["_machines", "_value", "_op", "_expected", "_description", "_tolerance", "_ceiling", "_args"];
private _noExpected = true;
if (_kind isEqualTo "agree") then {
    _given params ["_m", "_v", "_d", "_t", "_c", "_a"];
    _machines = if (isNil "_m") then { nil } else { _m };
    _value = if (isNil "_v") then { nil } else { _v };
    _description = if (isNil "_d") then { nil } else { _d };
    _tolerance = if (isNil "_t") then { 0 } else { _t };
    _ceiling = if (isNil "_c") then { nil } else { _c };
    _args = if (isNil "_a") then { [] } else { _a };
} else {
    _given params ["_m", "_v", "_o", "_e", "_d", "_c", "_a"];
    _machines = if (isNil "_m") then { nil } else { _m };
    _value = if (isNil "_v") then { nil } else { _v };
    _op = if (isNil "_o") then { nil } else { _o };
    _noExpected = isNil "_e";
    _expected = if (_noExpected) then { nil } else { _e };
    _description = if (isNil "_d") then { nil } else { _d };
    _ceiling = if (isNil "_c") then { nil } else { _c };
    _args = if (isNil "_a") then { [] } else { _a };
};
private _agree = _kind isEqualTo "agree";
private _n = [5, 3] select _agree;

private _targets = [_function, 1, if (isNil "_machines") then { nil } else { _machines }, _kind != "on"] call skd_fnc_targets;
if (isNil "_targets") exitWith {};
if (isNil "_value" || { !(_value isEqualType {}) }) exitWith {
    [_function, 2, "value", "Code returning the value to test", if (isNil "_value") then { nil } else { _value }] call skd_fnc_argError;
};
if (!_agree && { isNil "_op" || { !(_op isEqualType "") } || { !(_op in skd_ops) } }) exitWith {
    [_function, 3, "op", "one of " + (skd_ops joinString " "), if (isNil "_op") then { nil } else { _op }] call skd_fnc_argError;
};
if (isNil "_description" || { !(_description isEqualType "") }) exitWith {
    [_function, _n, "description", "a String", if (isNil "_description") then { nil } else { _description }] call skd_fnc_argError;
};
if (_agree && { !(_tolerance isEqualType 0) || { _tolerance < 0 } }) exitWith {
    [_function, 4, "tolerance", "a number of 0 or more", _tolerance] call skd_fnc_argError;
};
_ceiling = [_function, _n + 1, if (isNil "_ceiling") then { nil } else { _ceiling }] call skd_fnc_ceiling;
if (isNil "_ceiling") exitWith {};
if !(_args isEqualType []) exitWith { [_function, _n + 2, "args", "an Array", _args] call skd_fnc_argError };
private _compiled = [_function, _targets joinString ", ", _value] call skd_fnc_compileFor;
if (isNil "_compiled") exitWith {};

private _expectedW = if (_noExpected) then { [] } else { [_expected] };
private _isPos = {
    params ["_p"];
    _p isEqualType [] && { count _p in [2, 3] } && { (_p findIf { !(_x isEqualType 0) }) < 0 }
};
// Whether one machine's value meets the expectation; for "agree", against the first machine's.
private _meets = {
    params ["_w", "_first"];
    if (_agree) exitWith {
        if (_w isEqualTo _first) exitWith { true };
        if (_w isEqualTo [] || { _first isEqualTo [] }) exitWith { false };
        private _a = _w select 0;
        private _b = _first select 0;
        if (_a isEqualType 0 && { _b isEqualType 0 }) exitWith { abs (_a - _b) <= _tolerance };
        if ([_a] call _isPos && { [_b] call _isPos }) exitWith { (_a distance _b) <= _tolerance };
        false
    };
    private _r = [_w, _op, _expectedW] call skd_fnc_compare;
    if ((_r select 1) != "") then { _typeError = [_r select 1, [_w] call skd_fnc_show] };
    _r select 0
};

private _pollers = _targets apply { [_x, _compiled, _args] call skd_fnc_poller };
private _start = diag_tickTime;
private _typeError = [];
private _held = false;
private _missing = [];
waitUntil {
    if ((_pollers findIf { (_x select 1) == 0 }) < 0) then {
        private _first = (_pollers select 0) select 0;
        _missing = [];
        {
            if !([(_x select 0), _first] call _meets) then { _missing pushBack (_targets select _forEachIndex) };
        } forEach _pollers;
        _held = _missing isEqualTo [] && { _typeError isEqualTo [] };
    };
    _held || { _typeError isNotEqualTo [] } || { diag_tickTime - _start > _ceiling }
};
{ [_x] call skd_fnc_stopPoller } forEach _pollers;

private _observed = createHashMap;
{ if ((_x select 0) isNotEqualTo []) then { _observed set [_targets select _forEachIndex, (_x select 0) select 0] } } forEach _pollers;
if (_typeError isNotEqualTo []) exitWith {
    ["fail", "check-error", format ["%1: operator %2 needs %3; ""%4"" read %5 and expected %6.",
        _function, _op, _typeError select 0, _description, _typeError select 1, [_expectedW] call skd_fnc_show]] call skd_fnc_end;
};
if (_held) exitWith {
    if (_kind isEqualTo "on") then {
        private _w = (_pollers select 0) select 0;
        if (_w isEqualTo []) then { nil } else { _w select 0 }
    } else { _observed };
};
call skd_fnc_atCeiling;
private _values = (_targets apply { format ["%1 %2", _x, [(_pollers select (_targets find _x)) select 0] call skd_fnc_show] }) joinString ", ";
private _message = if (_agree) then {
    format ["%1: the machines did not agree within %2 s (tolerance %3): %4; %5 differ from %6.",
        _description, _ceiling, _tolerance, _values, _missing joinString ", ", _targets select 0]
} else {
    if (_kind isEqualTo "on") then {
        format ["%1: expected %2 %3 on %4 within %5 s, last read %6.",
            _description, _op, [_expectedW] call skd_fnc_show, _targets select 0, _ceiling, [(_pollers select 0) select 0] call skd_fnc_show]
    } else {
        format ["%1: expected %2 %3 on every machine within %4 s; last read %5; not met on %6.",
            _description, _op, [_expectedW] call skd_fnc_show, _ceiling, _values, _missing joinString ", "]
    };
};
["fail", "assertion", _message, _observed] call skd_fnc_end;
