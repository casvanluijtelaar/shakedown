/*
    skd_fnc_events [machine, event] - the events of that name the check recorded on the machine,
    oldest first (check API, "Recording events"; EVT-01).

    Returns: an array of HashMaps with "args" and "at".
*/
if !(["skd_fnc_events"] call skd_fnc_guard) exitWith {};
params ["_skd_machine", "_skd_event"];
private _skd_targets = ["skd_fnc_events", 1, if (isNil "_skd_machine") then { nil } else { _skd_machine }, false] call skd_fnc_targets;
if (isNil "_skd_targets") exitWith {};
if (isNil "_skd_event" || { !(_skd_event isEqualType "") } || { _skd_event isEqualTo "" }) exitWith {
    ["skd_fnc_events", 2, "event", "the event's name, a String that is not empty", if (isNil "_skd_event") then { nil } else { _skd_event }] call skd_fnc_argError;
};
private _skd_list = ["skd_fnc_events", _skd_targets select 0, _skd_event] call skd_fnc_eventList;
if (isNil "_skd_list") exitWith {};
_skd_list
