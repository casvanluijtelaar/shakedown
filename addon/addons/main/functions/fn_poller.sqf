/*
    Starts reading a value on a machine every frame (OBS-01, OBS-06): the code runs in a script of
    its own on that machine, with its arguments as _this, until it is told to stop or its ceiling
    passes. On the lead machine, and in singleplayer, the script runs here; on another machine it
    runs there, and each new value comes back through the runner (P-03).

    Arguments: [machine, compiled code, args, ceiling]
    Returns: the poller [last value ([] for nothing, else [value]), reads, stop flag, script,
    reference of a poll on another machine or "", the script error the code raised or ""]. A read
    that raises a script error ends the poll with that error (OBS-01).
*/
params ["_machine", "_code", "_args", ["_ceiling", 60]];
private _poller = [[], 0, false, scriptNull, "", ""];
if (_machine isEqualTo skd_machine || { (skd_check get "mode") isEqualTo "singleplayer" }) exitWith {
    private _script = [[_args] call skd_fnc_cross, _code, _poller] spawn {
        params ["_skd_args", "_skd_code", "_skd_poller"];
        private _skd_remote = true;
        while { !(_skd_poller select 2) } do {
            private _skd_errors = skd_errorCount;
            private _skd_value = switch (true) do { default { _skd_args call _skd_code } };
            if (skd_errorCount > _skd_errors) then {
                _skd_poller set [5, skd_lastError];
                _skd_poller set [2, true];
            };
            _skd_poller set [0, if (isNil "_skd_value") then { [] } else { [[_skd_value] call skd_fnc_cross] }];
            _skd_poller set [1, (_skd_poller select 1) + 1];
            private _skd_frame = diag_frameNo;
            waitUntil { diag_frameNo > _skd_frame || { _skd_poller select 2 } };
        };
    };
    _poller set [3, _script];
    _poller
};
skd_routeCount = skd_routeCount + 1;
private _ref = format ["%1-%2", skd_machine, skd_routeCount];
_poller set [4, _ref];
skd_polls set [_ref, _poller];
[createHashMapFromArray [
    ["kind", "route"], ["ref", _ref], ["target", _machine],
    ["order", createHashMapFromArray [
        ["kind", "poll"], ["source", toString _code], ["args", [_args] call skd_fnc_encode], ["ceiling", _ceiling + 5],
        ["path", call skd_fnc_stepPath]
    ]]
]] call skd_fnc_send;
_poller
