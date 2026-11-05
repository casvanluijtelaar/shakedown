/*
    Starts reading a value on a machine every frame (OBS-01, OBS-06): the code runs in a script of
    its own on that machine, with its arguments as _this, until it is told to stop. In singleplayer
    every machine is the lead, so the script runs here.

    Arguments: [machine, compiled code, args]
    Returns: the poller [last value ([] for nothing, else [value]), reads, stop flag, script].
*/
params ["_machine", "_code", "_args"];
private _poller = [[], 0, false, scriptNull];
private _script = [[_args] call skd_fnc_cross, _code, _poller] spawn {
    params ["_skd_args", "_skd_code", "_skd_poller"];
    private _skd_remote = true;
    while { !(_skd_poller select 2) } do {
        private _skd_value = _skd_args call _skd_code;
        _skd_poller set [0, if (isNil "_skd_value") then { [] } else { [[_skd_value] call skd_fnc_cross] }];
        _skd_poller set [1, (_skd_poller select 1) + 1];
        private _skd_frame = diag_frameNo;
        waitUntil { diag_frameNo > _skd_frame || { _skd_poller select 2 } };
    };
};
_poller set [3, _script];
_poller
