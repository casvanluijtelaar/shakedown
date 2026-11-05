/*
    Resolves the machine argument of a check API function to the names of the session's machines
    (DM-10, DM-12). With many = false the value must be one machine name; with many = true it may be
    a name, a group name ("all", "clients", "headless") or an array of names. In singleplayer
    "server" names p1. A name the session does not have ends the check fail / check-error.

    Arguments: [function, index, value, many]
    Returns: array of machine names, or nothing when the check has ended.
*/
params ["_function", "_index", "_value", "_many"];
private _machines = skd_check get "machines";
private _single = (skd_check get "mode") isEqualTo "singleplayer";
private _expected = ["a machine name (String)", "a machine name, a group name or an array of machine names"] select _many;
if (isNil "_value") exitWith { [_function, _index, "machine", _expected, nil] call skd_fnc_argError; nil };
private _names = nil;
if (_value isEqualType "") then {
    _names = [_value];
    if (_many) then {
        switch (_value) do {
            case "all": { _names = +_machines };
            case "clients": { _names = _machines select { (_x select [0, 1]) isEqualTo "p" } };
            case "headless": { _names = _machines select { (_x select [0, 2]) isEqualTo "hc" } };
        };
    };
} else {
    if (_many && { _value isEqualType [] } && { (_value findIf { !(_x isEqualType "") }) < 0 }) then { _names = _value };
};
if (isNil "_names") exitWith { [_function, _index, "machine", _expected, _value] call skd_fnc_argError; nil };
_names = _names apply { ["p1", _x] select !(_single && { _x isEqualTo "server" }) };
private _unknown = _names select { !(_x in _machines) };
if (_unknown isNotEqualTo []) exitWith {
    ["fail", "check-error", format ["%1: the session has no machine %2; its machines are %3, and the groups all, clients and headless.",
        _function, _unknown joinString ", ", _machines joinString ", "]] call skd_fnc_end;
    nil
};
_names arrayIntersect _names
