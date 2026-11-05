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
    default {
        diag_log text format ["[shakedown] unknown order kind %1", _order get "kind"];
    };
};
