// At every mission start the crate's counts are 0; the server sets them for every machine, and a
// machine that has not heard from it yet starts them at 0 itself.
if (isServer) then {
    missionNamespace setVariable ["skdfx_openCount", 0, true];
    missionNamespace setVariable ["skdfx_refusedCount", 0, true];
} else {
    if (isNil "skdfx_openCount") then { skdfx_openCount = 0 };
    if (isNil "skdfx_refusedCount") then { skdfx_refusedCount = 0 };
};
