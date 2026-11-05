/*
    Ends the check fail / check-error for a wrong argument to a check API function (DM-04).

    Arguments: [function, index, name, expected, value] - index counts from 1.
*/
params ["_function", "_index", "_name", "_expected", "_value"];
private _got = if (isNil "_value") then { "nothing" } else { format ["%1 %2", typeName _value, str _value] };
["fail", "check-error", format ["%1: argument %2 (%3) must be %4, got %5.", _function, _index, _name, _expected, _got]] call skd_fnc_end;
