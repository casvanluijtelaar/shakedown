/*
    skd_fnc_cleanup [code, args] - registers code to run on the lead machine at the start of the
    next reset, last registered first (check API; RST-01, P-06 step 1).
*/
if !(["skd_fnc_cleanup"] call skd_fnc_guard) exitWith {};
params ["_skd_code", "_skd_args"];
if (isNil "_skd_code" || { !(_skd_code isEqualType {}) }) exitWith {
    ["skd_fnc_cleanup", 1, "code", "Code", if (isNil "_skd_code") then { nil } else { _skd_code }] call skd_fnc_argError;
};
if (isNil "_skd_args") then { _skd_args = [] };
if !(_skd_args isEqualType []) exitWith { ["skd_fnc_cleanup", 2, "args", "an Array", _skd_args] call skd_fnc_argError };
skd_cleanups pushBack [_skd_code, _skd_args, skd_check get "id"];
nil
