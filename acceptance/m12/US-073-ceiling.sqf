// @check US-073 a wait on p2 for a signal only p1 raised locally ends fail wait-ceiling after 3 s with what each machine recorded
// @tags US-073 acceptance
// @setups ded

[["p1", "p2"], "skdfx_signalled"] call skd_fnc_record;

["p1 raises the signal locally", {
    ["p1", { ["skdfx_signalled", [player]] call CBA_fnc_localEvent; true }] call skd_fnc_on;
    ["p1", "skdfx_signalled", 10] call skd_fnc_waitEvent;
}] call skd_fnc_step;

["p2 waits 3 s for it", {
    ["p2", "skdfx_signalled", 3] call skd_fnc_waitEvent;
}] call skd_fnc_step;

["the step after the wait", { true }] call skd_fnc_step;
