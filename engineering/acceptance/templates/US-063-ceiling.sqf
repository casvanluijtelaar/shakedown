// @check US-063 RUN-06 ceiling check {N} of 8, timeout 1 s, registers a cleanup that runs 14 s of real time
// @tags US-063 acceptance
// @timeout 1

// The cleanup runs at the start of the next reset. It takes 14 s of real time whether it runs
// scheduled or not, so eight of them outlast the run's ceiling of 8 x 1 s + 60 s.
[{
    private _until = diag_tickTime + 14;
    if (canSuspend) then {
        waitUntil { diag_tickTime >= _until };
    } else {
        for "_i" from 0 to 1e9 do { if (diag_tickTime >= _until) exitWith {} };
    };
}] call skd_fnc_cleanup;
["the check itself ends at once", {
    [1, "==", 1, "one"] call skd_fnc_expect;
}] call skd_fnc_step;
