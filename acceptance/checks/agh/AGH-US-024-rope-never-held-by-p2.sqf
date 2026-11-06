// @check US-024 M3 demo: the M2 rope check with its last assertion changed to p2 holding the rope, which the mod cannot meet
// @tags M3 demo US-024 OBS-04 RES-03 acceptance

/* arma-grappling-hook's US-009, criterion 1, through its public interface: agh_fnc_placeHook,
   the "Get on rope" scroll action on the player's unit and the CBA keybind "Grappling hooks" /
   "agh_letGo" (that repository's docs/placed-rope.md and spec/07-tech-stack.md). */

if (isNil "agh_fnc_placeHook") then { ["arma-grappling-hook is not loaded in this setup"] call skd_fnc_invalid };

private _anchor = ["put a rope up on a container's south lip", {
    private _box = ["Land_Cargo20_military_green_F", AGLToASL [130, 130, 0], 0] call skd_fnc_create;
    // On a dedicated session the container reaches p1 a moment after the server made it, and
    // first at the place the server created it before moving it into position.
    ["p1", { (getPosASL objectFromNetId (_this select 0)) distance2D (_this select 1) < 0.5 }, "p1 sees the container in place", 10,
        [netId _box, AGLToASL [130, 130, 0]]] call skd_fnc_waitUntil;
    private _stand = ["p1", {
        params ["_box"];
        private _c = getPosASL _box;
        private _topAt = {
            private _from = [_this select 0, _this select 1, (_c select 2) + 20];
            private _hits = lineIntersectsSurfaces [_from, [_this select 0, _this select 1, (_c select 2) - 2], objNull, objNull, true, 1, "GEOM", "NONE"];
            if (count _hits == 0) exitWith { -1000 };
            private _hit = _hits select 0;
            if ((_hit select 2) != _box) exitWith { -1000 };
            (_hit select 0) select 2
        };
        private _top = [_c select 0, _c select 1] call _topAt;
        private _lip = 0;
        for "_d" from 0 to 5 step 0.05 do {
            if (([_c select 0, (_c select 1) - _d] call _topAt) < _top - 0.2) exitWith { _lip = _d };
        };
        [_c select 0, (_c select 1) - _lip + 0.6, _top]
    }, [_box]] call skd_fnc_on;
    ["p1", _stand, 180] call skd_fnc_place;
    ["p1", {
        player addBackpack "B_Carryall_khk";
        player addItemToBackpack "agh_placed_short";
        true
    }] call skd_fnc_on;
    // The server checks the unit carries the hook, so it must see the item before p1 places it.
    ["server", { "agh_placed_short" in items (_this select 0) }, "the server sees the hook in p1's backpack", 10,
        [["p1"] call skd_fnc_player]] call skd_fnc_waitUntil;
    ["p1", { [player, "agh_placed_short"] call agh_fnc_placeHook; true }] call skd_fnc_on;
    ["p1", { count (nearestObjects [player, ["agh_anchor"], 5]) > 0 }, "a rope is up within 5 m of p1", 10] call skd_fnc_waitUntil;
    ["p1", { (nearestObjects [player, ["agh_anchor"], 5]) select 0 }] call skd_fnc_on
}] call skd_fnc_step;

["p1 gets on the rope through the mod's action, and the server and both clients see p1 hold it", {
    params ["_anchor"];
    private _p1 = ["p1"] call skd_fnc_player;
    ["p1", _p1, "Get on rope"] call skd_fnc_useAction;
    ["p1", {
        (player getVariable ["agh_onRope", objNull]) isEqualTo (_this select 0) && { (player getVariable ["agh_climbState", "OFF"]) != "OFF" }
    }, "p1 holds the rope", 10, [_anchor]] call skd_fnc_waitUntil;
    [["server", "p1", "p2"], { ((_this select 0) getVariable ["agh_holder", objNull]) isEqualTo (_this select 1) }, "==", true,
        "the rope's holder is p1", 10, [_anchor, _p1]] call skd_fnc_expectEach;
}, [_anchor]] call skd_fnc_step;

["p1 presses the let-go key", {
    ["p1", "Grappling hooks", "agh_letGo"] call skd_fnc_pressKey;
}] call skd_fnc_step;

["p1 is off the rope and the server and both clients agree the rope is free", {
    params ["_anchor"];
    private _p1 = ["p1"] call skd_fnc_player;
    ["p1", {
        isNull (player getVariable ["agh_onRope", objNull]) && { (player getVariable ["agh_climbState", "OFF"]) == "OFF" }
    }, "p1 is off the rope", 10] call skd_fnc_waitUntil;
    [["server", "p1", "p2"], { isNull ((_this select 0) getVariable ["agh_holder", objNull]) }, "==", true,
        "the rope has no holder", 10, [_anchor]] call skd_fnc_expectEach;
}, [_anchor]] call skd_fnc_step;

// The last assertion of the M2 check, changed to one the mod cannot meet: nobody took the rope after p1.
["the server and both clients see p2 holding the rope", {
    params ["_anchor"];
    [["server", "p1", "p2"], {
        private _holder = (_this select 0) getVariable ["agh_holder", objNull];
        if (isNull _holder) then { "nobody" } else { name _holder }
    }, "==", "p2", "the rope's holder is p2", 5, [_anchor]] call skd_fnc_expectEach;
}, [_anchor]] call skd_fnc_step;
