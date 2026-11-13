// @check US-021 The documented way to read what a keybind's code returned reaches the code skd_fnc_pressKey calls
// @tags US-021 ACT-02 m10-3 acceptance

// docs/check-api.md, "Acting through the mod": a check that asserts whether a key is consumed calls the keybind's
// code itself on the player's machine, as skd_fnc_pressKey does, and reads its value. The fixture's signal key
// counts each call of its down code on the player's unit (spec/contract/fixture-mod.md).
private _unit = ["p1"] call skd_fnc_player;
[{ (_this select 0) setVariable ["skdfx_signalDown", nil, true]; (_this select 0) setVariable ["skdfx_signalUp", nil, true] }, [_unit]] call skd_fnc_cleanup;

private _read = {
    params ["_mod", "_action"];
    ["p1", {
        params ["_mod", "_action"];
        private _bind = [_mod, _action] call CBA_fnc_getKeybind;
        private _key = (_bind select 8) param [0, _bind select 5];
        _key params ["_dik", "_mods"];
        _mods params ["_shift", "_ctrl", "_alt"];
        private _out = nil;
        isNil { _out = [findDisplay 46, _dik, _shift, _ctrl, _alt] call (_bind select 3) };
        [isNil "_out", if (isNil "_out") then { "nil" } else { _out }]
    }, [_mod, _action]] call skd_fnc_on
};

["the fixture's signal down code runs once through the documented way", {
    private _got = ["skd_fixture", "signal"] call _read;
    ["fixture signal [nil, value]", _got] call skd_fnc_evidence;
    ["p1", { (_this select 0) getVariable ["skdfx_signalDown", 0] }, "==", 1, "skdfx_signalDown after one call of the down code", 10, [_unit]] call skd_fnc_expectOn;
}] call skd_fnc_step;

["the docs' example: off a rope arma-grappling-hook's let go passes on to the game", {
    private _got = ["Grappling hooks", "agh_letGo"] call _read;
    ["agh_letGo [nil, value]", _got] call skd_fnc_evidence;
    [_got, "==", [false, false], "what agh_letGo's down code returned off a rope"] call skd_fnc_expect;
}] call skd_fnc_step;
