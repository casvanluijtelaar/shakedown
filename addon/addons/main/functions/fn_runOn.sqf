/*
    Runs code on a machine, scheduled, with its arguments as _this and an in-game ceiling (CTL-01
    to CTL-03, P-03). In singleplayer every machine is the lead, so the code runs here, in a script
    of its own, and its arguments and result cross the channel's encoding (P-03 step 5).

    Arguments: [machine, compiled code, args, ceiling]
    Returns: [done, value] - done is false when the code was still running at the ceiling; value
    is [] for nothing, else [the value].
*/
params ["_machine", "_code", "_args", "_ceiling"];
private _box = [false, []];
private _start = diag_tickTime;
private _script = [[_args] call skd_fnc_cross, _code, _box] spawn {
    params ["_skd_args", "_skd_code", "_skd_box"];
    private _skd_remote = true;
    private _skd_value = _skd_args call _skd_code;
    _skd_box set [1, if (isNil "_skd_value") then { [] } else { [[_skd_value] call skd_fnc_cross] }];
    _skd_box set [0, true];
};
waitUntil { scriptDone _script || { diag_tickTime - _start > _ceiling } };
if (!scriptDone _script) then { terminate _script };
[_box select 0, _box select 1]
