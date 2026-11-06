/*
    The part of skd_fnc_useAction that runs on the acting machine (ACT-01): waits, every frame to
    the ceiling, until the action on the target exists, its condition holds and this machine's
    player is within its radius, then fires it with action ["User", target, id]. The target comes
    by net id and is looked up every frame, so an object the server just created is found once it
    reaches this machine.

    Arguments: [target net id, title, ceiling]
    Returns: ["fired"], ["no-player"] or the last state of skd_fnc_actionState.
*/
params ["_netId", "_title", "_ceiling"];
if (!hasInterface || { isNull player }) exitWith { ["no-player"] };
private _start = diag_tickTime;
private _state = [];
private _target = objNull;
waitUntil {
    _target = objectFromNetId _netId;
    _state = if (isNull _target) then { ["missing", []] } else { [_target, _title] call skd_fnc_actionState };
    (_state select 0) isEqualTo "ready" || { diag_tickTime - _start > _ceiling }
};
if ((_state select 0) isEqualTo "ready") exitWith {
    private _id = _state select 1;
    isNil { player action ["User", _target, _id] };
    ["fired"]
};
_state
