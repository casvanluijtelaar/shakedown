// @check US-044 a check that waits about 15 s for a value the server sets later, asked about while it runs
// @tags US-044 acceptance
// @setups ded
// @timeout 60

// skdfx_flag survives the reset, so the value awaited is one of this execution.
private _token = floor (random 1000000);
["server", { [_this select 0, 15] call skdfx_fnc_setFlagLater }, [_token]] call skd_fnc_on;
["wait for the flag the server sets 15 s later", {
    ["p1", { !isNil "skdfx_flag" && { skdfx_flag isEqualTo (_this select 0) } }, "the flag reached p1", 40, [_this select 0]] call skd_fnc_waitUntil
}, [_token]] call skd_fnc_step;
