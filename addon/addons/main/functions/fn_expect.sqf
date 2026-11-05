/*
    skd_fnc_expect [actual, op, expected, description] - an immediate assertion (check API,
    "Assertions"). Fails the check at once with fail / assertion when it does not hold.

    Returns: true.
*/
if !(["skd_fnc_expect"] call skd_fnc_guard) exitWith {};
params ["_skd_actual", "_skd_op", "_skd_expected", "_skd_description"];
if (isNil "_skd_op" || { !(_skd_op isEqualType "") } || { !(_skd_op in skd_ops) }) exitWith {
    ["skd_fnc_expect", 2, "op", "one of " + (skd_ops joinString " "), if (isNil "_skd_op") then { nil } else { _skd_op }] call skd_fnc_argError;
};
if (isNil "_skd_description" || { !(_skd_description isEqualType "") }) exitWith {
    ["skd_fnc_expect", 4, "description", "a String", if (isNil "_skd_description") then { nil } else { _skd_description }] call skd_fnc_argError;
};
private _skd_a = if (isNil "_skd_actual") then { [] } else { [_skd_actual] };
private _skd_e = if (isNil "_skd_expected") then { [] } else { [_skd_expected] };
([_skd_a, _skd_op, _skd_e] call skd_fnc_compare) params ["_skd_held", "_skd_typeError"];
if (_skd_typeError != "") exitWith {
    ["fail", "check-error", format ["skd_fnc_expect: operator %1 needs %2; got actual %3 and expected %4.",
        _skd_op, _skd_typeError, [_skd_a] call skd_fnc_show, [_skd_e] call skd_fnc_show]] call skd_fnc_end;
};
if (!_skd_held) exitWith {
    private _skd_observed = createHashMap;
    if (_skd_a isNotEqualTo []) then { _skd_observed set [skd_machine, _skd_a select 0] };
    ["fail", "assertion", format ["%1: expected %2 %3, got %4.", _skd_description, _skd_op, [_skd_e] call skd_fnc_show, [_skd_a] call skd_fnc_show],
        _skd_observed] call skd_fnc_end;
};
true
