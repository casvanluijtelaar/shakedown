/*
    Stops a poller started by skd_fnc_poller.

    Arguments: [poller]
*/
params ["_poller"];
_poller set [2, true];
private _script = _poller select 3;
if (!isNull _script && { !scriptDone _script }) then { terminate _script };
