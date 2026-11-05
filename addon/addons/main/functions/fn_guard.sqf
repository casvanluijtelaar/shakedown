/*
    Checks that a check API function is called from a check body on the lead machine. Called first
    by every skd_fnc_* function of the check API.

    Arguments: [functionName]
    Returns: true when the call may go on; otherwise the check has ended with check-error.
*/
params ["_function"];
if (!isNil "_skd_remote") exitWith {
    ["fail", "check-error", format ["%1 was called inside code sent to a machine; skd_fnc_* functions run only in the check body. Move the call out of that code.", _function]] call skd_fnc_end;
    false
};
if (isNil "skd_check" || { skd_check get "done" }) exitWith {
    diag_log text format ["[shakedown] %1 called outside a running check", _function];
    false
};
true
