// @check US-020 the server as the acting machine ends check-error saying server has no player
// @tags US-020 ACT-01 acceptance

private _crate = ["put the crate down", {
    ["skdfx_Crate", AGLToASL [250, 202, 0], 0] call skd_fnc_create
}] call skd_fnc_step;

["server uses Open crate", {
    ["server", _this select 0, "Open crate"] call skd_fnc_useAction;
}, [_crate]] call skd_fnc_step;

["never runs", {}] call skd_fnc_step;
