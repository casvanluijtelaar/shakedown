/*
    A join or a leave asked in a check (MCH-01), as a step of its own titled "join <machine>" or
    "leave <machine>": checks the arguments, asks the runner, which starts or ends the machine's
    process, and takes the session's machines from its answer, so every later function names the
    machine as it now is. The runner refuses a machine that cannot join or leave (fail /
    check-error, DM-04); a join that is not ready in time or a leave the server did not see ends
    the check invalid / machine-lost, with the machine's last RPT lines in the evidence (DM-05).

    Arguments: [function, verb, machine, ceiling] - verb "join" or "leave".
    Returns: true.
*/
params ["_function", "_verb", "_machine", "_ceiling"];
if ((skd_check get "mode") isEqualTo "singleplayer") exitWith {
    ["fail", "check-error", format ["%1: joining and leaving needs a dedicated session; this session is singleplayer. Give the check @setups with a dedicated setup.", _function]] call skd_fnc_end;
};
if (isNil "_machine" || { !(_machine isEqualType "") }) exitWith {
    [_function, 1, "machine", "a machine name (String)", if (isNil "_machine") then { nil } else { _machine }] call skd_fnc_argError;
};
if (isNil "_ceiling") then { _ceiling = [20, skd_check get "startTimeout"] select (_verb isEqualTo "join") };
_ceiling = [_function, 2, _ceiling] call skd_fnc_ceiling;
if (isNil "_ceiling") exitWith {};

[format ["%1 %2", _verb, _machine], {
    params ["_function", "_verb", "_machine", "_ceiling"];
    skd_routeCount = skd_routeCount + 1;
    private _ref = format ["%1-%2", skd_machine, skd_routeCount];
    private _box = [];
    skd_routes set [_ref, _box];
    [createHashMapFromArray [["kind", _verb], ["ref", _ref], ["machine", _machine], ["ceiling", _ceiling]]] call skd_fnc_send;
    // The runner answers within the ceiling and the few seconds a join needs after it; this
    // wait is the last resort.
    private _start = diag_tickTime;
    waitUntil { count _box > 0 || { diag_tickTime - _start > _ceiling + 60 } };
    skd_routes deleteAt _ref;
    if (count _box == 0) exitWith {
        ["invalid", "machine-lost", format ["%1: the runner sent no answer about %2 within %3 s.", _function, _machine, _ceiling + 60], nil, _machine] call skd_fnc_end;
    };
    private _answer = _box select 0;
    private _machines = _answer get "machines";
    if (!isNil "_machines") then { skd_check set ["machines", _machines] };
    switch (_answer getOrDefault ["state", "lost"]) do {
        case "done": { true };
        case "refused": {
            ["fail", "check-error", _answer getOrDefault ["message", format ["%1: %2 cannot %3.", _function, _machine, _verb]]] call skd_fnc_end;
        };
        default {
            private _tail = _answer getOrDefault ["rpt_tail", []];
            if (_tail isNotEqualTo []) then {
                (skd_check get "attached") pushBack [format ["the last 20 lines of %1's RPT", _machine], _tail];
            };
            ["invalid", "machine-lost", format ["%1: %2. The environment failed, not the mod; the session goes on without %3.", _function, _answer getOrDefault ["message", "no answer"], _machine], nil, _machine] call skd_fnc_end;
        };
    };
}, [_function, _verb, _machine, _ceiling]] call skd_fnc_step
