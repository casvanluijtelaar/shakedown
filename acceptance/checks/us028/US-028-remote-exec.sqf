// @check US-028 A script error skdfx_fnc_throw raises on p2 through remoteExec from the server fails the check naming p2 and the step
// @tags US-028 ERR-01 ERR-03 acceptance

["trigger on p2", {
    [] remoteExec ["skdfx_fnc_throw", ["p2"] call skd_fnc_player];
}] call skd_fnc_step;
