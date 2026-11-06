/*
    Stops a poller started by skd_fnc_poller, here or on the machine it reads.

    Arguments: [poller]
*/
params ["_poller"];
_poller set [2, true];
private _script = _poller select 3;
if (!isNull _script && { !scriptDone _script }) then { terminate _script };
private _ref = _poller param [4, ""];
if (_ref isNotEqualTo "") then {
    skd_polls deleteAt _ref;
    [createHashMapFromArray [["kind", "route-stop"], ["ref", _ref]]] call skd_fnc_send;
};
