// @check US-084 MAN-01 DM-04 a hand over to hc1 on a dedicated session with two clients and a headless client ends fail check-error naming p1 and p2
// @tags US-084 acceptance
// @setups ded

["build the scene", {
    ["p1", AGLToASL [200, 200, 0], 0] call skd_fnc_place;
    ["skdfx_Crate", AGLToASL [200, 203, 0], 180] call skd_fnc_create
}] call skd_fnc_step;

["hc1", [
    "An ammo crate stands 3 m in front of you."
]] call skd_fnc_handOver;
