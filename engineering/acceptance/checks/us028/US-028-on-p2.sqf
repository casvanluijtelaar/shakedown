// @check US-028 A script error skdfx_fnc_throw raises on p2 fails the check naming p2, the error and the step
// @tags US-028 ERR-01 ERR-03 OBS-05 RES-02 acceptance

["trigger on p2", {
    ["p2", {[] call skdfx_fnc_throw}] call skd_fnc_on;
}] call skd_fnc_step;
