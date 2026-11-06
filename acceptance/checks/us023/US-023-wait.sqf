// @check US-023 OBS-01 OBS-02 a wait for a flag the server sets 3 s later returns true on p1 in under 5 s
// @tags US-023 OBS-01 OBS-02 acceptance
// @setups ded

// skdfx_flag survives the reset, so the flag is a value of this execution: the wait holds only once it arrives.
private _token = floor (random 1000000);
["token", _token] call skd_fnc_evidence;
["server", { [_this select 0, 3] call skdfx_fnc_setFlagLater }, [_token]] call skd_fnc_on;
private _ok = ["wait for the flag", {
    ["p1", { !isNil "skdfx_flag" && { skdfx_flag isEqualTo (_this select 0) } }, "flag reached p1", 10, [_this select 0]] call skd_fnc_waitUntil
}, [_token]] call skd_fnc_step;
[_ok, "==", true, "waitUntil returned true"] call skd_fnc_expect;
