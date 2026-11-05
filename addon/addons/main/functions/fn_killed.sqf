/*
    The singleplayer player's Killed handler (RST-05). A singleplayer mission ends when its player
    dies, even with respawn set, so before the engine ends it the agent hands the player a new unit
    of the same class where the old one fell, with selectPlayer (SPK-05). The reset then brings the
    new unit to the baseline, and deletes the body.

    Arguments: the Killed event's [unit, killer, instigator, useEffects]
*/
params ["_unit"];
if (_unit isNotEqualTo player) exitWith {};
// The new unit joins the dead unit's group, so the session keeps the groups it started with.
private _group = group _unit;
if (isNull _group) then {
    _group = createGroup [if (isNil "skd_baseline") then { west } else { skd_baseline get "side" }, true];
};
private _new = _group createUnit [typeOf _unit, [0, 0, 0], [], 0, "CAN_COLLIDE"];
_new setDir (getDir _unit);
_new setPosASL (getPosASL _unit);
selectPlayer _new;
_new addEventHandler ["Killed", { _this call skd_fnc_killed }];
skd_created pushBack _unit;
