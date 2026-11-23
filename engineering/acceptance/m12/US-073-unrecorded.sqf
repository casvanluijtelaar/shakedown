// @check US-073 a wait on p2 for an event the check records only on p1 ends fail check-error
// @tags US-073 acceptance
// @setups ded

["p1", "skdfx_signalled"] call skd_fnc_record;

["p2 waits for an event it does not record", {
    ["p2", "skdfx_signalled", 3] call skd_fnc_waitEvent;
}] call skd_fnc_step;

["the step after the wait", { true }] call skd_fnc_step;
