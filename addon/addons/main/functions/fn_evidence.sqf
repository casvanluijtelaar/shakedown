/*
    skd_fnc_evidence [label, value] - adds a value to the check's evidence in the result file,
    kept for every outcome (check API, "Ending a check and keeping evidence").
*/
if !(["skd_fnc_evidence"] call skd_fnc_guard) exitWith {};
params ["_skd_label", "_skd_value"];
if (isNil "_skd_label" || { !(_skd_label isEqualType "") }) exitWith {
    ["skd_fnc_evidence", 1, "label", "a String", if (isNil "_skd_label") then { nil } else { _skd_label }] call skd_fnc_argError;
};
(skd_check get "attached") pushBack [_skd_label, if (isNil "_skd_value") then { nil } else { _skd_value }];
nil
