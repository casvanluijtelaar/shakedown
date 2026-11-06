/*
    One phase of the reset in place (P-06, RST-01), on the machine it runs on. The runner runs the
    phases in order: "cleanups" on the lead machine, then "free", "restore" and "verify" on every
    machine at once. In singleplayer every machine is p1.

    - "cleanups" [ceiling]: runs the cleanups the previous check registered, last first (P-06
      step 1). A cleanup still running at the ceiling is terminated and named.
    - "free" [ceiling]: waits for this machine's player to be alive (a dead player respawns at
      once, RST-05) and moves it out of any vehicle, so no deletion takes a player with it.
    - "restore" [ceiling, world]: deletes every entity this machine recorded since the baseline that is
      local here and not a player, crew first (P-06 steps 2 and 3); brings this machine's player unit back to its
      baseline (step 4); and sets the date and weather of `world`, the server's baseline, here
      (step 5), so no machine waits for the server's to reach it.
    - "verify" [ceiling, world, players]: polls every frame until nothing recorded is left, the
      date is the baseline's to the minute, this machine's player is at its baseline and, on the
      lead machine, every player in `players` ([name, position, direction]) is within 0.1 m and 3
      degrees of its baseline as this machine sees it and the overcast is the baseline's (step 6).
      While this machine's player is more than 2 cm from its baseline position it is set there
      again, for up to 1.5 s, and the phase ends once it has stayed within 2 cm for 0.3 s, since
      a unit just out of a vehicle can still slide.

    Arguments: [phase, args]
    Returns: an array of what did not match, empty when the phase is done.
*/
params ["_phase", "_args"];
if (isNil "skd_baseline") exitWith { [format ["%1: no baseline was captured at this mission's start", skd_machine]] };
private _base = skd_baseline;
private _hasPlayer = hasInterface && { !isNull player } && { !isNil { _base get "position" } };
private _start = diag_tickTime;
private _keep = {
    params ["_x"];
    _x in allPlayers || { isPlayer _x } || { _x isEqualTo player } || { _x isKindOf "HeadlessClient_F" }
};
// Deletes the recorded entities that are local here, a unit in a vehicle out of it first. Each
// machine deletes only its own: the machine an entity is local to is the one that deletes it, so
// no machine asks another to delete an object that machine has deleted already, which the server
// logs as "Object ... not found (message Type_189)" (DEF-09). "verify" deletes again every frame,
// for an entity whose locality moved here since, such as a vehicle a player has just left, and an
// entity made here while the reset runs, such as a crater.
private _deleteLocal = {
    params ["_entities"];
    private _mine = _entities select { !isNull _x && { local _x } && { !([_x] call _keep) } };
    { if (_x isKindOf "CAManBase" && { vehicle _x isNotEqualTo _x }) then { (vehicle _x) deleteVehicleCrew _x } } forEach _mine;
    { deleteVehicle _x } forEach _mine;
};

switch (_phase) do {
    case "cleanups": {
        _args params [["_ceiling", 15]];
        private _cleanups = +skd_cleanups;
        skd_cleanups = [];
        reverse _cleanups;
        private _stuck = [];
        {
            _x params ["_code", "_cargs", "_check"];
            private _script = [_cargs, _code] spawn { params ["_skd_args", "_skd_code"]; _skd_args call _skd_code };
            waitUntil { scriptDone _script || { diag_tickTime - _start > _ceiling } };
            if (!scriptDone _script) exitWith {
                terminate _script;
                _stuck = [format ["a cleanup registered by check %1 was still running at the reset's ceiling of %2 s", _check, _ceiling]];
            };
        } forEach _cleanups;
        _stuck
    };
    case "free": {
        _args params [["_ceiling", 15]];
        if (!_hasPlayer) exitWith { [] };
        waitUntil { alive player || { diag_tickTime - _start > _ceiling } };
        private _unit = player;
        if (vehicle _unit isNotEqualTo _unit) then {
            moveOut _unit;
            unassignVehicle _unit;
            waitUntil { vehicle _unit isEqualTo _unit || { diag_tickTime - _start > _ceiling } };
        };
        if (!alive player) exitWith { [format ["%1: the player is dead", skd_machine]] };
        []
    };
    case "restore": {
        _args params [["_ceiling", 15], "_world"];
        [skd_created] call _deleteLocal;
        if (_hasPlayer) then {
            private _unit = player;
            _unit setUnitLoadout (_base get "loadout");
            _unit setDamage 0;
            if (!isNil "ace_medical_treatment_fnc_fullHealLocal") then { [_unit] call ace_medical_treatment_fnc_fullHealLocal };
            _unit setDir (_base get "direction");
            _unit setPosASL (_base get "position");
            _unit setVelocity [0, 0, 0];
            // A group that saw an enemy during the check stays in combat and raises the player's
            // weapon out of the baseline's animation state.
            (group _unit) setBehaviour (_base getOrDefault ["behaviour", "AWARE"]);
            _unit switchMove (_base get "animation");
        };
        setDate (_world get "date");
        0 setOvercast (_world get "overcast");
        0 setFog (_world get "fog");
        0 setRain (_world get "rain");
        forceWeatherChange;
        []
    };
    case "verify": {
        _args params [["_ceiling", 15], "_world", ["_players", []]];
        private _lead = isServer || { !isMultiplayer };
        private _angle = { params ["_a", "_b"]; private _d = abs (_a - _b) % 360; _d min (360 - _d) };
        private _check = {
            private _out = [];
            private _left = skd_created select { !isNull _x && { !([_x] call _keep) } };
            if (_left isNotEqualTo []) then {
                _out pushBack format ["%1 created entities still exist: %2", count _left, (_left select [0, 5]) apply { typeOf _x } joinString ", "];
            };
            if ((date select [0, 5]) isNotEqualTo ((_world get "date") select [0, 5])) then {
                _out pushBack format ["the date is %1, the baseline %2", date select [0, 5], (_world get "date") select [0, 5]];
            };
            if (_hasPlayer) then {
                private _u = player;
                if (!alive _u) then { _out pushBack "the player is dead" };
                private _distance = (getPosASL _u) distance (_base get "position");
                if (_distance > 0.1) then { _out pushBack format ["the player is %1 m from its baseline position", _distance toFixed 2] };
                if ([getDir _u, _base get "direction"] call _angle > 3) then { _out pushBack format ["the player faces %1, the baseline %2", getDir _u, _base get "direction"] };
                if (vehicle _u isNotEqualTo _u) then { _out pushBack format ["the player is in a %1", typeOf vehicle _u] };
            };
            if (_lead) then {
                {
                    _x params ["_name", "_position", "_direction"];
                    private _u = [_name] call skd_fnc_unitOf;
                    if (isNull _u) then { _out pushBack format ["it sees no unit of %1", _name] } else {
                        private _distance = (getPosASL _u) distance _position;
                        if (_distance > 0.1) then { _out pushBack format ["it sees %1 %2 m from its baseline position", _name, _distance toFixed 2] };
                        if ([getDir _u, _direction] call _angle > 3) then { _out pushBack format ["it sees %1 facing %2, the baseline %3", _name, getDir _u, _direction] };
                        if (!alive _u) then { _out pushBack format ["it sees %1 dead", _name] };
                    };
                } forEach _players;
                if (abs (overcast - (_world get "overcast")) > 0.01) then { _out pushBack format ["the overcast is %1, the baseline %2", overcast, _world get "overcast"] };
            };
            _out
        };
        private _settle = diag_tickTime + 1.5;
        private _still = diag_tickTime;
        private _mismatches = call _check;
        waitUntil {
            private _off = 0;
            if (_hasPlayer) then {
                private _u = player;
                _off = (getPosASL _u) distance (_base get "position");
                // A unit whose animation state ran on from the baseline's is put back into it.
                if (animationState _u != (_base get "animation")) then {
                    _still = diag_tickTime;
                    if (diag_tickTime < _settle) then { _u switchMove (_base get "animation") };
                };
                if (_off > 0.02) then {
                    _still = diag_tickTime;
                    if (diag_tickTime < _settle) then {
                        _u setDir (_base get "direction");
                        _u setPosASL (_base get "position");
                        _u setVelocity [0, 0, 0];
                    };
                };
            };
            [skd_created] call _deleteLocal;
            _mismatches = call _check;
            (_mismatches isEqualTo [] && { (_hasPlayer && { diag_tickTime - _still >= 0.3 }) || { !_hasPlayer } || { diag_tickTime >= _settle } })
                || { diag_tickTime - _start > _ceiling }
        };
        if (_mismatches isEqualTo []) then {
            // Groups created since the baseline that the deletion left empty.
            { if (local _x && { !(_x in skd_baseGroups) } && { (units _x) isEqualTo [] }) then { deleteGroup _x } } forEach allGroups;
            skd_created = [];
        };
        _mismatches apply { format ["%1: %2", skd_machine, _x] }
    };
    default { [format ["%1: unknown reset phase %2", skd_machine, _phase]] };
}
