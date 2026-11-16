/*
    The events of one name the check recorded on a machine, read there (EVT-01). An event the
    check does not record on that machine ends it fail / check-error.

    Arguments: [function, machine, event]
    Returns: an array of HashMaps with "args" and "at" (clocks with numbers), or nothing when the
    check has ended.
*/
params ["_function", "_machine", "_event"];
private _where = (skd_check get "recorded") getOrDefault [toLower _event, []];
if !(_machine in _where) exitWith {
    ["fail", "check-error", format ["%1: the check does not record the event %2 on %3%4. Record it first with skd_fnc_record.",
        _function, _event, _machine,
        if (_where isEqualTo []) then { "" } else { format [" (it records it on %1)", _where joinString ", "] }], nil, _machine] call skd_fnc_end;
    nil
};
([_machine, { _this call skd_fnc_eventsOn }, [_event], 10] call skd_fnc_runOn) params ["_done", "_value"];
if (!_done) exitWith {
    ["fail", "wait-ceiling", format ["%1: %2 did not give its events of %3 within 10 s.", _function, _machine, _event], nil, _machine] call skd_fnc_end;
    nil
};
private _events = (_value select 0) select 1;
_events apply {
    _x params ["_args", "_at"];
    createHashMapFromArray [
        ["args", _args],
        ["at", createHashMapFromArray [
            ["utc", _at get "utc"],
            ["tick", parseNumber (_at get "tick")],
            ["time", parseNumber (_at get "time")],
            ["frame", parseNumber (_at get "frame")]
        ]]
    ]
}
