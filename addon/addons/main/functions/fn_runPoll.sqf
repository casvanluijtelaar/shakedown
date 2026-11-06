/*
    Runs an order of kind poll for another machine (OBS-01, OBS-06): evaluates the code every frame
    with the arguments as _this, and sends each value that differs from the last one sent, at most
    one every 0.05 s, until told to stop or until its ceiling passes. Then answers the order.

    Arguments: [order] - HashMap with "id", "source", "args", "ceiling".
*/
params ["_order"];
private _id = _order get "id";
private _ceiling = _order getOrDefault ["ceiling", 10];
private _args = _order getOrDefault ["args", []];
if (isNil "_args") then { _args = [] };
_args = [_args] call skd_fnc_decode;

skd_compileErrors = [];
skd_compiling = true;
private _code = compile (_order get "source");
skd_compiling = false;
if (skd_compileErrors isNotEqualTo []) exitWith {
    (skd_compileErrors select 0) params ["_text", "_line"];
    [createHashMapFromArray [
        ["kind", "result"], ["order", _id], ["state", "compile-error"],
        ["error", format ["%1 (line %2)", _text, _line]]
    ]] call skd_fnc_send;
};

private _stop = [false];
skd_stops set [_id, _stop];
private _script = [_args, _code, _id, _stop, _ceiling] spawn {
    params ["_skd_args", "_skd_code", "_skd_id", "_skd_stop", "_skd_ceiling"];
    private _skd_remote = true;
    private _skd_start = diag_tickTime;
    private _skd_sent = "";
    private _skd_sentAt = -1;
    private _skd_reads = 0;
    while { !(_skd_stop select 0) && { diag_tickTime - _skd_start < _skd_ceiling } } do {
        private _skd_value = [_skd_args call _skd_code] call skd_fnc_encode;
        private _skd_text = if (isNil "_skd_value") then { "nil" } else { toJSON [_skd_value] };
        if (_skd_text isNotEqualTo _skd_sent && { diag_tickTime - _skd_sentAt >= 0.05 }) then {
            _skd_reads = _skd_reads + 1;
            private _skd_message = createHashMapFromArray [["kind", "poll"], ["order", _skd_id], ["reads", _skd_reads]];
            if (!isNil "_skd_value") then { _skd_message set ["value", _skd_value] };
            [_skd_message] call skd_fnc_send;
            _skd_sent = _skd_text;
            _skd_sentAt = diag_tickTime;
        };
        private _skd_frame = diag_frameNo;
        waitUntil { diag_frameNo > _skd_frame || { _skd_stop select 0 } };
    };
};
waitUntil { scriptDone _script };
skd_stops deleteAt _id;
[createHashMapFromArray [["kind", "result"], ["order", _id], ["state", "done"]]] call skd_fnc_send;
