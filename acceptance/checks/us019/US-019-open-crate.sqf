// @check US-019 p1 opens the fixture's crate through its action and the server and every client see one more first aid kit
// @tags US-019 ACT-01 OBS-06 CTL-01 acceptance

/* One file for singleplayer and a dedicated session: the machines that see the kit are every
   machine but the headless clients, which is ["p1"] in singleplayer and server, p1, p2 on a
   dedicated session. */

private _p1 = ["p1"] call skd_fnc_player;
private _on = (call skd_fnc_machines) select { (_x select [0, 2]) != "hc" };
private _kits = { { _x == "FirstAidKit" } count items (_this select 0) };

private _crate = ["put the crate 2 m in front of p1", {
    ["p1", AGLToASL [200, 200, 0], 0] call skd_fnc_place;
    ["skdfx_Crate", AGLToASL [200, 202, 0], 0] call skd_fnc_create
}] call skd_fnc_step;

private _before = ["server", _kits, [_p1]] call skd_fnc_on;
["kits before", _before] call skd_fnc_evidence;

["p1 opens the crate", {
    params ["_crate"];
    private _used = ["p1", _crate, "Open crate"] call skd_fnc_useAction;
    [_used, "==", true, "the value skd_fnc_useAction returned"] call skd_fnc_expect;
}, [_crate]] call skd_fnc_step;

["every machine with an interface and the server see one more kit on p1", {
    params ["_on", "_kits", "_before", "_p1"];
    [_on, _kits, "==", _before + 1, "p1's first aid kits", 10, [_p1]] call skd_fnc_expectEach;
}, [_on, _kits, _before, _p1]] call skd_fnc_step;
