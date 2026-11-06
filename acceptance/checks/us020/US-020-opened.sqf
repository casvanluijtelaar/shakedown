// @check US-020 an already opened crate cannot be opened again and the action's condition is named false
// @tags US-020 ACT-01 acceptance

private _crate = ["put the crate 2 m in front of p1 and open it", {
    ["p1", AGLToASL [240, 200, 0], 0] call skd_fnc_place;
    private _crate = ["skdfx_Crate", AGLToASL [240, 202, 0], 0] call skd_fnc_create;
    ["p1", _crate, "Open crate"] call skd_fnc_useAction;
    ["p1", { (_this select 0) getVariable ["skdfx_open", false] }, "p1 sees the crate open", 10, [_crate]] call skd_fnc_waitUntil;
    _crate
}] call skd_fnc_step;

["p1 opens it again", {
    ["p1", _this select 0, "Open crate", 3] call skd_fnc_useAction;
}, [_crate]] call skd_fnc_step;

["never runs", {}] call skd_fnc_step;
