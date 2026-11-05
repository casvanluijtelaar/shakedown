/*
    Where an action of this machine's player on a target stands (ACT-01): found by its title after
    removing structured-text tags, exactly and case-sensitively; its condition evaluated as the
    engine does, with _target the object carrying it and _this the player; its radius against the
    player's distance (-1 is no limit, the default 50 m).

    Arguments: [target, title]
    Returns: ["ready", id], ["missing", titles], ["condition", condition text] or
    ["radius", distance, radius].
*/
params ["_target", "_title"];
private _titles = [];
private _found = -1;
private _params = [];
{
    private _p = _target actionParams _x;
    private _plain = (_p select 0) regexReplace ["<[^>]*>", ""];
    _titles pushBack _plain;
    if (_found < 0 && { _plain isEqualTo _title }) then { _found = _x; _params = _p };
} forEach (actionIDs _target);
if (_found < 0) exitWith { ["missing", _titles] };
private _condition = _params param [7, "true"];
if (_condition isEqualTo "") then { _condition = "true" };
private _radius = _params param [8, 50];
private _caller = player;
private _held = false;
private _skd_condition = compile _condition;
private _originalTarget = _target;
// The condition sees _target and _originalTarget from here and _this, the caller, from the call.
isNil {
    _held = _caller call {
        private _r = call _skd_condition;
        !isNil "_r" && { _r isEqualTo true }
    };
};
if (!_held) exitWith { ["condition", _condition] };
private _distance = if (_target isEqualTo _caller) then { 0 } else { _caller distance _target };
if (_radius >= 0 && { _distance > _radius }) exitWith { ["radius", _distance, _radius] };
["ready", _found]
