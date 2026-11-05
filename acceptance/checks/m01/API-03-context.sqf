// @check API-03 The context functions give the singleplayer session's machines, setup and player
// @tags API-03 acceptance

["the machines", {
    [call skd_fnc_machines, "==", ["p1"], "the session's machines"] call skd_fnc_expect;
}] call skd_fnc_step;

["the setup", {
    private _setup = call skd_fnc_setup;
    [typeName _setup, "==", "HASHMAP", "the type skd_fnc_setup returned"] call skd_fnc_expect;
    [_setup get "name", "==", "sp", "the setup's name"] call skd_fnc_expect;
    [_setup get "mode", "==", "singleplayer", "the setup's mode"] call skd_fnc_expect;
    [_setup get "clients", "==", 1, "the setup's clients"] call skd_fnc_expect;
    [_setup get "headless", "==", 0, "the setup's headless clients"] call skd_fnc_expect;
    [_setup get "world", "==", "VR", "the setup's world"] call skd_fnc_expect;
}] call skd_fnc_step;

["the player", {
    private _unit = ["p1"] call skd_fnc_player;
    private _onP1 = ["p1", { player }] call skd_fnc_on;
    [_unit, "==", _onP1, "skd_fnc_player p1 against p1's own player"] call skd_fnc_expect;
    [isNull _unit, "==", false, "skd_fnc_player p1 is null"] call skd_fnc_expect;
}] call skd_fnc_step;
