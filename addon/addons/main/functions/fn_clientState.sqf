/*
    The client state of a user as the server sees it (P-02 step 7, INT-AR-09): the state number of
    the user whose profile name is the machine name, from getUserInfo; -1 when no user has it.

    Arguments: [machine]
    Returns: the client state number, or -1.
*/
params ["_machine"];
private _state = -1;
{
    private _info = getUserInfo _x;
    if (_info isEqualType [] && { count _info > 6 } && { _machine in [_info select 3, _info select 4, _info select 5] }) exitWith {
        _state = _info select 6;
    };
} forEach allUsers;
_state
