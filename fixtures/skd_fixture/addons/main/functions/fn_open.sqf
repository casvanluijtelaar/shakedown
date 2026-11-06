// On the server: opens the crate for the caller if, as the server sees it, the caller is alive
// and within 3 m; otherwise counts a refusal.
params ["_crate", "_caller"];
if (!isServer) exitWith {};
if (alive _caller && { (_caller distance _crate) <= 3 } && { !(_crate getVariable ["skdfx_open", false]) }) then {
    _crate setVariable ["skdfx_open", true, true];
    [_caller, "FirstAidKit"] remoteExec ["addItem", _caller];
    missionNamespace setVariable ["skdfx_openCount", (missionNamespace getVariable ["skdfx_openCount", 0]) + 1, true];
} else {
    missionNamespace setVariable ["skdfx_refusedCount", (missionNamespace getVariable ["skdfx_refusedCount", 0]) + 1, true];
};
