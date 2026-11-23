// @check US-010 a passing check registers a cleanup that never ends
// @tags US-010 acceptance
// @setups sp

// The cleanup runs at the start of the next reset and never ends there.
[{
    if (canSuspend) then {
        waitUntil { uiSleep 0.2; false };
    } else {
        private _until = diag_tickTime + 60;
        for "_i" from 0 to 1e9 do { if (diag_tickTime >= _until) exitWith {} };
    };
}] call skd_fnc_cleanup;
["the check itself passes", {
    [1, "==", 1, "one"] call skd_fnc_expect;
}] call skd_fnc_step;
