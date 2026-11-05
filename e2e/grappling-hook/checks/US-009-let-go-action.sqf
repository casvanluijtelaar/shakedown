// @check US-009 A player on a placed rope chooses Let go of the rope in the scroll menu and drops off it
// @tags US-009
// @timeout 120

// arma-grappling-hook's US-009, criterion 3: the scroll-menu action lets go as the key does.

private _top = ["build a three-container stack and put p1 on its south lip", {
    private _ground = getTerrainHeightASL [100, 130];
    private _bottom = ["Land_Cargo20_military_green_F", [100, 130, _ground], 0] call skd_fnc_create;
    private _bb = boundingBoxReal _bottom;
    private _low = (_bb select 0) select 2;
    private _height = ((_bb select 1) select 2) - _low;
    _bottom enableSimulation false;
    _bottom setPosASL [100, 130, _ground - _low];
    for "_i" from 1 to 2 do {
        private _c = ["Land_Cargo20_military_green_F", [100, 130, _ground - _low + _i * _height], 0] call skd_fnc_create;
        _c enableSimulation false;
    };
    private _hit = lineIntersectsSurfaces [[98, 130, _ground + 30], [98, 130, _ground], objNull, objNull, true, 1, "GEOM", "VIEW"];
    private _topZ = ((_hit select 0) select 0) select 2;
    ["p1", [98, 130 - 1.24 + 0.6, _topZ], 180] call skd_fnc_place;
    _topZ
}] call skd_fnc_step;

private _anchor = ["place a long rope with agh_fnc_placeHook", {
    ["p1", {
        if (backpack player isEqualTo "") then { player addBackpack "B_AssaultPack_rgr" };
        player addItem "agh_placed_long";
        true
    }] call skd_fnc_on;
    ["p1", { [player, "agh_placed_long"] call agh_fnc_placeHook; true }] call skd_fnc_on;
    ["p1", { count (allMissionObjects "agh_anchor") == 1 }, "one rope is up"] call skd_fnc_waitUntil;
    ["p1", { (allMissionObjects "agh_anchor") select 0 }] call skd_fnc_on
}] call skd_fnc_step;

private _held = ["get on through the Get on rope action", {
    ["p1", player, "Get on rope"] call skd_fnc_useAction;
    ["p1", { player getVariable ["agh_climbState", "OFF"] }, "==", "AT_TOP", "p1 holds the rope at its top"] call skd_fnc_expectOn;
    ["p1", { (getPosASL player) select 2 }] call skd_fnc_on
}] call skd_fnc_step;

["let go through the Let go of the rope action", {
    ["p1", player, "Let go of the rope"] call skd_fnc_useAction;
    ["p1", { player getVariable ["agh_climbState", ""] }, "==", "OFF", "p1 is off the rope"] call skd_fnc_expectOn;
    ["p1", { ((getPosASL player) select 2) < (_this select 0) - 0.05 }, "p1 is 5 cm lower than where it held the rope", 1, [_held]] call skd_fnc_waitUntil;
    ["p1", { isTouchingGround player && { ((getPosASL player) select 2) < (_this select 0) - 2 } }, "p1 lands more than 2 m below the top", 10, [_top]] call skd_fnc_waitUntil;
    ["p1", { isNull ((_this select 0) getVariable ["agh_holder", objNull]) }, "the rope is free", 10, [_anchor]] call skd_fnc_waitUntil;
}] call skd_fnc_step;
