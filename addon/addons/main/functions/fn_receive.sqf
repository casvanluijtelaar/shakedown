/*
    Fetches an order from the extension in parts and runs it (INT-AR-03, P-03).

    Arguments: [orderId]
*/
params ["_id"];
private _parts = [];
private _part = 0;
private _last = false;
private _failed = false;
while {!_last && !_failed} do {
    private _answer = "shakedown" callExtension ["fetch", [_id, _part]];
    private _text = _answer select 0;
    if ((_answer select 1) != 0) then {
        diag_log text format ["[shakedown] fetch of order %1 part %2 failed: %3", _id, _part, _text];
        _failed = true;
    } else {
        _last = (_text select [0, 1]) isEqualTo "1";
        _parts pushBack (_text select [1]);
        _part = _part + 1;
    };
};
if (_failed) exitWith {};

private _order = fromJSON (_parts joinString "");
switch (_order get "kind") do {
    case "exec": { [_order] call skd_fnc_runExec };
    case "check": { [_order] call skd_fnc_runCheck };
    case "poll": { [_order] call skd_fnc_runPoll };
    // The runner's request for an exec order went away before its answer (AGT-04), or the run
    // of a check order was stopped (RUN-05): its code stops here.
    case "cancel": {
        private _script = skd_execs getOrDefault [_order get "order", scriptNull];
        if (!isNull _script) then { terminate _script };
    };
    // A poll this machine runs for another is told to stop.
    case "stop": {
        private _flag = skd_stops getOrDefault [_order get "poll", []];
        if (count _flag > 0) then { _flag set [0, true] };
    };
    // The answer to an order this machine routed to another.
    case "routed": {
        private _box = skd_routes getOrDefault [_order get "ref", []];
        private _answer = _order get "answer";
        if (isNil "_answer") then { _answer = createHashMap };
        _box pushBack _answer;
    };
    // A value a poll on another machine read for this one; a newer read replaces an older one.
    case "routed-poll": {
        private _poller = skd_polls getOrDefault [_order get "ref", []];
        private _reads = _order getOrDefault ["reads", 0];
        private _error = _order getOrDefault ["error", ""];
        if (count _poller > 0 && { !isNil "_error" } && { _error isNotEqualTo "" }) then { _poller set [5, _error] };
        if (count _poller > 0 && { _reads > (_poller select 1) }) then {
            private _value = _order get "value";
            _poller set [0, if (isNil "_value") then { [] } else { [[_value] call skd_fnc_decode] }];
            _poller set [1, _reads];
        };
    };
    default {
        diag_log text format ["[shakedown] unknown order kind %1", _order get "kind"];
    };
};
