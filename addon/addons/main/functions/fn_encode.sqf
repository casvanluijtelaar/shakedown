/*
    Encodes an SQF value for JSON as DM-13 says: values JSON cannot carry become tagged objects.

    Arguments: [value]
    Returns: the encoded value (nil stays nil, which toJSON writes as null).
*/
params ["_value"];
if (isNil "_value") exitWith { nil };
switch (typeName _value) do {
    case "SCALAR";
    case "STRING";
    case "BOOL": { _value };
    case "ARRAY": { _value apply { if (isNil "_x") then { nil } else { [_x] call skd_fnc_encode } } };
    case "HASHMAP": {
        if ((keys _value) findIf { !(_x isEqualType "") } < 0) then {
            private _out = createHashMap;
            { _out set [_x, [_y] call skd_fnc_encode] } forEach _value;
            _out
        } else {
            createHashMapFromArray [
                ["$type", "hashmap"],
                ["pairs", (_value toArray false) apply { [[_x select 0] call skd_fnc_encode, [_x select 1] call skd_fnc_encode] }]
            ]
        };
    };
    case "OBJECT": {
        createHashMapFromArray [
            ["$type", "object"],
            ["netId", if (isNull _value) then { "" } else { netId _value }],
            ["class", typeOf _value],
            ["alive", alive _value]
        ]
    };
    case "GROUP": { createHashMapFromArray [["$type", "group"], ["netId", netId _value]] };
    case "SIDE": { createHashMapFromArray [["$type", "side"], ["name", str _value]] };
    case "CODE": { createHashMapFromArray [["$type", "code"], ["text", toString _value]] };
    default { createHashMapFromArray [["$type", typeName _value], ["text", str _value]] };
};
