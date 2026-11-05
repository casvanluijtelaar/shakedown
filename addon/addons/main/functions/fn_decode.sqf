/*
    Turns a value that crossed the channel back into SQF as DM-14 says: a tagged object or group
    becomes the object or group, by its net id. Every other tagged value stays as it arrived; code
    is not turned back into code.

    Arguments: [value]
    Returns: the decoded value.
*/
params ["_value"];
if (isNil "_value") exitWith { nil };
switch (typeName _value) do {
    case "ARRAY": { _value apply { [_x] call skd_fnc_decode } };
    case "HASHMAP": {
        switch (_value getOrDefault ["$type", ""]) do {
            case "object": { objectFromNetId (_value getOrDefault ["netId", ""]) };
            case "group": { groupFromNetId (_value getOrDefault ["netId", ""]) };
            case "": {
                private _out = createHashMap;
                { _out set [_x, [_y] call skd_fnc_decode] } forEach _value;
                _out
            };
            default { _value };
        };
    };
    default { _value };
};
