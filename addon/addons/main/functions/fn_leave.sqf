/*
    skd_fnc_leave [machine, ceiling] - makes a game client leave and returns true once the
    server's PlayerDisconnected has fired for it; ceiling default 20 s (check API, "Joining and
    leaving"; MCH-01).
*/
if !(["skd_fnc_leave"] call skd_fnc_guard) exitWith {};
params ["_skd_machine", "_skd_ceiling"];
["skd_fnc_leave", "leave",
    if (isNil "_skd_machine") then { nil } else { _skd_machine },
    if (isNil "_skd_ceiling") then { nil } else { _skd_ceiling }] call skd_fnc_joinLeave
