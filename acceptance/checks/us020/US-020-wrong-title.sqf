// @check US-020 an action title with the wrong case is not found and the titles on the crate are listed
// @tags US-020 ACT-01 acceptance

private _crate = ["put the crate 2 m in front of p1", {
    ["p1", AGLToASL [220, 200, 0], 0] call skd_fnc_place;
    ["skdfx_Crate", AGLToASL [220, 202, 0], 0] call skd_fnc_create
}] call skd_fnc_step;

["p1 uses Open Crate", {
    ["p1", _this select 0, "Open Crate", 3] call skd_fnc_useAction;
}, [_crate]] call skd_fnc_step;

["never runs", {}] call skd_fnc_step;
