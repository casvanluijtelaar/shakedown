// @check US-083 The owner is handed p1 in front of the fixture's crate, with three things to look for
// @tags US-083 MAN-01 handover
// @setups dedicated

// The owner's look of M15: run it with `shakedown run --setup dedicated handover/US-083-crate`
// from the e2e folder; the session stays up until `shakedown session stop`.
private _crate = ["build the scene", {
    ["p1", AGLToASL [100, 100, 0], 0] call skd_fnc_place;
    ["skdfx_Crate", AGLToASL [100, 103, 0], 180] call skd_fnc_create
}] call skd_fnc_step;

["p1 sees the crate", {
    ["p1", { !isNull (_this select 0) }, "the crate reached p1", 10, [_crate]] call skd_fnc_waitUntil;
}] call skd_fnc_step;

["p1", [
    "An ammo crate stands 3 m in front of you.",
    "Looking at it, the action menu (mouse wheel) offers Open crate.",
    "Opening it puts a first aid kit in your inventory."
]] call skd_fnc_handOver;
