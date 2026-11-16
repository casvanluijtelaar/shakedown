/*
    skd_fnc_record [machines, event, kind, ceiling] - records a CBA event or an engine mission
    event on each machine until the check ends (check API, "Recording events"; EVT-01).

    Returns: true once every machine records it.
*/
if !(["skd_fnc_record"] call skd_fnc_guard) exitWith {};
params ["_skd_machines", "_skd_event", "_skd_kind", "_skd_ceiling"];
private _skd_targets = ["skd_fnc_record", 1, if (isNil "_skd_machines") then { nil } else { _skd_machines }, true] call skd_fnc_targets;
if (isNil "_skd_targets") exitWith {};
if (isNil "_skd_event" || { !(_skd_event isEqualType "") } || { _skd_event isEqualTo "" }) exitWith {
    ["skd_fnc_record", 2, "event", "the event's name, a String that is not empty", if (isNil "_skd_event") then { nil } else { _skd_event }] call skd_fnc_argError;
};
if (isNil "_skd_kind") then { _skd_kind = "cba" };
if !(_skd_kind in ["cba", "mission"]) exitWith {
    ["skd_fnc_record", 3, "kind", """cba"" or ""mission""", _skd_kind] call skd_fnc_argError;
};
_skd_ceiling = ["skd_fnc_record", 4, if (isNil "_skd_ceiling") then { nil } else { _skd_ceiling }] call skd_fnc_ceiling;
if (isNil "_skd_ceiling") exitWith {};

private _skd_start = diag_tickTime;
private _skd_late = [];
{
    private _skd_left = (_skd_ceiling - (diag_tickTime - _skd_start)) max 0.001;
    ([_x, { _this call skd_fnc_recordOn }, [_skd_event, _skd_kind], _skd_left] call skd_fnc_runOn) params ["_skd_done", "_skd_value"];
    if (!_skd_done) then { _skd_late pushBack _x } else {
        private _skd_why = if (_skd_value isEqualTo []) then { "" } else { _skd_value select 0 };
        if (_skd_why isNotEqualTo "") then {
            ["fail", "check-error", format ["skd_fnc_record: %1.", _skd_why], nil, _x] call skd_fnc_end;
        };
        // The lead keeps where the check records each event, for skd_fnc_events and
        // skd_fnc_waitEvent.
        private _skd_where = (skd_check get "recorded") getOrDefault [toLower _skd_event, []];
        _skd_where pushBackUnique _x;
        (skd_check get "recorded") set [toLower _skd_event, _skd_where];
    };
} forEach _skd_targets;
if (_skd_late isNotEqualTo []) exitWith {
    call skd_fnc_atCeiling;
    ["fail", "wait-ceiling", format ["skd_fnc_record: %1 did not start recording %2 within the ceiling of %3 s.", _skd_late joinString ", ", _skd_event, _skd_ceiling]] call skd_fnc_end;
};
true
