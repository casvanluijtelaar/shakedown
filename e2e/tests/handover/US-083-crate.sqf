// @check US-083 The owner is handed p1 in front of the fixture's crate, with three things to look for
// @tags US-083 MAN-01 handover
// @setups dedicated

// The owner's look of M15: run it with `shakedown run --setup dedicated handover/US-083-crate`
// from the e2e folder; the session stays up until `shakedown session stop`.
// p1 stands away from the clients' start positions, so no other player stands in its view, and
// 2 m from the crate, inside the 3 m radius of its action Open crate. p1's loadout has its first
// aid kit taken out, so the one the crate gives is the only one; the reset puts the loadout back.
private _crate = ["build the scene", {
    ["p1", AGLToASL [130, 130, 0], 0] call skd_fnc_place;
    ["p1", { player removeItems "FirstAidKit"; { _x == "FirstAidKit" } count items player }] call skd_fnc_on;
    ["skdfx_Crate", AGLToASL [130, 132, 0], 180] call skd_fnc_create
}] call skd_fnc_step;

["p1 can open the crate", {
    ["p1", {
        params ["_crate"];
        !isNull _crate
            && { (actionIDs _crate) findIf { ((_crate actionParams _x) select 0) isEqualTo "Open crate" } >= 0 }
            && { (player distance _crate) < 3 }
    }, "the crate reached p1 with its action Open crate in reach", 10, [_crate]] call skd_fnc_waitUntil;
}] call skd_fnc_step;

["p1", [
    "An ammo crate stands 2 m in front of you.",
    "Looking at it, the action menu (mouse wheel) offers Open crate.",
    "You carry no first aid kit; opening the crate puts one in your inventory."
]] call skd_fnc_handOver;
