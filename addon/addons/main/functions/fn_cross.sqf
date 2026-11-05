/*
    A value as it arrives on the other side of the channel: encoded by DM-13 and decoded by DM-14.
    Code that runs on the lead machine itself takes its arguments and gives its result through
    this, so a check behaves the same whether the code ran on the lead or on another machine
    (P-03 step 5).

    Arguments: [value]
    Returns: the value after the crossing.
*/
params ["_value"];
if (isNil "_value") exitWith { nil };
[[_value] call skd_fnc_encode] call skd_fnc_decode
