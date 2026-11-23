// @check RST-01 The cleanups of the check before ran at the reset, last registered first
// @tags RST-01 acceptance

["the cleanups ran in reverse order", {
    private _ran = missionNamespace getVariable ["acc_m01_cleanups", "never set"];
    acc_m01_cleanups = nil;
    [_ran, "==", [2, 1], "the cleanups that ran, in order"] call skd_fnc_expect;
}] call skd_fnc_step;
