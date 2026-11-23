// @check US-083 RUN-07 MAN-01 in run --each-setup a check of the first setup builds the crate scene in singleplayer and hands p1 over
// @tags US-083 acceptance
// @setups sp

private _crate = ["build the scene", {
    ["p1", AGLToASL [150, 150, 0], 0] call skd_fnc_place;
    ["p1", { player removeItems "FirstAidKit"; { _x == "FirstAidKit" } count items player }] call skd_fnc_on;
    ["skdfx_Crate", AGLToASL [150, 152, 0], 180] call skd_fnc_create
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
