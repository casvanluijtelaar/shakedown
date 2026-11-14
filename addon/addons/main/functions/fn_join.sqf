/*
    skd_fnc_join [machine, ceiling] - starts a late client, or a client that left, and returns
    true once it is ready; ceiling default the setup's start_timeout (check API, "Joining and
    leaving"; MCH-01).
*/
if !(["skd_fnc_join"] call skd_fnc_guard) exitWith {};
params ["_skd_machine", "_skd_ceiling"];
["skd_fnc_join", "join",
    if (isNil "_skd_machine") then { nil } else { _skd_machine },
    if (isNil "_skd_ceiling") then { nil } else { _skd_ceiling }] call skd_fnc_joinLeave
