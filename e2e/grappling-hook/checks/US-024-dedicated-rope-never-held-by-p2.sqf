// @check US-024 The M2 rope check with a last assertion the mod cannot meet fails with its step, each machine's last value and log excerpt
// @tags US-024 demo dedicated
// @setups dedicated dedicated-2
// @timeout 120

// The M3 demo (plan/milestones/m03.md): US-009-dedicated-freed-rope with its last assertion
// changed to one the mod cannot meet, that p2 holds the rope p1 let go of. It fails in the last
// step with each machine's last value, and the run folder keeps each machine's log excerpt.

// A stack of three containers north of p1, simulation off, with p1 on its south lip facing the drop.
private _top = ["build a three-container stack and put p1 on its south lip", {
    private _ground = getTerrainHeightASL [100, 130];
    private _bottom = ["Land_Cargo20_military_green_F", [100, 130, _ground], 0] call skd_fnc_create;
    private _bb = boundingBoxReal _bottom;
    private _low = (_bb select 0) select 2;
    private _height = ((_bb select 1) select 2) - _low;
    _bottom enableSimulationGlobal false;
    _bottom setPosASL [100, 130, _ground - _low];
    for "_i" from 1 to 2 do {
        private _c = ["Land_Cargo20_military_green_F", [100, 130, _ground - _low + _i * _height], 0] call skd_fnc_create;
        _c enableSimulationGlobal false;
    };
    private _topZ = ["p1", {
        private _ground = getTerrainHeightASL [100, 130];
        private _hit = [];
        waitUntil {
            _hit = lineIntersectsSurfaces [[98, 130, _ground + 30], [98, 130, _ground], objNull, objNull, true, 1, "GEOM", "VIEW"];
            _hit isNotEqualTo [] && { (((_hit select 0) select 0) select 2) - _ground > 7 }
        };
        ((_hit select 0) select 0) select 2
    }] call skd_fnc_on;
    [_topZ - _ground, ">", 7, "p1 sees the stack over 7 m tall"] call skd_fnc_expect;
    ["p1", [98, 130 - 1.24 + 0.6, _topZ], 180] call skd_fnc_place;
    _topZ
}] call skd_fnc_step;

private _anchor = ["place a long rope with agh_fnc_placeHook", {
    ["p1", {
        if (backpack player isEqualTo "") then { player addBackpack "B_AssaultPack_rgr" };
        player addItem "agh_placed_long";
        "agh_placed_long" in items player
    }] call skd_fnc_on;
    ["p1", { [player, "agh_placed_long"] call agh_fnc_placeHook; true }] call skd_fnc_on;
    ["p1", { count (allMissionObjects "agh_anchor") == 1 }, "one rope is up"] call skd_fnc_waitUntil;
    ["p1", { (allMissionObjects "agh_anchor") select 0 }] call skd_fnc_on
}] call skd_fnc_step;

private _held = ["get on through the Get on rope action", {
    ["p1", ["p1"] call skd_fnc_player, "Get on rope"] call skd_fnc_useAction;
    ["p1", { player getVariable ["agh_climbState", "OFF"] }, "==", "AT_TOP", "p1 holds the rope at its top"] call skd_fnc_expectOn;
    [["server", "p1", "p2"], { ((_this select 0) getVariable ["agh_holder", objNull]) isEqualTo ((allPlayers select { name _x == "p1" }) select 0) }, "==", true, "every machine sees p1 as the rope's holder", 10, [_anchor]] call skd_fnc_expectEach;
    ["p1", { (getPosASL player) select 2 }] call skd_fnc_on
}] call skd_fnc_step;

["let go with the let-go key", {
    ["p1", "Grappling hooks", "agh_letGo"] call skd_fnc_pressKey;
    ["p1", { player getVariable ["agh_climbState", ""] }, "==", "OFF", "p1 is off the rope"] call skd_fnc_expectOn;
    ["p1", { ((getPosASL player) select 2) < (_this select 0) - 0.05 }, "p1 is 5 cm lower than where it held the rope", 1, [_held]] call skd_fnc_waitUntil;
    ["p1", { isTouchingGround player && { ((getPosASL player) select 2) < (_this select 0) - 2 } }, "p1 lands more than 2 m below the top", 10, [_top]] call skd_fnc_waitUntil;
}] call skd_fnc_step;

["the server and both clients see p2 holding the rope", {
    [["server", "p1", "p2"], { private _h = (_this select 0) getVariable ["agh_holder", objNull]; if (isNull _h) then { "nobody" } else { name _h } }, "==", "p2", "the rope's holder is p2", 5, [_anchor]] call skd_fnc_expectEach;
}] call skd_fnc_step;
