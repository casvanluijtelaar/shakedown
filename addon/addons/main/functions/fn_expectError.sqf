/*
    skd_fnc_expectError [text, code, args] - declares that a script error containing text (case-
    insensitive) is expected on any machine from the start of code until the check ends (check
    API, "Script errors"; ERR-04). Matching errors do not fail the check; when none came by the
    check's end it ends fail / expected-error-missing. The runner, which sees every machine's
    errors, matches them. code runs with call in the body's own script, as a step's code does.

    Returns: the value of code.
*/
if !(["skd_fnc_expectError"] call skd_fnc_guard) exitWith {};
params ["_skd_text", "_skd_code", ["_skd_args", []]];
if (isNil "_skd_text" || { !(_skd_text isEqualType "") } || { _skd_text isEqualTo "" }) exitWith {
    ["skd_fnc_expectError", 1, "text", "a String that is not empty", if (isNil "_skd_text") then { nil } else { _skd_text }] call skd_fnc_argError;
};
if (isNil "_skd_code" || { !(_skd_code isEqualType {}) }) exitWith {
    ["skd_fnc_expectError", 2, "code", "Code", if (isNil "_skd_code") then { nil } else { _skd_code }] call skd_fnc_argError;
};
if (isNil "_skd_args") then { _skd_args = [] };
if !(_skd_args isEqualType []) exitWith {
    ["skd_fnc_expectError", 3, "args", "an Array", _skd_args] call skd_fnc_argError;
};
(skd_check get "expects") pushBack [_skd_text, (call skd_fnc_clocks) get "utc", call skd_fnc_stepPath];
private _skd_result = switch (true) do { default { _skd_args call _skd_code } };
if (isNil "_skd_result") exitWith { nil };
_skd_result
