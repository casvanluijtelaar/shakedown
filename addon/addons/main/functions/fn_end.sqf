/*
    Ends the running check with an outcome, once: the first cause decides. Records the reason
    against the innermost running step, stops the body, and stops the calling script if it is
    another one, so a function that fails does not return (check API, "Failure ends the check").

    Arguments: [outcome, kind, message, observed, machine] - observed is a HashMap from machine to
    value, or nil; machine the machine the reason concerns, by default this one.
*/
params ["_outcome", "_kind", "_message", ["_observed", nil], ["_machine", skd_machine]];
if (isNil "skd_check" || { skd_check get "done" }) exitWith {
    diag_log text format ["[shakedown] %1 / %2 outside a running check: %3", _outcome, _kind, _message];
};
if (isNil { skd_check get "end" }) then {
    private _clocks = call skd_fnc_clocks;
    private _steps = skd_check get "steps";
    private _stack = skd_check get "stack";
    private _path = nil;
    if (count _stack > 0) then {
        {
            private _step = _steps select _x;
            _step set ["state", "failed"];
            _step set ["end", _clocks];
            _step set ["t1", diag_tickTime];
        } forEach _stack;
        private _innermost = _steps select (_stack select -1);
        _innermost set ["error", _message];
        _path = _innermost get "path";
    };
    skd_check set ["end", [_outcome, _kind, _message, _clocks get "utc", _observed, _path, _machine]];
};
private _body = skd_check getOrDefault ["body", scriptNull];
if (!isNull _body && { !scriptDone _body }) then { terminate _body };
if (canSuspend && { !isNil "_thisScript" }) then {
    terminate _thisScript;
    waitUntil { false };
};
