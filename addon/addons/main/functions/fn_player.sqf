/*
    skd_fnc_player [machine] - the unit the player of that machine controls, as the lead machine
    sees it; objNull for a dedicated server and a headless client (check API, "Context").
*/
if !(["skd_fnc_player"] call skd_fnc_guard) exitWith {};
params ["_skd_machine"];
private _skd_targets = ["skd_fnc_player", 1, if (isNil "_skd_machine") then { nil } else { _skd_machine }, false] call skd_fnc_targets;
if (isNil "_skd_targets") exitWith {};
// In singleplayer the only machine is p1, whose player is this machine's.
player
