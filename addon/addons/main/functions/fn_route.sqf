/*
    Sends an order to another machine through the runner and waits for its answer (P-03): the
    runner sends it to the target and answers with the target's answer, or with state "lost" when
    the target did not answer within the order's ceiling plus the grace (OBS-03).

    Arguments: [machine, order, ceiling] - order a HashMap with "kind" and its fields; ceiling the
    order's in-game ceiling in seconds.
    Returns: the answer, a HashMap with "state".
*/
params ["_machine", "_order", "_ceiling"];
skd_routeCount = skd_routeCount + 1;
private _ref = format ["%1-%2", skd_machine, skd_routeCount];
private _box = [];
skd_routes set [_ref, _box];
[createHashMapFromArray [["kind", "route"], ["ref", _ref], ["target", _machine], ["order", _order]]] call skd_fnc_send;
// The runner answers within the ceiling plus its grace of 10 s; this wait is the last resort.
private _start = diag_tickTime;
waitUntil { count _box > 0 || { diag_tickTime - _start > _ceiling + 15 } };
skd_routes deleteAt _ref;
if (count _box == 0) exitWith {
    createHashMapFromArray [["state", "lost"], ["message", format ["the runner sent no answer from %1 within %2 s", _machine, _ceiling + 15]]]
};
_box select 0
