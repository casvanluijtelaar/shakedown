/*
    skd_fnc_step [title, code, args] - a named, timed part of a check (check API, "Steps").

    code runs with call in the body's own script, so it sees the body's local variables; _this is
    args. Its path, its clocks (OBS-05) and its state are recorded. Every local variable here starts
    with _skd_ so the step's code never sees or changes one by accident.

    Returns: the value of code.
*/
if !(["skd_fnc_step"] call skd_fnc_guard) exitWith {};
params ["_skd_title", "_skd_code", ["_skd_args", []]];
if (isNil "_skd_title" || { !(_skd_title isEqualType "") }) exitWith {
    ["skd_fnc_step", 1, "title", "a String", if (isNil "_skd_title") then { nil } else { _skd_title }] call skd_fnc_argError;
};
if (isNil "_skd_code" || { !(_skd_code isEqualType {}) }) exitWith {
    ["skd_fnc_step", 2, "code", "Code", if (isNil "_skd_code") then { nil } else { _skd_code }] call skd_fnc_argError;
};
if (isNil "_skd_args") then { _skd_args = [] };
if !(_skd_args isEqualType []) exitWith {
    ["skd_fnc_step", 3, "args", "an Array", _skd_args] call skd_fnc_argError;
};

private _skd_steps = skd_check get "steps";
private _skd_stack = skd_check get "stack";
private _skd_parent = if (count _skd_stack > 0) then {
    (_skd_steps select (_skd_stack select -1)) get "path"
} else { [] };
private _skd_step = createHashMapFromArray [
    ["path", _skd_parent + [_skd_title]],
    ["state", "running"],
    ["start", call skd_fnc_clocks],
    ["t0", diag_tickTime]
];
_skd_stack pushBack (_skd_steps pushBack _skd_step);

// The value goes through a switch: code whose last statement is an assignment returns nothing,
// and assigning nothing raises "GIAS pre stack size violation"; the switch gives nil instead.
private _skd_result = switch (true) do { default { _skd_args call _skd_code } };

_skd_step set ["state", "passed"];
_skd_step set ["end", call skd_fnc_clocks];
_skd_step set ["t1", diag_tickTime];
_skd_stack deleteAt (count _skd_stack - 1);
if (isNil "_skd_result") exitWith { nil };
_skd_result
