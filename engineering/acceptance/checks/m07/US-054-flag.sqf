// @check US-054 a second short check of the finding: a value the server sets reaches every machine
// @tags m07-demo US-054 acceptance
// @setups rld7 rld7sp

// skdfx_flag survives the reset, so the value is one of this execution.
private _token = floor (random 1000000);
["set skdfx_flag on the server", {
    ["server", { [_this select 0] call skdfx_fnc_setFlag }, [_token]] call skd_fnc_on;
}, [_token]] call skd_fnc_step;
["every machine sees it", {
    ["all", { missionNamespace getVariable ["skdfx_flag", -1] }, "==", _this select 0, "skdfx_flag", 5, []] call skd_fnc_expectEach;
}, [_token]] call skd_fnc_step;
