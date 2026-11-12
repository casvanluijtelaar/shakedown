/*
    The state summary of inspect (INS-01), read on this machine without stopping anything: the
    mission and clocks, the step running, the player unit, up to 50 objects near the player or a
    named object, and up to 50 running scheduled scripts, leaving out the script that reads them.

    Arguments: [near, radius] - near the net id of an object, or "" for the player; radius metres.
    Returns: HashMap; "error" alone when no object has the net id.
*/
params [["_near", ""], ["_radius", 50]];
private _unit = if (hasInterface) then { player } else { objNull };
private _center = _unit;
if (_near isNotEqualTo "") then { _center = objectFromNetId _near };
if (_near isNotEqualTo "" && { isNull _center }) exitWith {
    createHashMapFromArray [["error", format ["No object has the net id %1 on %2", _near, skd_machine]]]
};
private _out = createHashMapFromArray [
    ["mission", missionName],
    ["time", time toFixed 3],
    ["tick", diag_tickTime toFixed 3],
    ["fps", diag_fps toFixed 1],
    ["frame", diag_frameNo toFixed 0],
    ["step_path", []]
];
// The step running: the step of the check that runs here, or else the lead's step whose order
// (a wait, a poll or code run here) is the newest still running here.
if (!isNil "skd_check" && { !(skd_check get "done") }) then {
    _out set ["step_path", call skd_fnc_stepPath];
} else {
    private _i = -1;
    for "_j" from count skd_orderPaths - 1 to 0 step -1 do {
        if (((skd_orderPaths select _j) select 1) isNotEqualTo []) exitWith { _i = _j };
    };
    if (_i >= 0) then { _out set ["step_path", (skd_orderPaths select _i) select 1] };
};
if (!isNull _unit) then {
    private _player = createHashMapFromArray [
        ["class", typeOf _unit],
        ["position", getPosASL _unit],
        ["direction", getDir _unit],
        ["damage", damage _unit],
        ["alive", alive _unit],
        ["animation", animationState _unit]
    ];
    if (vehicle _unit isNotEqualTo _unit) then { _player set ["vehicle", vehicle _unit] };
    _out set ["player", _player];
};
private _nearby = [];
if (!isNull _center) then {
    {
        if (_x isNotEqualTo _center && { count _nearby < 50 }) then {
            _nearby pushBack createHashMapFromArray [
                ["class", typeOf _x], ["net_id", netId _x], ["position", getPosASL _x], ["distance", _x distance _center]
            ];
        };
    } forEach (nearestObjects [_center, ["All"], _radius]);
};
_out set ["nearby", _nearby];
// The scheduled scripts running here, without the exec order that reads them: _skd_order is the
// order of the script this function runs in (fn_runExec.sqf).
private _self = if (isNil "_skd_order") then { "" } else { format ["skd_exec %1", _skd_order] };
private _scripts = [];
{
    _x params ["_name", "_file", "_running", "_line"];
    if (_name isNotEqualTo _self && { count _scripts < 50 }) then {
        _scripts pushBack createHashMapFromArray [["name", _name], ["file", _file], ["line", _line]];
    };
} forEach diag_activeSQFScripts;
_out set ["scripts", _scripts];
_out
