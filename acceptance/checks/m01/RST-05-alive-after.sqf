// @check RST-05 The check after a player died finds p1 alive and unhurt at its start position with no body left
// @tags RST-05 acceptance

private _unit = ["p1"] call skd_fnc_player;
private _base = missionNamespace getVariable ["acc_m01_base", createHashMap];

["the start state is known", {
    [count (_this select 0) > 0, "==", true, "an NFR-RL-02 check recorded the start state before this one"] call skd_fnc_expect;
}, [_base]] call skd_fnc_step;

["p1 is alive, unhurt and on foot", {
    params ["_unit"];
    [alive _unit, "==", true, "p1 is alive"] call skd_fnc_expect;
    [damage _unit, "==", 0, "p1's damage"] call skd_fnc_expect;
    [vehicle _unit == _unit, "==", true, "p1 is on foot"] call skd_fnc_expect;
}, [_unit]] call skd_fnc_step;

["no dead body is left", {
    [count allDeadMen, "==", 0, "dead men in the mission"] call skd_fnc_expect;
}] call skd_fnc_step;

["p1 stands where the mission started it, with its start loadout", {
    params ["_unit", "_base"];
    [getPosASL _unit, "~=", [_base get "pos", 0.1], "p1's position against the start state"] call skd_fnc_expect;
    [getUnitLoadout _unit, "==", _base get "loadout", "p1's loadout against the start state"] call skd_fnc_expect;
}, [_unit, _base]] call skd_fnc_step;
