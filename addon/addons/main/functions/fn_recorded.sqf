/*
    Keeps one event a recording on this machine received (EVT-01), with its clocks (OBS-05), and
    sends it to the runner, stamped with the check execution it came in, for the result file.
    Runs in the event's handler, unscheduled.

    Arguments: [key, args] - key the event's name in lower case; args the event's arguments.
*/
params ["_key", "_args"];
private _recorder = skd_recorders getOrDefault [_key, []];
if (_recorder isEqualTo []) exitWith {};
if (isNil "_args") then { _args = [] };
if (_args isEqualType []) then { _args = +_args };
private _at = call skd_fnc_clocks;
(_recorder select 3) pushBack [_args, _at];
[createHashMapFromArray [
    ["kind", "event"],
    ["machine", skd_machine],
    ["key", skd_context select 0],
    ["name", _recorder select 0],
    ["type", _recorder select 1],
    ["args", [_args] call skd_fnc_encode],
    ["at", _at]
]] call skd_fnc_send;
