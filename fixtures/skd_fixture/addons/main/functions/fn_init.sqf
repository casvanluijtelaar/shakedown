// At every mission start the crate's counts are 0; the server sets them for every machine, and a
// machine that has not heard from it yet starts them at 0 itself.
// The same holds for the record of the players who left, which a dedicated server keeps.
if (isServer) then {
    missionNamespace setVariable ["skdfx_openCount", 0, true];
    missionNamespace setVariable ["skdfx_refusedCount", 0, true];
    missionNamespace setVariable ["skdfx_disconnected", [], true];
    if (isMultiplayer) then {
        addMissionEventHandler ["PlayerDisconnected", {
            params ["_id", "_uid", "_name"];
            private _left = missionNamespace getVariable ["skdfx_disconnected", []];
            _left pushBack _name;
            missionNamespace setVariable ["skdfx_disconnected", _left, true];
        }];
    };
} else {
    if (isNil "skdfx_openCount") then { skdfx_openCount = 0 };
    if (isNil "skdfx_refusedCount") then { skdfx_refusedCount = 0 };
    if (isNil "skdfx_disconnected") then { skdfx_disconnected = [] };
};
