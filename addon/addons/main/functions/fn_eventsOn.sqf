/*
    The events this machine recorded (EVT-01).

    Arguments: [event] - an event's name, or "" for every event recorded here.
    Returns: for one event, [recorded, [[args, clocks], ...]]; for "", [[name, count], ...] of each
    event recorded here.
*/
params ["_event"];
if (_event isEqualTo "") exitWith {
    (values skd_recorders) apply { [_x select 0, count (_x select 3)] }
};
private _recorder = skd_recorders getOrDefault [toLower _event, []];
if (_recorder isEqualTo []) exitWith { [false, []] };
[true, +(_recorder select 3)]
