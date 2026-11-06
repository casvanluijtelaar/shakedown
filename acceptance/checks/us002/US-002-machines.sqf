// @check US-002 skd_fnc_machines names server, p1, p2 and hc1, and server and hc1 have no player
// @tags US-002 DM-10 DM-11 API-03 acceptance

["the session's machines", {
    [call skd_fnc_machines, "==", ["server", "p1", "p2", "hc1"], "the machines skd_fnc_machines names"] call skd_fnc_expect;
    private _setup = call skd_fnc_setup;
    [_setup get "mode", "==", "dedicated", "the setup's mode"] call skd_fnc_expect;
    [_setup get "clients", "==", 2, "the setup's clients"] call skd_fnc_expect;
    [_setup get "headless", "==", 1, "the setup's headless clients"] call skd_fnc_expect;
}] call skd_fnc_step;

["server and hc1 have no player, p1 and p2 each have one", {
    [isNull (["hc1"] call skd_fnc_player), "==", true, "hc1's player is objNull"] call skd_fnc_expect;
    [isNull (["server"] call skd_fnc_player), "==", true, "server's player is objNull"] call skd_fnc_expect;
    [isNull player, "==", true, "player in the body on the server is objNull"] call skd_fnc_expect;
    private _p1 = ["p1"] call skd_fnc_player;
    private _p2 = ["p2"] call skd_fnc_player;
    [!isNull _p1 && { !isNull _p2 } && { _p1 != _p2 }, "==", true, "p1 and p2 are two units"] call skd_fnc_expect;
    ["p1", { player == (_this select 0) }, "==", true, "p1 controls the unit skd_fnc_player names for p1", 10, [_p1]] call skd_fnc_expectOn;
    ["p2", { player == (_this select 0) }, "==", true, "p2 controls the unit skd_fnc_player names for p2", 10, [_p2]] call skd_fnc_expectOn;
}] call skd_fnc_step;
