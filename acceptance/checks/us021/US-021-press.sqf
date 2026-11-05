// @check US-021 Pressing the fixture signal runs its key-down and key-up code once each
// @tags US-021 ACT-02 acceptance

private _unit = ["p1"] call skd_fnc_player;
// The reset leaves variables on the player's unit; put the counts back for the next check.
[{
    (_this select 0) setVariable ["skdfx_signalDown", nil, true];
    (_this select 0) setVariable ["skdfx_signalUp", nil, true];
}, [_unit]] call skd_fnc_cleanup;

["the counts start at zero", {
    ["server", { private _u = _this select 0; [_u getVariable ["skdfx_signalDown", 0], _u getVariable ["skdfx_signalUp", 0]] },
        "==", [0, 0], "down and up counts before the press", 10, [_unit]] call skd_fnc_expectOn;
}, []] call skd_fnc_step;

["press the signal", {
    private _pressed = ["p1", "skd_fixture", "signal"] call skd_fnc_pressKey;
    [_pressed, "==", true, "the value skd_fnc_pressKey returned"] call skd_fnc_expect;
}] call skd_fnc_step;

["the server reads one down and one up", {
    ["server", { private _u = _this select 0; [_u getVariable ["skdfx_signalDown", 0], _u getVariable ["skdfx_signalUp", 0]] },
        "==", [1, 1], "down and up counts after the press", 10, [_unit]] call skd_fnc_expectOn;
}] call skd_fnc_step;
