/*
    Reads the ceiling argument of a check API function (OBS-01, DM-04): nil takes the default of
    10 s, a number above 0 is kept, anything else ends the check fail / check-error. A ceiling that
    reaches past the check's timeout is cut to the time left (check API, "Ceilings").

    Arguments: [function, index, value]
    Returns: the ceiling in seconds, or nothing when the check has ended.
*/
params ["_function", "_index", "_value"];
if (isNil "_value") then { _value = 10 };
if (!(_value isEqualType 0) || { !(finite _value) } || { _value <= 0 }) exitWith {
    [_function, _index, "ceiling", "a number of seconds above 0", _value] call skd_fnc_argError;
    nil
};
_value min (((skd_check get "deadline") - diag_tickTime) max 0.001)
