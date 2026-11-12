// @check US-033 the second check of the walk-away suite waits about 20 s for a value the server sets later
// @tags US-033 US-034 acceptance
// @timeout 60

// skdfx_flag survives the reset, so the value awaited is one of this execution.
private _token = floor (random 1000000);
["server", { [_this select 0, 20] call skdfx_fnc_setFlagLater }, [_token]] call skd_fnc_on;
["wait for the flag the server sets 20 s later", {
    ["p1", { !isNil "skdfx_flag" && { skdfx_flag isEqualTo (_this select 0) } }, "the flag reached p1", 40, [_this select 0]] call skd_fnc_waitUntil
}, [_token]] call skd_fnc_step;
