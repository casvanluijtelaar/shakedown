// @check US-020 p1 10 m from the crate cannot use its action and is told the distance and the radius
// @tags US-020 ACT-01 acceptance

private _crate = ["put the crate 10 m in front of p1", {
    ["p1", AGLToASL [230, 200, 0], 0] call skd_fnc_place;
    ["skdfx_Crate", AGLToASL [230, 210, 0], 0] call skd_fnc_create
}] call skd_fnc_step;

["p1 uses Open crate from 10 m", {
    ["p1", _this select 0, "Open crate", 3] call skd_fnc_useAction;
}, [_crate]] call skd_fnc_step;

["never runs", {}] call skd_fnc_step;
