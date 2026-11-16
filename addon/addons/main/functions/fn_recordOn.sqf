/*
    Starts recording an event on this machine (EVT-01): adds a CBA event handler or a mission
    event handler that keeps each event with its arguments and clocks and sends it to the runner.
    An event recorded here already keeps what it recorded.

    Arguments: [event, kind] - kind "cba" or "mission".
    Returns: "" when it records, else why it cannot.
*/
params ["_event", "_kind"];
private _key = toLower _event;
private _known = skd_recorders getOrDefault [_key, []];
if (_known isNotEqualTo []) exitWith {
    if ((_known select 1) isEqualTo _kind) then { "" } else {
        format ["%1 records %2 already as a %3 event", skd_machine, _event, _known select 1]
    };
};
private _code = compile format ["[%1, _this] call skd_fnc_recorded", str _key];
private _id = -1;
if (_kind isEqualTo "cba") then {
    if (isNil "CBA_fnc_addEventHandler") exitWith {};
    _id = [_event, _code] call CBA_fnc_addEventHandler;
} else {
    _id = addMissionEventHandler [_event, _code];
};
if (isNil "_id" || { !(_id isEqualType 0) } || { _id < 0 }) exitWith {
    if (_kind isEqualTo "cba") then {
        format ["CBA is not loaded on %1, so it has no CBA event %2", skd_machine, _event]
    } else {
        format ["the engine has no mission event %1 (on %2)", _event, skd_machine]
    };
};
skd_recorders set [_key, [_event, _kind, _id, []]];
""
