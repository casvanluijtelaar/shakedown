// @check US-058 a check that waits about 15 s for a value the server sets later, the run a reload is asked during
// @tags US-058 acceptance
// @setups rld7 rld7sp
// @timeout 60

// skdfx_flag survives the reset, so the value is one of this execution.
private _token = floor (random 1000000);
["token", _token] call skd_fnc_evidence;
["server", { [_this select 0, 15] call skdfx_fnc_setFlagLater }, [_token]] call skd_fnc_on;
private _ok = ["wait for the flag the server sets 15 s later", {
    ["p1", { !isNil "skdfx_flag" && { skdfx_flag isEqualTo (_this select 0) } }, "the flag reached p1", 40, [_this select 0]] call skd_fnc_waitUntil
}, [_token]] call skd_fnc_step;
[_ok, "==", true, "waitUntil returned true"] call skd_fnc_expect;
