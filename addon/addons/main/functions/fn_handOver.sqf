/*
    skd_fnc_handOver [machine, items] - hands the situation the check built over to a manual
    tester (MAN-01; check API, "Handing over to a manual tester"). Does not return.

    Shows the numbered items on the game client's screen, records the hand over for the end
    report, closes every step still running as passed and ends the body. The runner fixes the
    outcome as for a body that reached its end; a check that ends pass is handed over.
*/
if !(["skd_fnc_handOver"] call skd_fnc_guard) exitWith {};
params ["_skd_machine", "_skd_items"];
private _skd_targets = ["skd_fnc_handOver", 1, if (isNil "_skd_machine") then { nil } else { _skd_machine }, false] call skd_fnc_targets;
if (isNil "_skd_targets") exitWith {};
private _skd_target = _skd_targets select 0;
// Only a game client has a window a person looks through (US-084, DM-04).
if ((_skd_target select [0, 1]) isNotEqualTo "p") exitWith {
    private _clients = (skd_check get "machines") select { (_x select [0, 1]) isEqualTo "p" };
    ["fail", "check-error", format ["skd_fnc_handOver: %1 has no screen a person can look through. Hand over to a game client: %2.",
        _skd_target, _clients joinString ", "]] call skd_fnc_end;
};
if (isNil "_skd_items" || { !(_skd_items isEqualType []) } || { count _skd_items < 1 } || { count _skd_items > 10 }
    || { (_skd_items findIf { !(_x isEqualType "") || { _x isEqualTo "" } }) >= 0 }) exitWith {
    ["skd_fnc_handOver", 2, "items", "an Array of 1 to 10 Strings that are not empty", if (isNil "_skd_items") then { nil } else { _skd_items }] call skd_fnc_argError;
};

([_skd_target, { _this call skd_fnc_handOverShow }, [_skd_items], 10] call skd_fnc_runOn) params ["_skd_done", "_skd_value"];
if (!_skd_done) exitWith {
    call skd_fnc_atCeiling;
    ["fail", "wait-ceiling", format ["skd_fnc_handOver: the list was not on %1's screen within 10 s.", _skd_target]] call skd_fnc_end;
};
if !(_skd_value isEqualTo [true]) exitWith {
    ["fail", "check-error", format ["skd_fnc_handOver: %1 has no game window to show the list in.", _skd_target]] call skd_fnc_end;
};

skd_check set ["handover", [_skd_target, +_skd_items]];
private _skd_clocks = call skd_fnc_clocks;
{
    private _skd_step = (skd_check get "steps") select _x;
    _skd_step set ["state", "passed"];
    _skd_step set ["end", _skd_clocks];
    _skd_step set ["t1", diag_tickTime];
} forEach (skd_check get "stack");
(skd_check get "stack") resize 0;
private _skd_body = skd_check getOrDefault ["body", scriptNull];
if (!isNull _skd_body && { !scriptDone _skd_body }) then { terminate _skd_body };
if (canSuspend && { !isNil "_thisScript" }) then {
    terminate _thisScript;
    waitUntil { false };
};
