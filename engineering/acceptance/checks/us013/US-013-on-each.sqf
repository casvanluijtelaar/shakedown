// @check US-013 CTL-01 onEach on all returns one key per machine of the session
// @tags US-013 CTL-01 acceptance
// @setups ded

private _map = ["all", { clientOwner }] call skd_fnc_onEach;
private _keys = keys _map;
_keys sort true;
private _machines = +(call skd_fnc_machines);
_machines sort true;
["keys", _keys] call skd_fnc_evidence;
["machines", _machines] call skd_fnc_evidence;
[typeName _map, "==", "HASHMAP", "onEach returns a HashMap"] call skd_fnc_expect;
[_keys, "==", _machines, "one key per machine of the session"] call skd_fnc_expect;
