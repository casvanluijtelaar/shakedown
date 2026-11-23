// @check US-021 Mode down runs only the key-down code and a later mode up the key-up code
// @tags US-021 ACT-02 acceptance

private _unit = ["p1"] call skd_fnc_player;
[{
    (_this select 0) setVariable ["skdfx_signalDown", nil, true];
    (_this select 0) setVariable ["skdfx_signalUp", nil, true];
}, [_unit]] call skd_fnc_cleanup;

private _counts = { private _u = _this select 0; [_u getVariable ["skdfx_signalDown", 0], _u getVariable ["skdfx_signalUp", 0]] };

["the counts start at zero", {
    ["server", _this select 0, "==", [0, 0], "down and up counts before the press", 10, [_this select 1]] call skd_fnc_expectOn;
}, [_counts, _unit]] call skd_fnc_step;

["press down", {
    private _pressed = ["p1", "skd_fixture", "signal", "down"] call skd_fnc_pressKey;
    [_pressed, "==", true, "the value skd_fnc_pressKey returned for down"] call skd_fnc_expect;
    ["server", _this select 0, "==", [1, 0], "down and up counts after the key went down", 10, [_this select 1]] call skd_fnc_expectOn;
}, [_counts, _unit]] call skd_fnc_step;

["press up", {
    private _pressed = ["p1", "skd_fixture", "signal", "up"] call skd_fnc_pressKey;
    [_pressed, "==", true, "the value skd_fnc_pressKey returned for up"] call skd_fnc_expect;
    ["server", _this select 0, "==", [1, 1], "down and up counts after the key went up", 10, [_this select 1]] call skd_fnc_expectOn;
}, [_counts, _unit]] call skd_fnc_step;
