/*
    skd_fnc_on [machine, code, args, ceiling] - runs code on a machine and returns its value
    (check API, "Running code on machines").

    The code is sent as text and compiled on the target, so it does not see the check's local
    variables; args arrive as _this. It runs scheduled with an in-game ceiling. On the lead machine
    itself it runs locally, without the round trip (P-03 step 5).

    Returns: the value code returned.
*/
if !(["skd_fnc_on"] call skd_fnc_guard) exitWith {};
params ["_skd_machine", "_skd_code", ["_skd_args", []], ["_skd_ceiling", 10]];
if (isNil "_skd_machine" || { !(_skd_machine isEqualType "") }) exitWith {
    ["skd_fnc_on", 1, "machine", "a machine name (String)", if (isNil "_skd_machine") then { nil } else { _skd_machine }] call skd_fnc_argError;
};
if (isNil "_skd_code" || { !(_skd_code isEqualType {}) }) exitWith {
    ["skd_fnc_on", 2, "code", "Code", if (isNil "_skd_code") then { nil } else { _skd_code }] call skd_fnc_argError;
};
if (isNil "_skd_args") then { _skd_args = [] };
if !(_skd_args isEqualType []) exitWith {
    ["skd_fnc_on", 3, "args", "an Array", _skd_args] call skd_fnc_argError;
};
if (isNil "_skd_ceiling") then { _skd_ceiling = 10 };
if (!(_skd_ceiling isEqualType 0) || { _skd_ceiling <= 0 }) exitWith {
    ["skd_fnc_on", 4, "ceiling", "a number of seconds above 0", _skd_ceiling] call skd_fnc_argError;
};
private _skd_machines = skd_check get "machines";
// In singleplayer "server" names p1 too (DM-10).
private _skd_target = [_skd_machine, skd_machine] select (_skd_machine isEqualTo "server" && { (skd_check get "mode") isEqualTo "singleplayer" });
if !(_skd_target in _skd_machines) exitWith {
    ["fail", "check-error", format ["skd_fnc_on: the session has no machine %1; its machines are %2.", str _skd_machine, _skd_machines joinString ", "]] call skd_fnc_end;
};
_skd_ceiling = _skd_ceiling min (((skd_check get "deadline") - diag_tickTime) max 0.001);

skd_compileErrors = [];
skd_compiling = true;
private _skd_compiled = compile toString _skd_code;
skd_compiling = false;
if (skd_compileErrors isNotEqualTo []) exitWith {
    (skd_compileErrors select 0) params ["_skd_text", "_skd_line"];
    ["fail", "check-error", format ["skd_fnc_on: the code does not compile on %1: %2 (line %3 of the code).", _skd_target, _skd_text, _skd_line]] call skd_fnc_end;
};

private _skd_box = [nil];
private _skd_start = diag_tickTime;
private _skd_script = [_skd_args, _skd_compiled, _skd_box] spawn {
    params ["_skd_args", "_skd_code", "_skd_box"];
    private _skd_remote = true;
    _skd_box set [0, _skd_args call _skd_code];
};
waitUntil { scriptDone _skd_script || { diag_tickTime - _skd_start > _skd_ceiling } };
if (!scriptDone _skd_script) exitWith {
    terminate _skd_script;
    ["fail", "wait-ceiling", format ["skd_fnc_on: the code on %1 was still running at its ceiling of %2 s.", _skd_target, _skd_ceiling]] call skd_fnc_end;
};
_skd_box select 0
