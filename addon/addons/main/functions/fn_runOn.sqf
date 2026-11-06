/*
    Runs code on a machine, scheduled, with its arguments as _this and an in-game ceiling (CTL-01
    to CTL-03, P-03). Code for the lead machine itself runs here, in a script of its own, and its
    arguments and result cross the channel's encoding (P-03 step 5); in singleplayer every machine
    is the lead. Code for another machine goes there through the runner.

    Arguments: [machine, compiled code, args, ceiling]
    Returns: [done, value] - done is false when the code was still running at the ceiling; value
    is [] for nothing, else [the value]. A compile error on the target ends the check fail /
    check-error, and a machine that does not answer ends it invalid / machine-lost.
*/
params ["_machine", "_code", "_args", "_ceiling"];
if (_machine isEqualTo skd_machine || { (skd_check get "mode") isEqualTo "singleplayer" }) exitWith {
    private _box = [false, []];
    private _start = diag_tickTime;
    private _script = [[_args] call skd_fnc_cross, _code, _box] spawn {
        params ["_skd_args", "_skd_code", "_skd_box"];
        private _skd_remote = true;
        private _skd_value = switch (true) do { default { _skd_args call _skd_code } };
        _skd_box set [1, if (isNil "_skd_value") then { [] } else { [[_skd_value] call skd_fnc_cross] }];
        _skd_box set [0, true];
    };
    waitUntil { scriptDone _script || { diag_tickTime - _start > _ceiling } };
    if (!scriptDone _script) then { terminate _script };
    [_box select 0, _box select 1]
};
private _answer = [_machine, createHashMapFromArray [
    ["kind", "exec"], ["source", toString _code], ["args", [_args] call skd_fnc_encode], ["ceiling", _ceiling],
    ["path", call skd_fnc_stepPath]
], _ceiling] call skd_fnc_route;
switch (_answer getOrDefault ["state", "lost"]) do {
    case "done": {
        private _value = _answer get "value";
        [true, if (isNil "_value") then { [] } else { [[_value] call skd_fnc_decode] }]
    };
    case "ceiling": { [false, []] };
    case "compile-error": {
        ["fail", "check-error", format ["The code does not compile on %1: %2.", _machine, _answer getOrDefault ["error", ""]]] call skd_fnc_end;
        [false, []]
    };
    default {
        ["invalid", "machine-lost", format ["%1 did not answer: %2.", _machine, _answer getOrDefault ["message", "no answer"]], nil, _machine] call skd_fnc_end;
        [false, []]
    };
}
