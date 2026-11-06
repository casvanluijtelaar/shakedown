/*
    The unit the player of a machine controls, as this machine sees it: on a dedicated session the
    player whose profile name is the machine's name (DM-11). objNull for the server, a headless
    client, and a client with no unit.

    Arguments: [machine]
    Returns: the unit, or objNull.
*/
params ["_machine"];
if (!isMultiplayer) exitWith { player };
if !((_machine select [0, 1]) isEqualTo "p") exitWith { objNull };
private _i = allPlayers findIf { name _x isEqualTo _machine && { !(_x isKindOf "HeadlessClient_F") } };
if (_i < 0) exitWith { objNull };
allPlayers select _i
