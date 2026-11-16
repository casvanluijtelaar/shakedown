/*
    Removes every handler the event recording added on this machine and forgets what it recorded
    (EVT-01), when a check execution starts and when it ends.
*/
{
    _y params ["_event", "_kind", "_id"];
    if (_kind isEqualTo "cba") then {
        if (!isNil "CBA_fnc_removeEventHandler") then { [_event, _id] call CBA_fnc_removeEventHandler };
    } else {
        removeMissionEventHandler [_event, _id];
    };
} forEach skd_recorders;
skd_recorders = createHashMap;
