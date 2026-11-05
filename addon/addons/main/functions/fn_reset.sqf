/*
    The reset in place on the lead machine (P-06, RST-01): runs the cleanups the previous check
    registered, last first; brings the player out of any vehicle and back to life; deletes every
    entity created since the baseline (crew first); restores the player's loadout, damage,
    direction, position, velocity and animation state and the date and weather; then verifies, polling
    every frame, that nothing recorded is left, the player is within 0.1 m and 3 degrees of the
    baseline and the date and overcast match. The whole reset has the ceiling of RST-04. In
    singleplayer every machine is this one.

    Arguments: [ceiling] - seconds, 15 by RST-04.
    Returns: HashMap with "state" ("done" or "failed"), "duration_s" and "mismatches".
*/
params [["_ceiling", 15]];
private _start = diag_tickTime;
private _deadline = _start + _ceiling;
private _result = {
    params ["_mismatches"];
    createHashMapFromArray [
        ["state", ["failed", "done"] select (_mismatches isEqualTo [])],
        ["duration_s", (diag_tickTime - _start) toFixed 3],
        ["mismatches", _mismatches]
    ]
};
if (isNil "skd_baseline") exitWith { [["no baseline was captured at this mission's start"]] call _result };
private _base = skd_baseline;

// 1. The cleanups of the previous check, last registered first.
private _cleanups = +skd_cleanups;
skd_cleanups = [];
reverse _cleanups;
private _stuck = "";
{
    _x params ["_code", "_args", "_check"];
    private _script = [_args, _code] spawn { params ["_skd_args", "_skd_code"]; _skd_args call _skd_code };
    waitUntil { scriptDone _script || { diag_tickTime > _deadline } };
    if (!scriptDone _script) exitWith {
        terminate _script;
        _stuck = format ["a cleanup registered by check %1 was still running at the reset's ceiling of %2 s", _check, _ceiling];
    };
} forEach _cleanups;
if (_stuck != "") exitWith { [[_stuck]] call _result };

// 2. The player: alive (SPK-05 gives a dead player a new unit at once) and out of any vehicle.
waitUntil { alive player || { diag_tickTime > _deadline } };
private _unit = player;
if (vehicle _unit isNotEqualTo _unit) then {
    moveOut _unit;
    unassignVehicle _unit;
    waitUntil { vehicle _unit isEqualTo _unit || { diag_tickTime > _deadline } };
};

// 3. Everything created since the baseline, crew first. The player's unit is never deleted.
private _created = skd_created select { !isNull _x && { _x isNotEqualTo _unit } };
{
    if (!(_x isKindOf "CAManBase") && { (crew _x) findIf { _x isEqualTo _unit } < 0 }) then { deleteVehicleCrew _x };
} forEach _created;
{ deleteVehicle _x } forEach _created;

// 4. The player's unit as at the baseline.
_unit setUnitLoadout (_base get "loadout");
_unit setDamage 0;
if (!isNil "ace_medical_treatment_fnc_fullHealLocal") then { [_unit] call ace_medical_treatment_fnc_fullHealLocal };
_unit setDir (_base get "direction");
_unit setPosASL (_base get "position");
_unit setVelocity [0, 0, 0];
_unit switchMove (_base get "animation");

// 5. The date and the weather.
setDate (_base get "date");
0 setOvercast (_base get "overcast");
0 setFog (_base get "fog");
0 setRain (_base get "rain");
forceWeatherChange;

// 6. Verify, every frame, until everything matches or the ceiling passes.
private _check = {
    private _out = [];
    private _left = skd_created select { !isNull _x && { _x isNotEqualTo player } };
    if (_left isNotEqualTo []) then {
        _out pushBack format ["%1 created entities still exist: %2", count _left, (_left select [0, 5]) apply { typeOf _x } joinString ", "];
    };
    private _u = player;
    if (!alive _u) then { _out pushBack "the player is dead" };
    private _distance = (getPosASL _u) distance (_base get "position");
    if (_distance > 0.1) then { _out pushBack format ["the player is %1 m from the baseline position", _distance toFixed 2] };
    private _d = abs ((getDir _u) - (_base get "direction")) % 360;
    if ((_d min (360 - _d)) > 3) then { _out pushBack format ["the player faces %1, the baseline %2", getDir _u, _base get "direction"] };
    if (vehicle _u isNotEqualTo _u) then { _out pushBack format ["the player is in a %1", typeOf vehicle _u] };
    if ((date select [0, 5]) isNotEqualTo ((_base get "date") select [0, 5])) then { _out pushBack format ["the date is %1, the baseline %2", date, _base get "date"] };
    if (abs (overcast - (_base get "overcast")) > 0.01) then { _out pushBack format ["the overcast is %1, the baseline %2", overcast, _base get "overcast"] };
    _out
};
// A unit that just left a vehicle or fell can still slide after it was set down, so while it is
// more than 2 cm from its baseline position it is set there again, for up to a second.
private _settle = diag_tickTime + 1;
private _mismatches = call _check;
waitUntil {
    private _u = player;
    private _off = (getPosASL _u) distance (_base get "position");
    if (_off > 0.02 && { diag_tickTime < _settle }) then {
        _u setDir (_base get "direction");
        _u setPosASL (_base get "position");
        _u setVelocity [0, 0, 0];
    };
    _mismatches = call _check;
    (_mismatches isEqualTo [] && { _off <= 0.02 || { diag_tickTime >= _settle } }) || { diag_tickTime > _deadline }
};
if (_mismatches isEqualTo []) then {
    // Groups created since the baseline that the deletion left empty.
    { if (!(_x in skd_baseGroups) && { (units _x) isEqualTo [] }) then { deleteGroup _x } } forEach allGroups;
    skd_created = [];
};
[_mismatches] call _result
