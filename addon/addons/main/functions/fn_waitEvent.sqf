/*
    skd_fnc_waitEvent [machine, event, ceiling, after] - waits until the machine has recorded more
    than `after` events of that name and returns the arguments of the next one (check API,
    "Recording events"; EVT-01, OBS-04).

    Returns: the event's arguments as the machine received them.
*/
if !(["skd_fnc_waitEvent"] call skd_fnc_guard) exitWith {};
params ["_skd_machine", "_skd_event", "_skd_ceiling", "_skd_after"];
private _skd_targets = ["skd_fnc_waitEvent", 1, if (isNil "_skd_machine") then { nil } else { _skd_machine }, false] call skd_fnc_targets;
if (isNil "_skd_targets") exitWith {};
if (isNil "_skd_event" || { !(_skd_event isEqualType "") } || { _skd_event isEqualTo "" }) exitWith {
    ["skd_fnc_waitEvent", 2, "event", "the event's name, a String that is not empty", if (isNil "_skd_event") then { nil } else { _skd_event }] call skd_fnc_argError;
};
_skd_ceiling = ["skd_fnc_waitEvent", 3, if (isNil "_skd_ceiling") then { nil } else { _skd_ceiling }] call skd_fnc_ceiling;
if (isNil "_skd_ceiling") exitWith {};
if (isNil "_skd_after") then { _skd_after = 0 };
if (!(_skd_after isEqualType 0) || { _skd_after < 0 } || { floor _skd_after != _skd_after }) exitWith {
    ["skd_fnc_waitEvent", 4, "after", "a whole number from 0", _skd_after] call skd_fnc_argError;
};
private _skd_target = _skd_targets select 0;
private _skd_where = (skd_check get "recorded") getOrDefault [toLower _skd_event, []];
if !(_skd_target in _skd_where) exitWith {
    ["skd_fnc_waitEvent", _skd_target, _skd_event] call skd_fnc_eventList;
};

private _skd_met = ["skd_fnc_waitEvent", true, [
    _skd_target,
    { count (((skd_recorders getOrDefault [_this select 0, ["", "", -1, []]]) select 3)) > (_this select 1) },
    format ["an event %1 on %2", _skd_event, _skd_target],
    _skd_ceiling,
    [toLower _skd_event, _skd_after]
]] call skd_fnc_wait;
if (isNil "_skd_met") exitWith {};
if (!_skd_met) exitWith {
    // What did arrive: the events of every name on the machine waited on and on every other
    // machine the check records an event on.
    private _skd_machines = [_skd_target];
    { { _skd_machines pushBackUnique _x } forEach _y } forEach (skd_check get "recorded");
    private _skd_observed = createHashMap;
    private _skd_lines = [];
    {
        private _skd_m = _x;
        ([_skd_m, { _this call skd_fnc_eventsOn }, [""], 5] call skd_fnc_runOn) params ["_skd_done", "_skd_value"];
        private _skd_counts = createHashMap;
        if (_skd_done && { _skd_value isNotEqualTo [] }) then {
            { _skd_counts set [_x select 0, _x select 1] } forEach (_skd_value select 0);
        };
        _skd_observed set [_skd_m, _skd_counts];
        private _skd_total = 0;
        { _skd_total = _skd_total + _y } forEach _skd_counts;
        _skd_lines pushBack format ["%1: %2", _skd_m,
            if (_skd_total == 0) then { "none" } else {
                ((_skd_counts toArray false) apply { format ["%1 %2", _x select 1, _x select 0] }) joinString ", "
            }];
    } forEach _skd_machines;
    ["fail", "wait-ceiling", format ["skd_fnc_waitEvent: no event %1 came on %2 within its ceiling of %3 s%4. Recorded events: %5.",
        _skd_event, _skd_target, _skd_ceiling,
        if (_skd_after > 0) then { format [" after the %1 it had", _skd_after] } else { "" },
        _skd_lines joinString "; "], _skd_observed, _skd_target] call skd_fnc_end;
};
private _skd_list = ["skd_fnc_waitEvent", _skd_target, _skd_event] call skd_fnc_eventList;
if (isNil "_skd_list") exitWith {};
(_skd_list select _skd_after) get "args"
