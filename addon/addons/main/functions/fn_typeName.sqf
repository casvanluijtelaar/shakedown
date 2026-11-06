/*
    The type of a value in words for a message, such as "a String" or "a Number".

    Arguments: [wrapped] - [] for nothing, else [value].
*/
params ["_wrapped"];
if (_wrapped isEqualTo []) exitWith { "nothing" };
private _v = _wrapped select 0;
if (isNil "_v") exitWith { "nothing" };
switch (typeName _v) do {
    case "SCALAR": { "a Number" };
    case "STRING": { "a String" };
    case "BOOL": { "a Boolean" };
    case "ARRAY": { "an Array" };
    case "HASHMAP": { "a HashMap" };
    case "OBJECT": { "an Object" };
    case "CODE": { "Code" };
    default { format ["a %1", typeName _v] };
}
