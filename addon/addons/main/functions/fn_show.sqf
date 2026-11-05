/*
    A value as text for a message: "nil" for nothing, else str.

    Arguments: [wrapped] - [] for nothing, else [value].
*/
params ["_wrapped"];
if (_wrapped isEqualTo []) exitWith { "nil" };
private _v = _wrapped select 0;
if (isNil "_v") exitWith { "nil" };
str _v
