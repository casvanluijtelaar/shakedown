// @check US-083 RUN-07 MAN-01 in run --each-setup a check of the first setup builds the crate scene in singleplayer and hands p1 over
// @tags US-083 acceptance
// @setups sp

["build the scene", {
    ["p1", AGLToASL [100, 100, 0], 0] call skd_fnc_place;
    ["skdfx_Crate", AGLToASL [100, 103, 0], 180] call skd_fnc_create
}] call skd_fnc_step;

["p1", [
    "An ammo crate stands 3 m in front of you.",
    "Looking at it, the action menu (mouse wheel) offers Open crate.",
    "Opening it puts a first aid kit in your inventory."
]] call skd_fnc_handOver;
